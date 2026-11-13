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
| **FR-001** | Generic domain contract (`initialState`, `decide`, `evolve`) | M1 | TASK-002 | `tests/core_contracts.rs` | Verified |
| **FR-002** | Deterministic state reconstruction | M1 | TASK-003 | `tests/core_replay.rs` | Verified |
| **FR-003** | Immutable state and event handling | M1 | TASK-002, TASK-003 | `tests/core_contracts.rs`, `tests/core_replay.rs` | Verified |
| **FR-004** | Invalid commands rejected without recording events | M1, M2, M3 | TASK-002, TASK-005, TASK-006, TASK-007 | `tests/core_runner.rs`, `tests/todo_domain.rs`, `tests/tic_tac_toe_domain.rs` | Verified |
| **FR-005** | Replay current state and state at revision `0 <= r <= current` | M1 | TASK-003, TASK-005 | `tests/core_replay.rs`, `tests/core_runner.rs` | Verified |
| **FR-006** | Read-only time travel (no mutation or effects) | M1, M2 | TASK-003, TASK-005, TASK-008 | `tests/core_replay.rs`, `tests/core_runner.rs` | Verified (M1) |
| **FR-007** | Ascending aggregate version ordering; no gaps/duplicates | M1 | TASK-004 | `tests/store_contract.rs` | Verified |
| **FR-008** | Optimistic concurrency via `expectedRevision` & conflict error | M1 | TASK-004, TASK-005 | `tests/store_contract.rs`, `tests/core_runner.rs` | Verified |
| **FR-009** | Atomic batch append per accepted command | M1 | TASK-004 | `tests/store_contract.rs` | Verified |
| **FR-010** | Independent streams and versions per aggregate ID | M1 | TASK-004 | `tests/store_contract.rs` | Verified |
| **FR-011** | Replaceable storage/serialization/presentation boundaries | M1 | TASK-002, TASK-004, TASK-005 | `src/core/*` zero domain/adapter imports | Verified |
| **FR-012** | Generic command runner for both reference domains | M1, M2, M3 | TASK-005, TASK-006, TASK-007 | `tests/core_runner.rs`, `tests/todo_domain.rs`, `tests/tic_tac_toe_domain.rs` | Verified |
| **FR-013** | Distinct error categories (validation, not-found/ID, conflict, infra) | M1 | TASK-002, TASK-005 | `tests/core_contracts.rs`, `tests/core_runner.rs` | Verified |
| **FR-014** | Deterministic in-memory event store adapter | M1 | TASK-004 | `tests/store_contract.rs` | Verified |
| **FR-015** | Durable storage adapter (e.g., PostgreSQL) | M4 | TASK-009 | Durable store integration tests | Deferred (M4) |
| **FR-016** | Event envelope metadata & deterministic payload | M1 | TASK-002, TASK-004 | `tests/core_contracts.rs`, `tests/store_contract.rs` | Verified |
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
Status: Done
Task ID: TASK-001
Requirement IDs: Milestone M0, NFR-002, NFR-007, NFR-008, NFR-010
Dependencies: None
Owner / files in scope: `docs/adr/0001-language-and-toolchain.md`, `sbxenv.yaml`, `kits/rust-toolchain/*`, `Cargo.toml`, `Cargo.lock`, `.gitignore`, `src/lib.rs`, `README.md`

### User-visible outcome
A developer or agent can run a single documented command (`cargo test`) to build and execute automated tests in a reproducible project setup, with the language/runtime choice documented in ADR-0001.

### Acceptance criteria
- [x] Given a fresh checkout in the sandbox, when the documented build/test command is executed, then the test runner executes a smoke test and exits `0`.
- [x] Invalid/edge scenario: Failing assertions cause the test runner to exit with a non-zero status code.
- [x] Architecture Decision Record (`docs/adr/0001-language-and-toolchain.md`) documents the chosen language, test runner, and zero-dependency core rationale, and `sbxenv.yaml` + `kits/rust-toolchain/` are kept in sync with the installed Rust 1.85.1 package set.

