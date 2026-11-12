# Portable Event-Driven State Engine — Product & Technical Specification

**Specification ID:** PSE-001  
**Version:** 1.0.0  
**Status:** Implementation-ready reference specification  
**Language/runtime:** Independent  
**Normative terms:** **MUST**, **MUST NOT**, **SHOULD**, **MAY** have their usual requirement meanings.

## 1. Purpose

Build a small, reusable engine that models deterministic domain behavior as commands, accepted events, and state transitions. It MUST be practical for both a time-travel Todo application and a turn-based game such as Tic-Tac-Toe without altering the engine's domain-independent core. It SHOULD accommodate additional domains (for example, a virtual lamp or a financial ledger) through adapters and domain modules.

This specification defines observable behavior and invariants, **not** a programming language, framework, database, deployment stack, folder layout, or implementation technique. An implementation MAY use functional programming, OOP, or a combination.

### Goals

- Decouple domain rules from persistence, transport, UI, clocks, external effects, and infrastructure.
- Rebuild state deterministically from an initial state and an ordered event history.
- Reject invalid commands without recording domain events or executing their effects.
- Support read-only inspection of historical states (time travel).
- Provide a replaceable event-store boundary and explicit concurrency rules.
- Demonstrate reuse with **two independent domain modules: Todo and Tic-Tac-Toe**.
- Make features independently testable and implementations reproducible from this document.

### Non-goals for the initial release

- General-purpose distributed consensus, peer-to-peer networking, production banking, double-entry accounting, complex workflow orchestration, visual state-machine designers, Kafka integration, or mandatory cloud infrastructure.
- A promise of exactly-once delivery to external systems.
- A claim that every business process can be safely represented as one aggregate or one global state machine.

## 2. Vocabulary and semantics

| Term | Definition |
|---|---|
| **Aggregate** | Independently versioned logical entity whose domain events are ordered (e.g., one Todo-list or one game). |
| **State** | Immutable, domain-specific value representing the aggregate after a prefix of events. |
| **Command** | Request for a change; may be rejected or produce zero or more events. |
| **Event** | Immutable, past-tense record of a domain fact accepted by the system. |
| **decide** | Pure domain function: `(state, command, context) -> accepted events OR domain error`. Context contains only explicit, deterministic inputs when necessary. |
| **evolve** | Pure domain function: `(state, event) -> state'`. |
| **Replay** | Fold the ordered event stream into an initial state, optionally starting from a verified compatible snapshot. |
| **Snapshot** | Cached state plus its event version; never the canonical history. |
| **Effect** | External interaction (e.g., network message, notification); NEVER run during replay. |
| **Revision/version** | Nonnegative integer equal to the count of committed events in an aggregate stream. Initial version is `0`. |
| **Projection** | Derived view of events/state, such as an API response or analytics view. |

**Core algebra:**

```text
initialState : AggregateDefinition -> State

decide : State × Command × ExplicitContext -> Either<DomainError, List<Event>>
evolve : State × Event -> State
replay(S0, [E1...En]) = fold(evolve, S0, [E1...En])

execute(command):
  (state, revision) <- loadCurrent(aggregateId)
  newEvents        <- decide(state, command, explicitContext)
  on error         -> return rejection, without append
  on success       -> appendAtomically(aggregateId, expectedRevision=revision, newEvents)
  return evolveAll(state, newEvents) and newRevision
```

A successful no-op command MAY produce an empty event list; this leaves the aggregate revision unchanged. `decide` MUST NOT rely on ambient time, global randomness, mutable global variables, databases, or network calls. If timestamps or IDs are required, supply them explicitly at the boundary and include necessary resulting values in events.

## 3. Architecture and boundaries

```text
         UI / CLI / HTTP / other adapters
                    |
                 Commands
                    v
           Application / Runner
          +---------------------+
          | load state + version|
          | decide()            |----> Domain rejection
          | append(expected ver)|
          | evolve()/return     |
          +---------------------+
                |          |
                v          v
       Event Store      Optional outbox
                |          |
                v          v
       Events/Snapshots  Effect dispatcher ---> External systems
                |
                v
       Replay / projections / read-only historical queries

Domain module: initialState + decide + evolve + domain types/rules
Engine core: orchestration contracts + replay + result/error conventions
Adapters: memory store, optional durable store, API, CLI, UI, external transports
```

The core MUST NOT import domain-specific Todo/Game types or depend directly on an HTTP framework, SQL driver, UI toolkit, or MQTT client. Domain modules MUST NOT directly execute effects or write their own events. A production-grade outbox is an optional later capability, not a prerequisite of the initial examples.

