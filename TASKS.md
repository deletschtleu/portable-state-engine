# Portable State Engine — Tasks & Traceability

This file is the canonical task board and requirement-to-test traceability map for `portable-state-engine` (`SPEC.md` PSE-001).

**Status progression:** `Backlog → Refined → Ready → In Progress → Review → Verified → Done` (or `Blocked`).

---

## 1. Milestone Plan (Baseline M0–M3 & Follow-on M4–M8)

- **M0 — Project Skeleton & Language/Toolchain ADR:** Choose language/toolchain, record ADR-0001, configure reproducible build/test runner (`TASK-001`).
- **M1 — Generic Engine Core & In-Memory Store:** Pure domain contracts, error model, pure `replay` & historical revision queries, in-memory store with atomic expected-version append, and generic command runner (`TASK-002` to `TASK-005`).
- **M2 — Reference Domain A (Todo) & CLI Time-Travel Demo:** Pure Todo rules (`AddTodo`, `CompleteTodo`, `DeleteTodo`, permanent IDs) and runnable CLI for commands, history, and read-only historical queries (`TASK-006`, `TASK-008`).
- **M3 — Reference Domain B (Tic-Tac-Toe):** Pure 3×3 Tic-Tac-Toe rules (`MakeMove`, win/draw detection, illegal move rejection, cross-instance replay) on the unchanged engine core (`TASK-007`).
- **M4–M8 (Deferred until M0–M3 complete):** Durable store adapter (`TASK-009`), snapshots & schema evolution (`TASK-010`), optional HTTP/UI (`TASK-011`), optional outbox/virtual lamp (`TASK-012`), optional P2P verifiable games (`TASK-013`).

---

## 2. Requirement Traceability Matrix

| Requirement ID | Summary | Milestone | Task ID(s) | Verification / Test Target | Status |
|---|---|---|---|---|---|
| **FR-001** | Generic domain contract (`initialState`, `decide`, `evolve`) | M1 | TASK-002 | Contract unit tests | Planned |
| **FR-002** | Deterministic state reconstruction | M1 | TASK-003 | Replay property & unit tests | Planned |
| **FR-003** | Immutable state and event handling | M1 | TASK-002, TASK-003 | Immutability checks in replay/domain tests | Planned |
| **FR-004** | Invalid commands rejected without recording events | M1, M2, M3 | TASK-002, TASK-005, TASK-006, TASK-007 | Runner + domain rejection tests | Planned |
| **FR-005** | Replay current state and state at revision `0 <= r <= current` | M1 | TASK-003, TASK-005 | Revision boundary & replay tests | Planned |
| **FR-006** | Read-only time travel (no mutation or effects) | M1, M2 | TASK-003, TASK-005, TASK-008 | Historical query side-effect-free tests | Planned |
| **FR-007** | Ascending aggregate version ordering; no gaps/duplicates | M1 | TASK-004 | Store contract tests | Planned |
| **FR-008** | Optimistic concurrency via `expectedRevision` & conflict error | M1 | TASK-004, TASK-005 | Store concurrency & runner conflict tests | Planned |
| **FR-009** | Atomic batch append per accepted command | M1 | TASK-004 | Store batch atomicity tests | Planned |
| **FR-010** | Independent streams and versions per aggregate ID | M1 | TASK-004 | Store multi-aggregate isolation tests | Planned |
| **FR-011** | Replaceable storage/serialization/presentation boundaries | M1 | TASK-002, TASK-004, TASK-005 | Architecture boundary & zero-import checks | Planned |
| **FR-012** | Generic command runner for both reference domains | M1, M2, M3 | TASK-005, TASK-006, TASK-007 | Cross-domain runner tests | Planned |
| **FR-013** | Distinct error categories (validation, not-found/ID, conflict, infra) | M1 | TASK-002, TASK-005 | Error model unit tests | Planned |
| **FR-014** | Deterministic in-memory event store adapter | M1 | TASK-004 | In-memory store contract test suite | Planned |
| **FR-015** | Durable storage adapter (e.g., PostgreSQL) | M4 | TASK-009 | Durable store integration tests | Deferred (M4) |
| **FR-016** | Event envelope metadata & deterministic payload | M1 | TASK-002, TASK-004 | Envelope contract & store tests | Planned |
| **FR-017** | Snapshot validity & compatibility metadata | M5 | TASK-010 | Stale/incompatible snapshot fallback tests | Deferred (M5) |
| **FR-018** | Snapshot equivalence to full replay | M5 | TASK-010 | Snapshot vs full-replay property tests | Deferred (M5) |
| **FR-019** | Event schema evolution / upcasting strategy | M5 | TASK-010 | Versioned event compatibility tests | Deferred (M5) |
| **FR-020** | Crash & incomplete batch recovery | M4 | TASK-009 | Durable adapter transaction rollback tests | Deferred (M4) |
| **FR-021** | Explicit external effects outside `decide`/`evolve`/replay | M7 | TASK-012 | Effect isolation tests | Deferred (M7) |
| **FR-022** | Transactional outbox & idempotent delivery | M7 | TASK-012 | Outbox retry & deduplication tests | Deferred (M7) |
| **FR-023** | Honest desired vs reported external state | M7 | TASK-012 | Virtual lamp state transition tests | Deferred (M7) |
| **FR-024** | Inspectable aggregate event history ordered by revision | M1, M2 | TASK-004, TASK-005, TASK-008 | History listing & CLI output tests | Planned |
| **FR-025** | CLI/API demonstrator for commands, state, history, time travel | M2, M3 | TASK-008 | CLI end-to-end smoke tests | Planned |
| **FR-026** | Todo & Tic-Tac-Toe run on unchanged core/runner/store contracts | M2, M3 | TASK-006, TASK-007, TASK-008 | Zero core diff verification on M3 | Planned |
| **NFR-001..010** | Determinism, testability, concurrency, reliability, security, docs | M0–M3 | TASK-001..TASK-008 | Enforced across unit/contract/smoke suites | Planned |