### Risks and decisions
- Sandbox base image is `Docker Hardened Images/Debian GNU/Linux 13 (trixie)`; Rust 1.85.1 (`rustc`, `cargo`, `rustfmt`, `rust-clippy`, `build-essential`) is installed via `apt-get` and packaged declaratively as a local v3 mixin kit in `kits/rust-toolchain/` referenced by `sbxenv.yaml` per `SANDBOX.md`.

### Test plan
- RED test(s): Initial sanity test (`tests::reports_canonical_specification_id`) failed as expected (`left: "TODO"`, `right: "PSE-001"`) before implementation.
- Integration/contract checks: Single build/test command (`cargo test`) succeeds cleanly with zero warnings.
- Full verification command(s): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

### Completion evidence
- Changes: Added local v3 mixin kit `kits/rust-toolchain/` (`rust-toolchain.yaml` and `rust-toolchain.dockerfile`), updated `sbxenv.yaml`, wrote `docs/adr/0001-language-and-toolchain.md`, initialized `Cargo.toml`, `src/lib.rs`, `.gitignore`, and documented build/test commands in `README.md`.
- Test results (commands, pass/fail): `cargo test` (1 passed, 0 failed), `cargo clippy --all-targets -- -D warnings` (pass), `cargo fmt --check` (pass).
- Review findings/resolution: Verified zero external dependencies in `Cargo.toml`, pinned package versions in `kits/rust-toolchain/rust-toolchain.dockerfile`, and clean `git status`.
- Passing incremental commits / PR: Committed on branch `main` (see git log).
- Remaining limitations / follow-up: Core engine contracts and replay logic are implemented next in `TASK-002` through `TASK-005`.

---

## TASK-002 — Core domain contracts, event envelope, and error model
Status: Done
Task ID: TASK-002
Requirement IDs: FR-001, FR-003, FR-004, FR-011, FR-013, FR-016, NFR-001, NFR-002, NFR-006
Dependencies: TASK-001
Owner / files in scope: `src/core/mod.rs`, `src/core/contracts.rs`, `src/core/errors.rs`, `tests/core_contracts.rs`

### User-visible outcome
Domain modules can be defined against a strictly typed, domain-independent contract (`initialState`, `decide`, `evolve`, `ExplicitContext`, `EventEnvelope`) with structured, distinguishable error categories.

### Acceptance criteria
- [x] Given a dummy domain definition implementing `AggregateDefinition<State, Command, Event>`, when `decide` accepts or rejects a command, then the result clearly distinguishes accepted events (including `[]` no-op) from a `DomainRejectionError`.
- [x] Invalid/edge scenario: Error types distinguish `DomainRejectionError`, `InvalidAggregateIdError` / `AggregateNotFoundError`, `RevisionConflictError`, `InvalidRevisionError`, and `StorageFailureError` with diagnostic context (`aggregateId`, `expectedRevision`, `actualRevision`, etc.).
- [x] Event envelope structure captures `aggregateId`, `revision` (positive integer `1..n`), `eventId`, `eventType`, `schemaVersion`, `payload`, and `recordedAt` metadata without coupling `decide`/`evolve` to storage.

### Risks and decisions
- Keep `ExplicitContext` optional or defaulting to empty so simple domains do not require ceremony while supporting deterministic metadata when needed. Used `BTreeMap` for `ExplicitContext::metadata` to guarantee deterministic iteration (`NFR-010`).

### Test plan
- RED test(s): Unit tests constructing each error category, verifying envelope invariants, and testing pure decision results (accepted events, empty no-op list, domain rejection) in `tests/core_contracts.rs`.
- Integration/contract checks: Verify `src/core` has zero imports from domain or adapter modules.
- Full verification command(s): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

### Completion evidence
- Changes: Implemented `AggregateDefinition`, `DomainEvent`, `ExplicitContext`, `PendingEvent`, `EventEnvelope`, and `validate_aggregate_id` in `src/core/contracts.rs`, and `DomainError`, `EngineError`, and `ErrorCategory` in `src/core/errors.rs`. Added contract tests in `tests/core_contracts.rs`.
- Test results (commands, pass/fail): `cargo test` (4 passed, 0 failed), `cargo clippy --all-targets -- -D warnings` (pass), `cargo fmt --check` (pass).
- Review findings/resolution: Verified zero external/domain imports in `src/core`, shared-reference immutability on `decide`/`evolve`, and explicit envelope field validation.
- Passing incremental commits / PR: Branch `feat/TASK-002-core-contracts` (`4e4e7db`), merged to `main`.
- Remaining limitations / follow-up: Pure replay (`TASK-003`), in-memory store (`TASK-004`), and generic runner (`TASK-005`) build on these contracts next.