## 4. Functional requirements

### Core engine

- **FR-001 Generic domain definition:** Define a type-safe or equivalent contract for initial state, `decide`, and `evolve`, parameterized by domain state, command, and event. Domain-specific rule code MUST live outside the engine.
- **FR-002 Determinism:** Given identical initial state, event sequence, rule version, and explicit context, replay MUST produce an equivalent state and version on every execution.
- **FR-003 Immutability:** Neither `decide` nor `evolve` may mutate input state/events; results MUST behave as immutable values, including nested collections.
- **FR-004 Validation:** Invalid commands MUST return a domain-level rejection, not write any events. Rejections MUST be distinguishable from infrastructure failures.
- **FR-005 Replay:** Reconstruct current state and a state at any valid committed revision `r` with `0 <= r <= currentRevision`.
- **FR-006 Read-only time travel:** Historical queries MUST NOT modify persisted events, cached current state, or dispatch external effects. A rewind/undo operation, if implemented, MUST be explicit new domain behavior rather than editing history.
- **FR-007 Ordering:** Events MUST be processed in ascending aggregate version. A store MUST prohibit missing or duplicate versions within an aggregate stream.
- **FR-008 Concurrency:** Appending MUST support `expectedRevision` and atomically fail on conflict. The caller MUST receive a typed conflict and MAY reload and retry the *entire* decision, with bounded retries. It MUST NOT blindly append events decided against a stale state.
- **FR-009 Append atomicity:** A batch generated by one accepted command MUST be committed entirely or not at all.
- **FR-010 State isolation:** Separate aggregate IDs MUST have independent event streams and versions.
- **FR-011 Portability:** Storage, serialization, framework, language, and presentation choices MUST be replaceable without modifying domain semantics.
- **FR-012 Reusable execution:** A generic runner/application service MUST orchestrate load, decide, append, and return state for both reference domains, with no domain-specific branches.
- **FR-013 Error model:** Expose at least validation/rejection, aggregate-not-found or invalid ID, version conflict, and infrastructure/storage failure categories.

### Persistence

- **FR-014 In-memory adapter:** Supply a deterministic in-memory event-store implementation for quickstart and tests. Document that it loses data on process restart.
- **FR-015 Durable adapter:** Supply or plan a durable storage adapter in a later milestone. Preferred reference adapter: PostgreSQL; no database is mandated by this specification.
- **FR-016 Event envelope:** Persist aggregate ID, aggregate version, event ID, event type, schema version, event payload, and recorded timestamp. Event payload MUST contain the deterministic information necessary for replay. Timestamps are metadata unless explicitly part of domain state.
- **FR-017 Snapshot validity:** A snapshot MUST include aggregate ID, covered event version, state schema/rule compatibility metadata, and serialized state. A stale or incompatible snapshot MUST be ignored/rebuilt, never silently trusted.
- **FR-018 Snapshot semantics:** Snapshots MAY accelerate current-state reconstruction but MUST NOT change results. A correct implementation MUST work without any snapshots.
- **FR-019 Event evolution:** Event types and versions MUST have an explicit strategy for backwards compatibility (e.g., upcasters, version-aware readers, or controlled migrations). Historic events MUST NOT be rewritten casually.
- **FR-020 Failure recovery:** After a crash, committed events MUST remain replayable, and incomplete commands MUST NOT leave partial batches. Durable-store tests MUST verify this with the selected backend.

### External effects (optional milestone)

- **FR-021 Explicit effects:** External communication MUST be triggered outside `decide`, `evolve`, and replay. Model effects/intentions as explicit outputs or persistence records.
- **FR-022 Safe delivery:** For durable external messaging, persist dispatch intention atomically with the accepted events (transactional outbox or equivalent). Consumers MUST tolerate repeated delivery, typically using command/event IDs or desired-state commands.
- **FR-023 Honest status:** An accepted request MUST NOT be reported as physically completed before the external system confirms it. A device command and device-reported state are different facts.

### Inspectability and developer experience

- **FR-024 History:** Allow listing an aggregate's events ordered by revision with stable IDs and types.
- **FR-025 CLI/API demonstrator:** Provide a small runnable interface (CLI is enough for the initial milestone) to issue commands, display current state, view history, and query state at a revision.
- **FR-026 Reusable examples:** Todo and Tic-Tac-Toe MUST use the same core/runner/store contracts without core changes between them.

## 5. Reference domain A — Todo list

**Aggregate:** One Todo list identified by `listId`.