---

## 3. Active & Refined Tasks (M0–M3)

## TASK-001 — Project skeleton, toolchain setup, and language ADR
Status: Ready
Task ID: TASK-001
Requirement IDs: Milestone M0, NFR-002, NFR-007, NFR-008, NFR-010
Dependencies: None
Owner / files in scope: `docs/adr/0001-language-and-toolchain.md`, `sbxenv.yaml` / `Dockerfile.sandbox` (if needed), project manifest & test runner config, `README.md`

### User-visible outcome
A developer or agent can run a single documented command to build and execute automated tests in a reproducible project setup, with the language/runtime choice documented in an ADR.

### Acceptance criteria
- [ ] Given a fresh checkout in the sandbox, when the documented build/test command is executed, then the test runner executes a smoke test and exits `0`.
- [ ] Invalid/edge scenario: Failing assertions cause the test runner to exit with a non-zero status code.
- [ ] Architecture Decision Record (`docs/adr/0001-language-and-toolchain.md`) documents the chosen language, test runner, and zero-dependency core rationale, and `sbxenv.yaml` / sandbox environment recipe is kept in sync if any runtime kit/package is added.

### Risks and decisions
- Sandbox base image is `Docker Hardened Images/Debian GNU/Linux 13 (trixie)`; any installed language toolchain must be persisted declaratively per `SANDBOX.md`.

### Test plan
- RED test(s): Initial sanity test verifying test runner catches a failing assertion before turning green.
- Integration/contract checks: Single build/test command succeeds cleanly.
- Full verification command(s): Project test command (to be finalized in ADR-0001).

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## TASK-002 — Core domain contracts, event envelope, and error model
Status: Ready
Task ID: TASK-002
Requirement IDs: FR-001, FR-003, FR-004, FR-011, FR-013, FR-016, NFR-001, NFR-002, NFR-006
Dependencies: TASK-001
Owner / files in scope: `src/core/contracts.*`, `src/core/errors.*`, `tests/core/contracts.test.*`