---

## TASK-003 — Pure `replay` and historical revision reconstruction
Status: Done
Task ID: TASK-003
Requirement IDs: FR-002, FR-003, FR-005, FR-006, FR-007, NFR-001, NFR-002, NFR-010
Dependencies: TASK-002
Owner / files in scope: `src/core/replay.rs`, `tests/core_replay.rs`

### User-visible outcome
The engine can deterministically fold an ordered event stream into an initial state to compute current state or the historical state at any revision `0 <= r <= currentRevision` without mutating inputs or triggering side effects.

### Acceptance criteria
- [x] Given an empty event stream, when `replay` is called, then it returns `initialState` at revision `0` (Invariant 1).
- [x] Given an event stream `history + newEvents`, when `replay` is called on the combined stream, then the resulting state and revision equal folding `evolve` over `newEvents` starting from `replay(history)` (Invariant 2).
- [x] Given a stream of `n` events and a target revision `r` (`0 <= r <= n`), when `replayAtRevision` is called, then it returns the exact state at revision `r` without mutating `initialState`, intermediate states, or the input event list (Invariant 3, 8).
- [x] Invalid/edge scenario: Requesting a negative revision (`r < 0`), a revision beyond committed history (`r > n`), or replaying non-contiguous/out-of-order event revisions raises a typed error (`InvalidRevisionError` / `CorruptedStreamError`).

### Risks and decisions
- Enforce contiguous ascending revisions (`1, 2, ..., n`) and single-aggregate consistency during replay so corrupted streams fail fast (`validate_contiguous_stream`).

### Test plan
- RED test(s): Unit and property/invariant tests for empty stream, prefix replay (`0..n`), incremental fold equivalence, input immutability, and out-of-range/non-contiguous revision errors in `tests/core_replay.rs`.
- Integration/contract checks: Pure function execution with zero I/O.
- Full verification command(s): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

### Completion evidence
- Changes: Implemented `ReplayResult`, `validate_contiguous_stream`, `evolve_all`, `replay`, and `replay_at_revision` in `src/core/replay.rs` and added invariant tests in `tests/core_replay.rs`.
- Test results (commands, pass/fail): `cargo test` (8 passed, 0 failed), `cargo clippy --all-targets -- -D warnings` (pass), `cargo fmt --check` (pass).
- Review findings/resolution: Confirmed input slices and initial states are never mutated and corrupted/non-contiguous streams fail with `EngineError::CorruptedStream`.
- Passing incremental commits / PR: Branch `feat/TASK-003-event-replay` (`122c815`), merged to `main`.
- Remaining limitations / follow-up: Store contract (`TASK-004`) and runner (`TASK-005`) next.

---

## TASK-004 — In-memory event store with atomic expected-version append
Status: Done
Task ID: TASK-004
Requirement IDs: FR-007, FR-008, FR-009, FR-010, FR-014, FR-016, FR-024, NFR-003
Dependencies: TASK-002
Owner / files in scope: `src/core/store.rs`, `src/adapters/mod.rs`, `src/adapters/memory_store.rs`, `tests/store_contract.rs`

### User-visible outcome
An in-memory event store persists ordered event envelopes per aggregate stream, enforces atomic optimistic concurrency (`expectedRevision`), isolates separate aggregate IDs, and exposes a reusable store contract test suite.