**State:** `todos` keyed by stable todo ID; each Todo has `id`, nonblank `title`, `completed` Boolean. Initial state is an empty collection.

**Commands:** `AddTodo(id, title)`, `CompleteTodo(id)`, `DeleteTodo(id)`.

**Events:** `TodoAdded(id, title)`, `TodoCompleted(id)`, `TodoDeleted(id)`.

**Rules:**

- Add rejects blank/whitespace-only titles and duplicate IDs.
- Complete rejects unknown IDs. Completing an already completed Todo is a successful no-op (no event).
- Delete rejects unknown IDs. Deleted Todos are absent in the current projection but remain reconstructible via earlier revisions.
- Once deleted, an ID MAY NOT be reused within the same list (prefer permanent identity). To enforce this from replay, retain a `usedIds` set in aggregate state or otherwise derive it from the event history; do not rely solely on currently visible todos.
- Each accepted mutation emits exactly one domain event except explicit no-ops.
- UI ordering, if implemented, MUST be deterministic (e.g., append order stored explicitly, not unspecified map iteration order).

**Acceptance scenarios:**

```gherkin
Scenario: Replay a Todo list
  Given an empty list at revision 0
  When AddTodo("a", "Write tests") is accepted
  And AddTodo("b", "Build engine") is accepted
  And CompleteTodo("a") is accepted
  Then the current revision is 3
  And Todo "a" is completed
  And Todo "b" is open
  And the state at revision 1 contains only open Todo "a"

Scenario: Reject duplicate IDs
  Given AddTodo("a", "Write tests") has been committed
  When AddTodo("a", "Something else") is requested
  Then the command is rejected
  And no new event is appended

Scenario: Historical read is side-effect-free
  Given a list with 3 events
  When the state at revision 1 is requested
  Then its contents match the first event applied to initial state
  And the current revision and event history are unchanged
```

## 6. Reference domain B — Tic-Tac-Toe

**Aggregate:** One 3×3 game, identified by `gameId`.

**State:** Board of nine empty/X/O cells, participants X and O, next player, status (`in-progress`, `X-won`, `O-won`, `draw`). Rules MUST not depend on wall-clock time or storage. Choose a fixed cell addressing scheme (`0..8` is recommended) and document it.

**Commands:** `MakeMove(player, cell)`; game creation may be represented by an initial state with fixed participants or an explicit `CreateGame` event, but the choice MUST be documented and deterministic.

**Events:** `MoveMade(player, cell)`; explicit conclusion events are OPTIONAL if the result can be computed deterministically from the board.

**Rules:**

- Reject an out-of-range cell, occupied cell, incorrect player, or move after the game ends.
- An accepted move places exactly one mark and advances turn unless the game finishes.
- Winner detection covers all 3 rows, 3 columns and 2 diagonals.
- Exactly one terminal status must result: X win, O win, or draw, when applicable.
- Replaying the same ordered legal move events MUST produce the same state on separate machines.

**Acceptance scenarios:**

```gherkin
Scenario: Deterministic victory
  Given a new game with X to move
  When X moves at 0 and O at 3
  And X moves at 1 and O at 4
  And X moves at 2
  Then X is the winner
  And the game is terminal
  And a further move is rejected without an event

Scenario: Reject illegal move
  Given X has moved at cell 0
  When O attempts cell 0
  Then the command is rejected
  And the revision and board are unchanged

Scenario: Cross-instance replay
  Given two independent engine instances with the same rules and initial state
  When both replay the same ordered game events
  Then both produce equivalent boards, turns and status
```

## 7. Future optional extension — peer-to-peer verifiable games

The initial Tic-Tac-Toe module MUST work without networking or cryptography. A future extension MAY add:

- Deterministic, versioned rule identity and canonical event serialization.
- Signed move envelopes identifying game, actor, sequence, previous hash, and payload.
- Hash chaining for tamper evidence and replay-verifiable state hashes.
- Agreed ordering and fork/conflict handling; signatures and hash chains **alone do not provide consensus**.
- Swappable peer transports (e.g., WebRTC/WebSocket) and local persistence.
- Explicit threat model: identity, replay attacks, conflicting moves, dropped peers, rollback and rule upgrades.
- Privacy/hidden-information protocols for games that require them; full-state replication is NOT suitable for hidden hands.

P2P features are **out of scope until the engine and both local reference domains are complete**.

## 8. Nonfunctional requirements