### User-visible outcome
Domain modules can be defined against a strictly typed, domain-independent contract (`initialState`, `decide`, `evolve`, `ExplicitContext`, `EventEnvelope`) with structured, distinguishable error categories.

### Acceptance criteria
- [ ] Given a dummy domain definition implementing `AggregateDefinition<State, Command, Event>`, when `decide` accepts or rejects a command, then the result clearly distinguishes accepted events (including `[]` no-op) from a `DomainRejectionError`.
- [ ] Invalid/edge scenario: Error types distinguish `DomainRejectionError`, `InvalidAggregateIdError` / `AggregateNotFoundError`, `RevisionConflictError`, `InvalidRevisionError`, and `StorageFailureError` with diagnostic context (`aggregateId`, `expectedRevision`, `actualRevision`, etc.).
- [ ] Event envelope structure captures `aggregateId`, `revision` (positive integer `1..n`), `eventId`, `eventType`, `schemaVersion`, `payload`, and `recordedAt` metadata without coupling `decide`/`evolve` to storage.

### Risks and decisions
- Keep `ExplicitContext` optional or defaulting to empty so simple domains do not require ceremony while supporting deterministic metadata when needed.

### Test plan
- RED test(s): Unit tests constructing each error category, verifying envelope invariants, and testing pure decision results (accepted events, empty no-op list, domain rejection).
- Integration/contract checks: Verify `src/core` has zero imports from domain or adapter modules.
- Full verification command(s): Project build + unit test suite.

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## TASK-003 — Pure `replay` and historical revision reconstruction
Status: Ready
Task ID: TASK-003
Requirement IDs: FR-002, FR-003, FR-005, FR-006, FR-007, NFR-001, NFR-002, NFR-010
Dependencies: TASK-002
Owner / files in scope: `src/core/replay.*`, `tests/core/replay.test.*`

### User-visible outcome
The engine can deterministically fold an ordered event stream into an initial state to compute current state or the historical state at any revision `0 <= r <= currentRevision` without mutating inputs or triggering side effects.

### Acceptance criteria
- [ ] Given an empty event stream, when `replay` is called, then it returns `initialState` at revision `0` (Invariant 1).
- [ ] Given an event stream `history + newEvents`, when `replay` is called on the combined stream, then the resulting state and revision equal folding `evolve` over `newEvents` starting from `replay(history)` (Invariant 2).
- [ ] Given a stream of `n` events and a target revision `r` (`0 <= r <= n`), when `replayAtRevision` is called, then it returns the exact state at revision `r` without mutating `initialState`, intermediate states, or the input event list (Invariant 3, 8).
- [ ] Invalid/edge scenario: Requesting a negative revision (`r < 0`), a revision beyond committed history (`r > n`), or replaying non-contiguous/out-of-order event revisions raises a typed error (`InvalidRevisionError` / `CorruptedStreamError`).

### Risks and decisions
- Enforce contiguous ascending revisions (`1, 2, ..., n`) during replay so corrupted streams fail fast.

### Test plan
- RED test(s): Unit and property/invariant tests for empty stream, prefix replay (`0..n`), incremental fold equivalence, input immutability, and out-of-range/non-contiguous revision errors.
- Integration/contract checks: Pure function execution with zero I/O.
- Full verification command(s): Project build + unit test suite.

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## TASK-004 — In-memory event store with atomic expected-version append
Status: Ready
Task ID: TASK-004
Requirement IDs: FR-007, FR-008, FR-009, FR-010, FR-014, FR-016, FR-024, NFR-003
Dependencies: TASK-002
Owner / files in scope: `src/core/store.*`, `src/adapters/memory-store.*`, `tests/adapters/store-contract.test.*`

### User-visible outcome
An in-memory event store persists ordered event envelopes per aggregate stream, enforces atomic optimistic concurrency (`expectedRevision`), isolates separate aggregate IDs, and exposes a reusable store contract test suite.