### Acceptance criteria
- [x] Given an aggregate at revision `r` (initially `0`), when a batch of `k` events is appended with `expectedRevision = r`, then all `k` events are assigned contiguous revisions `r+1 .. r+k` and committed atomically (Invariant 4).
- [x] Given an aggregate at revision `r`, when an empty event batch (`[]`) is appended with `expectedRevision = r`, then the stream remains at revision `r` with no new events.
- [x] Given two appends against the same aggregate with the same `expectedRevision = r` where both are non-empty, when executed sequentially or concurrently, then the first succeeds and the second fails with `RevisionConflictError` leaving the first batch intact (Invariant 6).
- [x] Given events appended to `aggregateA` and `aggregateB`, when reading either stream or revision, then each aggregate stream is completely isolated (FR-010).
- [x] Invalid/edge scenario: Blank/invalid `aggregateId` is rejected with `InvalidAggregateIdError`; failed appends never leave partial batches in the stream (FR-009).

### Risks and decisions
- Wrote the tests as a reusable `run_event_store_contract_suite(create_store)` in `tests/store_contract.rs` so M4's durable adapter can execute the exact same contract tests.

### Test plan
- RED test(s): Reusable store contract tests covering empty stream (`rev 0`), single/multi-event atomic batch append, `RevisionConflictError` on stale/future `expectedRevision`, multi-threaded concurrent append serialization (`Barrier`), aggregate isolation, and defensive copy history listing in `tests/store_contract.rs`.
- Integration/contract checks: Verify returned event lists are defensive copies so caller mutation cannot corrupt stored history.
- Full verification command(s): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

### Completion evidence
- Changes: Added `EventStore` trait in `src/core/store.rs`, implemented `InMemoryEventStore` with `RwLock<BTreeMap<String, Vec<EventEnvelope<E>>>>` and pre-commit batch validation in `src/adapters/memory_store.rs`, and created `run_event_store_contract_suite` in `tests/store_contract.rs`.
- Test results (commands, pass/fail): `cargo test` (9 passed, 0 failed), `cargo clippy --all-targets -- -D warnings` (pass), `cargo fmt --check` (pass).
- Review findings/resolution: Verified pre-validation of entire batch before mutating stream vector, explicit documentation of volatile in-memory semantics (`FR-014`), and multi-threaded barrier test for `NFR-003`.
- Passing incremental commits / PR: Branch `feat/TASK-004-memory-store` (`f92ea9f`), merged to `main`.
- Remaining limitations / follow-up: Generic command runner (`TASK-005`) next.

---

## TASK-005 — Generic command runner and read-only query service
Status: Done
Task ID: TASK-005
Requirement IDs: FR-004, FR-005, FR-006, FR-008, FR-009, FR-012, FR-013, FR-024, NFR-003, NFR-006
Dependencies: TASK-003, TASK-004
Owner / files in scope: `src/core/runner.rs`, `tests/core_runner.rs`

### User-visible outcome
A generic application runner executes commands (`load -> decide -> append -> evolve`) with optional bounded conflict retries, and provides read-only queries for current state, historical state at revision `r`, and ordered event history.

### Acceptance criteria
- [x] Given a valid command, when `execute` is called on the runner, then it loads current events, reconstructs state, calls `decide`, atomically appends resulting events with `expectedRevision`, and returns the new state, new revision, and committed event envelopes.
- [x] Given an invalid command rejected by `decide`, when `execute` is called, then it returns the `DomainRejectionError` and appends zero events to the store (Invariant 5).
- [x] Given a command that produces an empty event list (`[]`, no-op), when `execute` is called, then the state and revision remain unchanged and no events are written.
- [x] Given a historical query `getStateAtRevision(aggregateId, r)` or `getHistory(aggregateId)`, when executed, then it returns the state at revision `r` (or event list) without modifying stored events or current state (FR-006, Invariant 3).
- [x] Invalid/edge scenario: When a concurrent write causes a `RevisionConflictError` during `execute`, the runner either returns `RevisionConflictError` immediately (default `maxRetries = 0`) or reloads current state and re-runs the entire `decide` function up to `maxRetries` times, never blindly appending stale events (FR-008).

### Risks and decisions
- Ensure the runner has zero domain-specific branches so both Todo and Tic-Tac-Toe plug directly into it.