- **NFR-001 Correctness over micro-optimization:** Core invariants and reproducible replay take priority over premature caching or parallelism.
- **NFR-002 Testability:** Every domain transition MUST be testable without databases, network, current time or threads.
- **NFR-003 Explicit concurrency:** No implementation may claim a memory-store concurrency guarantee it does not enforce. At minimum, serialize append and compare revisions atomically per stream.
- **NFR-004 Reliability:** Durable writes are transactional; replay failures and unknown event schema versions MUST surface explicit errors.
- **NFR-005 Security:** Validate untrusted commands at boundaries; ensure aggregate-level authorization is provided by the application adapter where necessary. Do not execute code deserialized from events.
- **NFR-006 Observability:** Provide structured diagnostic context for aggregate ID, command/event ID, revision and failures without exposing secrets.
- **NFR-007 Maintainability:** Small functions with meaningful names, minimal hidden mutation, no speculative abstractions, comments only when they explain non-obvious *why* or invariants.
- **NFR-008 Documentation:** Provide quickstart, architecture rationale, invariant list and storage tradeoffs.
- **NFR-009 Performance:** Correctness tests are mandatory; benchmark replay and snapshot benefits only after a durable adapter exists. Snapshot frequency is configurable or measured, not fixed by this specification.
- **NFR-010 Portability:** Avoid relying on unspecified iteration order, locale, noncanonical serialization or ambient time for domain outcomes.

## 9. Key invariants and consistency rules

1. `replay(emptyHistory) == initialState` at revision 0.
2. `replay(history + events) == fold(evolve, replay(history), events)`.
3. Historical reads never append events or dispatch effects.
4. All successful appends are contiguous and atomic within one aggregate.
5. Rejected commands cause zero persisted domain events.
6. Two appends with the same `expectedRevision` cannot both succeed against the same aggregate version when both are nonempty.
7. Snapshots are disposable caches; deleting them never changes externally observed state.
8. `decide` and `evolve` have no side effects and mutate no inputs.
9. Durable external effects are not assumed exactly-once; deduplication/idempotency is explicit.
10. Todo and game domains run without modifications to the generic engine.

## 10. Iterative delivery and acceptance gates

| Milestone | Deliverable | Exit gate | Dependencies |
|---|---|---|---|
| **M0** | Repository, build/test runner, language decision record, minimal README | Reproducible commands run locally | None |
| **M1** | Domain contracts, pure replay, error model, in-memory store with atomic versioned append, runner | Unit tests for determinism, rejection, isolation and concurrency pass | M0 |
| **M2** | Todo domain + CLI historical queries | All Todo acceptance scenarios pass | M1 |
| **M3** | Tic-Tac-Toe domain | All game acceptance scenarios pass with **no core changes** | M1; can proceed parallel to M2 |
| **M4** | Durable adapter (e.g., PostgreSQL), migration strategy, restart recovery | Integration tests for append atomicity, conflicts and restart pass | M1; may proceed alongside M2/M3 |
| **M5** | Snapshots and event schema compatibility | Snapshot-on/off and rebuild tests produce equivalent state | M4 |
| **M6 (optional)** | HTTP/UI time-travel demo | View current/historical states without mutation | M2; M4 optional for persistence |
| **M7 (optional)** | Outbox/effects and virtual lamp | Simulated failure, retries and reported-vs-desired-state tests pass | M4 |
| **M8 (future)** | P2P verifiable game extension | Threat-model tests; deterministic and authenticated replay | M3 |

The baseline deliverable is **M0–M3**. M4–M5 are the preferred persistent follow-on, not needed to prove core generality. Independent milestones MAY be worked in parallel once their shared contracts are stable.

## 11. Definition of acceptance / reproducibility

A fresh implementation is accepted only when:

- Every mandatory `FR` relevant to delivered milestones has a test or documented verification.
- The Todo and Tic-Tac-Toe modules execute using one unchanged engine.
- Time travel returns historical states without altering current state/history.
- Conflict, invalid-command and deterministic-replay tests pass.
- There is a single documented command for build/test and an executable demo.
- The repository contains an architecture decision record for deliberate implementation-specific choices.
- Any deferred requirement is called out clearly; no unsupported feature is claimed complete.

## 12. Open design choices (deliberately unspecified)

- Programming language, libraries, module layout, serialization format, logging stack.
- Single process vs multiple services; CLI vs HTTP-first.
- Whether event-store reads use streaming, pagination or batch loading.
- PostgreSQL vs another durable adapter; exact snapshot policy.
- Retry policy, authorization policy and distributed deployment topology.

The implementing agent MUST choose simple defaults, record tradeoffs in a short decision log, and avoid making these choices part of the portable core contract. Material departures from requirements require an explicit change proposal.