### Acceptance criteria
- [ ] Given an aggregate at revision `r` (initially `0`), when a batch of `k` events is appended with `expectedRevision = r`, then all `k` events are assigned contiguous revisions `r+1 .. r+k` and committed atomically (Invariant 4).
- [ ] Given an aggregate at revision `r`, when an empty event batch (`[]`) is appended with `expectedRevision = r`, then the stream remains at revision `r` with no new events.
- [ ] Given two appends against the same aggregate with the same `expectedRevision = r` where both are non-empty, when executed sequentially or concurrently, then the first succeeds and the second fails with `RevisionConflictError` leaving the first batch intact (Invariant 6).
- [ ] Given events appended to `aggregateA` and `aggregateB`, when reading either stream or revision, then each aggregate stream is completely isolated (FR-010).
- [ ] Invalid/edge scenario: Blank/invalid `aggregateId` is rejected with `InvalidAggregateIdError`; failed appends never leave partial batches in the stream (FR-009).

### Risks and decisions
- Write the tests as a reusable `runEventStoreContractSuite(createStore)` so M4's durable adapter can execute the exact same contract tests.

### Test plan
- RED test(s): Reusable store contract tests covering empty stream (`rev 0`), single/multi-event atomic batch append, `RevisionConflictError` on stale/future `expectedRevision`, concurrent append serialization, aggregate isolation, and history listing.
- Integration/contract checks: Verify returned event lists are defensive copies so caller mutation cannot corrupt stored history.
- Full verification command(s): Project build + unit/contract test suite.

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## TASK-005 — Generic command runner and read-only query service
Status: Ready
Task ID: TASK-005
Requirement IDs: FR-004, FR-005, FR-006, FR-008, FR-009, FR-012, FR-013, FR-024, NFR-003, NFR-006
Dependencies: TASK-003, TASK-004
Owner / files in scope: `src/core/runner.*`, `tests/core/runner.test.*`

### User-visible outcome
A generic application runner executes commands (`load -> decide -> append -> evolve`) with optional bounded conflict retries, and provides read-only queries for current state, historical state at revision `r`, and ordered event history.

### Acceptance criteria
- [ ] Given a valid command, when `execute` is called on the runner, then it loads current events, reconstructs state, calls `decide`, atomically appends resulting events with `expectedRevision`, and returns the new state, new revision, and committed event envelopes.
- [ ] Given an invalid command rejected by `decide`, when `execute` is called, then it returns the `DomainRejectionError` and appends zero events to the store (Invariant 5).
- [ ] Given a command that produces an empty event list (`[]`, no-op), when `execute` is called, then the state and revision remain unchanged and no events are written.
- [ ] Given a historical query `getStateAtRevision(aggregateId, r)` or `getHistory(aggregateId)`, when executed, then it returns the state at revision `r` (or event list) without modifying stored events or current state (FR-006, Invariant 3).
- [ ] Invalid/edge scenario: When a concurrent write causes a `RevisionConflictError` during `execute`, the runner either returns `RevisionConflictError` immediately (default `maxRetries = 0`) or reloads current state and re-runs the entire `decide` function up to `maxRetries` times, never blindly appending stale events (FR-008).

### Risks and decisions
- Ensure the runner has zero domain-specific branches so both Todo and Tic-Tac-Toe plug directly into it.

### Test plan
- RED test(s): Runner tests for accepted commands, multi-event commands, no-op commands, domain rejections, `getStateAtRevision` (`0..current` and out-of-bounds), `getHistory`, and conflict retry/exhaustion behavior using a deterministic conflict-injecting store spy.
- Integration/contract checks: Core + memory store integration.
- Full verification command(s): Project build + unit/contract test suite.

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## TASK-006 — Reference domain A: Todo list module
Status: Ready
Task ID: TASK-006
Requirement IDs: Milestone M2, FR-001..FR-006, FR-012, FR-026, Section 5 (Todo specification)
Dependencies: TASK-005
Owner / files in scope: `src/domains/todo/*`, `tests/domains/todo.test.*`