### Test plan
- RED test(s): Runner tests for accepted commands, multi-event commands, no-op commands, domain rejections, `get_state_at_revision` (`0..current` and out-of-bounds), `get_history`, and conflict retry/exhaustion behavior using a deterministic conflict-injecting store spy in `tests/core_runner.rs`.
- Integration/contract checks: Core + memory store integration.
- Full verification command(s): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

### Completion evidence
- Changes: Implemented `EngineRunner`, `ExecutionOptions`, and `ExecutionResult` in `src/core/runner.rs` and added unit/integration tests in `tests/core_runner.rs`.
- Test results (commands, pass/fail): `cargo test` (12 passed, 0 failed), `cargo clippy --all-targets -- -D warnings` (pass), `cargo fmt --check` (pass).
- Review findings/resolution: Verified `FR-008` reload + re-decide on retry (demonstrated that a command valid at revision 1 is rejected on retry at revision 2 if a concurrent write invalidated its precondition).
- Passing incremental commits / PR: Branch `feat/TASK-005-command-runner` (`582af82`), merged to `main`.
- Remaining limitations / follow-up: Milestone M1 is complete and `src/core/*` is frozen. Next unblocked tasks: `TASK-006` (Todo domain) and `TASK-007` (Tic-Tac-Toe domain).

---

## TASK-006 — Reference domain A: Todo list module
Status: Done
Task ID: TASK-006
Requirement IDs: Milestone M2, FR-001..FR-006, FR-012, FR-026, Section 5 (Todo specification)
Dependencies: TASK-005
Owner / files in scope: `src/domains/mod.rs`, `src/domains/todo.rs`, `tests/todo_domain.rs`

### User-visible outcome
A pure Todo domain module supports `AddTodo`, `CompleteTodo`, and `DeleteTodo` with permanent IDs, deterministic ordering, and time-travel inspection through the generic engine runner.

### Acceptance criteria
- [x] **Scenario: Replay a Todo list** — Given an empty list at revision 0, when `AddTodo("a", "Write tests")`, `AddTodo("b", "Build engine")`, and `CompleteTodo("a")` are accepted, then current revision is 3, Todo `"a"` is completed, Todo `"b"` is open, and state at revision 1 contains only open Todo `"a"`.
- [x] **Scenario: Reject duplicate IDs** — Given `AddTodo("a", "Write tests")` is committed, when `AddTodo("a", "Something else")` is requested, then the command is rejected and no new event is appended.
- [x] **Scenario: Permanent ID after deletion** — Given Todo `"a"` was added and then deleted via `DeleteTodo("a")`, when `AddTodo("a", "Reused ID")` is requested, then the command is rejected because `"a"` remains in `usedIds`, while revision 1 still reconstructs Todo `"a"`.
- [x] **Scenario: Historical read is side-effect-free** — Given a list with 3 events, when state at revision 1 is requested, then its contents match the first event applied to initial state and current revision/history are unchanged.
- [x] Invalid/edge scenarios:
  - `AddTodo` with blank or whitespace-only title or blank ID is rejected.
  - `CompleteTodo` on an already completed Todo succeeds as a no-op (`[]` events, revision unchanged).
  - `CompleteTodo` or `DeleteTodo` on an unknown or already-deleted ID is rejected without emitting events.
  - Todo items are returned in deterministic append order, and state objects are deeply immutable.

### Risks and decisions
- Track `used_ids` (`BTreeSet<String>`) alongside active `todos` (`Vec<TodoItem>`) in the immutable `TodoListState` so deleted IDs cannot be reused after replay and item order is deterministic.

### Test plan
- RED test(s): Unit tests for pure `decide`/`evolve` and runner-integrated acceptance scenarios from `SPEC.md` Section 5 + edge cases (whitespace title, deleted ID reuse, idempotent complete, immutability) in `tests/todo_domain.rs`.
- Integration/contract checks: Verify zero changes to `src/core/*`.
- Full verification command(s): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

### Completion evidence
- Changes: Implemented `TodoDomain`, `TodoListState`, `TodoItem`, `TodoCommand`, and `TodoEvent` in `src/domains/todo.rs` and added acceptance and edge-case tests in `tests/todo_domain.rs`.
- Test results (commands, pass/fail): `cargo test` (16 passed, 0 failed), `cargo clippy --all-targets -- -D warnings` (pass), `cargo fmt --check` (pass).
- Review findings/resolution: Confirmed `git diff main -- src/core/` is empty (`FR-026`) and deleted IDs remain permanently reserved in `used_ids`.
- Passing incremental commits / PR: Branch `feat/TASK-006-todo-domain` (`6253ce1`), merged to `main`.
- Remaining limitations / follow-up: Next unblocked tasks: `TASK-007` (Tic-Tac-Toe domain) and `TASK-008` (CLI demonstrator).

---

## TASK-007 — Reference domain B: Tic-Tac-Toe module
Status: Done
Task ID: TASK-007
Requirement IDs: Milestone M3, FR-001..FR-006, FR-012, FR-026, Section 6 (Tic-Tac-Toe specification)
Dependencies: TASK-005
Owner / files in scope: `src/domains/mod.rs`, `src/domains/tic_tac_toe.rs`, `tests/tic_tac_toe_domain.rs`

### User-visible outcome
A pure 3×3 Tic-Tac-Toe domain module processes `MakeMove(player, cell)` commands, detects all row/column/diagonal wins and draws deterministically, rejects illegal moves, and replays identically across independent engine instances without any changes to the core engine.

### Acceptance criteria
- [x] **Scenario: Deterministic victory** — Given a new game with `X` to move (`cells 0..8` empty, status `in-progress`), when `X` moves at `0`, `O` at `3`, `X` at `1`, `O` at `4`, and `X` at `2`, then `X` is the winner (`status = "X-won"`, `nextPlayer = null`), the game is terminal, and a further move is rejected without an event.
- [x] **Scenario: Reject illegal move** — Given `X` has moved at cell `0`, when `O` attempts cell `0` (occupied), or `X` attempts to move out of turn, or a player attempts cell `-1` or `9` (out of range), then the command is rejected and the revision and board are unchanged.
- [x] **Scenario: All win lines and draw** — Winner detection covers all 3 rows (`0,1,2`, `3,4,5`, `6,7,8`), all 3 columns (`0,3,6`, `1,4,7`, `2,5,8`), both diagonals (`0,4,8`, `2,4,6`), and a full 9-move non-winning board transitions to `status = "draw"` (and rejects subsequent moves).
- [x] **Scenario: Cross-instance replay** — Given two independent engine instances with the same rules and initial state, when both replay the same ordered game events, then both produce equivalent boards, turns, and status, and historical revisions (`0..n`) reconstruct every prior turn accurately.

### Risks and decisions
- Cell addressing is `0..8` in row-major order (`0..2` top row, `3..5` middle row, `6..8` bottom row); game starts at revision `0` with `X` to move and status `in-progress`.
- Core files (`src/core/*`) remained 100% untouched when implementing this task (`git diff main -- src/core/` is empty).

### Test plan
- RED test(s): Table-driven unit tests for all 8 winning lines (for both X and O), draw scenario, out-of-turn/occupied/out-of-bounds/post-terminal rejections, cross-instance replay, and time-travel revision inspection in `tests/tic_tac_toe_domain.rs`.
- Integration/contract checks: `git diff` confirms zero modifications to `src/core/*`.
- Full verification command(s): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

### Completion evidence
- Changes: Implemented `TicTacToeDomain`, `TicTacToeState`, `Player`, `GameStatus`, `TicTacToeCommand`, and `TicTacToeEvent` in `src/domains/tic_tac_toe.rs` and added acceptance and table-driven win/draw/replay tests in `tests/tic_tac_toe_domain.rs`.
- Test results (commands, pass/fail): `cargo test` (20 passed, 0 failed), `cargo clippy --all-targets -- -D warnings` (pass), `cargo fmt --check` (pass).
- Review findings/resolution: Confirmed zero modifications to `src/core/*` (`FR-026`) and deterministic evaluation of all 8 win lines + draw.
- Passing incremental commits / PR: Branch `feat/TASK-007-tic-tac-toe-domain` (`bb88423`), merged to `main`.
- Remaining limitations / follow-up: Next unblocked task: `TASK-008` (CLI demonstrator and comprehensive `README.md`).

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