### User-visible outcome
A pure Todo domain module supports `AddTodo`, `CompleteTodo`, and `DeleteTodo` with permanent IDs, deterministic ordering, and time-travel inspection through the generic engine runner.

### Acceptance criteria
- [ ] **Scenario: Replay a Todo list** — Given an empty list at revision 0, when `AddTodo("a", "Write tests")`, `AddTodo("b", "Build engine")`, and `CompleteTodo("a")` are accepted, then current revision is 3, Todo `"a"` is completed, Todo `"b"` is open, and state at revision 1 contains only open Todo `"a"`.
- [ ] **Scenario: Reject duplicate IDs** — Given `AddTodo("a", "Write tests")` is committed, when `AddTodo("a", "Something else")` is requested, then the command is rejected and no new event is appended.
- [ ] **Scenario: Permanent ID after deletion** — Given Todo `"a"` was added and then deleted via `DeleteTodo("a")`, when `AddTodo("a", "Reused ID")` is requested, then the command is rejected because `"a"` remains in `usedIds`, while revision 1 still reconstructs Todo `"a"`.
- [ ] **Scenario: Historical read is side-effect-free** — Given a list with 3 events, when state at revision 1 is requested, then its contents match the first event applied to initial state and current revision/history are unchanged.
- [ ] Invalid/edge scenarios:
  - `AddTodo` with blank or whitespace-only title or blank ID is rejected.
  - `CompleteTodo` on an already completed Todo succeeds as a no-op (`[]` events, revision unchanged).
  - `CompleteTodo` or `DeleteTodo` on an unknown or already-deleted ID is rejected without emitting events.
  - Todo items are returned in deterministic append order, and state objects are deeply immutable.

### Risks and decisions
- Track `usedIds` alongside active `todos` in the immutable `TodoListState` so deleted IDs cannot be reused after replay.

### Test plan
- RED test(s): Unit tests for pure `decide`/`evolve` and runner-integrated acceptance scenarios from `SPEC.md` Section 5 + edge cases (whitespace title, deleted ID reuse, idempotent complete, immutability).
- Integration/contract checks: Verify zero changes to `src/core/*`.
- Full verification command(s): Project build + full test suite.

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## TASK-007 — Reference domain B: Tic-Tac-Toe module
Status: Ready
Task ID: TASK-007
Requirement IDs: Milestone M3, FR-001..FR-006, FR-012, FR-026, Section 6 (Tic-Tac-Toe specification)
Dependencies: TASK-005
Owner / files in scope: `src/domains/tic-tac-toe/*`, `tests/domains/tic-tac-toe.test.*`

### User-visible outcome
A pure 3×3 Tic-Tac-Toe domain module processes `MakeMove(player, cell)` commands, detects all row/column/diagonal wins and draws deterministically, rejects illegal moves, and replays identically across independent engine instances without any changes to the core engine.

### Acceptance criteria
- [ ] **Scenario: Deterministic victory** — Given a new game with `X` to move (`cells 0..8` empty, status `in-progress`), when `X` moves at `0`, `O` at `3`, `X` at `1`, `O` at `4`, and `X` at `2`, then `X` is the winner (`status = "X-won"`, `nextPlayer = null`), the game is terminal, and a further move is rejected without an event.
- [ ] **Scenario: Reject illegal move** — Given `X` has moved at cell `0`, when `O` attempts cell `0` (occupied), or `X` attempts to move out of turn, or a player attempts cell `-1` or `9` (out of range), then the command is rejected and the revision and board are unchanged.
- [ ] **Scenario: All win lines and draw** — Winner detection covers all 3 rows (`0,1,2`, `3,4,5`, `6,7,8`), all 3 columns (`0,3,6`, `1,4,7`, `2,5,8`), both diagonals (`0,4,8`, `2,4,6`), and a full 9-move non-winning board transitions to `status = "draw"` (and rejects subsequent moves).
- [ ] **Scenario: Cross-instance replay** — Given two independent engine instances with the same rules and initial state, when both replay the same ordered game events, then both produce equivalent boards, turns, and status, and historical revisions (`0..n`) reconstruct every prior turn accurately.

### Risks and decisions
- Cell addressing is `0..8` in row-major order (`0..2` top row, `3..5` middle row, `6..8` bottom row); game starts at revision `0` with `X` to move and status `in-progress`.
- Core files (`src/core/*`) MUST remain untouched when implementing this task.

### Test plan
- RED test(s): Table-driven unit tests for all 8 winning lines (for both X and O), draw scenario, out-of-turn/occupied/out-of-bounds/post-terminal rejections, cross-instance replay, and time-travel revision inspection.
- Integration/contract checks: `git diff` confirms zero modifications to `src/core/*`.
- Full verification command(s): Project build + full test suite.

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## TASK-008 — CLI demonstrator for Todo and Tic-Tac-Toe with history and time travel
Status: Ready
Task ID: TASK-008
Requirement IDs: Milestone M2, Milestone M3, FR-005, FR-006, FR-024, FR-025, FR-026, NFR-008
Dependencies: TASK-006, TASK-007
Owner / files in scope: `src/cli/*`, `tests/cli/cli.test.*`, `README.md`

### User-visible outcome
Users and automated tests can run a CLI demonstrator that executes Todo and Tic-Tac-Toe commands, prints current state, lists ordered event history, and queries historical state at any revision `r` without mutating state or history.

### Acceptance criteria
- [ ] Given the CLI demo command (or interactive/scripted subcommands), when run for Todo and Tic-Tac-Toe, then it demonstrates accepted commands, rejected invalid commands, ordered event history (`FR-024`), and read-only time-travel state queries at earlier revisions (`FR-006`, `FR-025`).
- [ ] Invalid/edge scenario: Issuing an invalid command or out-of-range revision query via the CLI reports a clear error without corrupting the aggregate stream or crashing unhandled.
- [ ] `README.md` documents the quickstart, single build/test command, CLI demo usage, architecture boundaries, invariants, and storage tradeoffs (`NFR-008`).

### Risks and decisions
- Support both a deterministic scripted/demo mode (for one-command verification per Section 11) and command-line arguments/subcommands for inspecting commands, history, and revisions.

### Test plan
- RED test(s): Automated CLI smoke and integration tests verifying stdout/stderr and exit codes for Todo and Tic-Tac-Toe workflows, including verifying that time-travel queries leave current revision and history unchanged.
- Integration/contract checks: End-to-end execution of the documented demo command.
- Full verification command(s): Single build/test command + demo execution command.

### Completion evidence
- Changes: Pending
- Test results (commands, pass/fail): Pending
- Review findings/resolution: Pending
- Passing incremental commits / PR: Pending
- Remaining limitations / follow-up: Pending

---

## 4. Deferred Follow-on Tasks (M4–M8, Out of Scope for Baseline M0–M3)

- **TASK-009 (M4 — Backlog):** Durable event-store adapter (e.g., PostgreSQL/SQLite), schema migrations, transactional batch append, and restart recovery tests (`FR-015`, `FR-020`, `NFR-004`). Depends on `TASK-004`.
- **TASK-010 (M5 — Backlog):** Snapshots (`FR-017`, `FR-018`) and event schema version upcasting/compatibility (`FR-019`, `NFR-009`). Depends on `TASK-009`.
- **TASK-011 (M6 Optional — Backlog):** HTTP/UI time-travel demonstrator (`SPEC.md` M6). Depends on `TASK-006`, `TASK-008`.
- **TASK-012 (M7 Optional — Backlog):** Transactional outbox, effect dispatcher, and virtual lamp domain (`FR-021`, `FR-022`, `FR-023`). Depends on `TASK-009`.
- **TASK-013 (M8 Future — Backlog):** Peer-to-peer verifiable game extension with signed envelopes and hash-chained replay (`SPEC.md` Section 7). Depends on `TASK-007`.
