# Portable State Engine (`PSE-001`)

A deterministic, domain-independent event-driven state engine implemented in Rust (Edition 2021, zero external crate dependencies for the M0–M3 baseline). The same generic engine core and runner execute both a time-travel **Todo list** domain and a turn-based **3×3 Tic-Tac-Toe** domain without any domain-specific branches in the engine core.

- [`SPEC.md`](SPEC.md) is the **product contract**: requirements, invariants, and acceptance criteria.
- [`AGENTS.md`](AGENTS.md) and [`AGENT_WORKFLOW.md`](AGENT_WORKFLOW.md) define the engineering and verification workflow.
- [`TASKS.md`](TASKS.md) tracks tasks, requirement-to-test traceability, and verification evidence.
- [`docs/adr/0001-language-and-toolchain.md`](docs/adr/0001-language-and-toolchain.md) records the Rust 1.85 toolchain and zero-dependency core decision.

---

## Quickstart

### 1. Build, Test, Lint, and Format

Run the full verification suite with a single command:

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

Or run the test suite directly:

```bash
cargo test
```

### 2. Executable CLI Demonstrator (`FR-024`, `FR-025`, `FR-026`)

Run the built-in end-to-end demonstration (executes commands, rejects invalid commands without appending events, prints current state, lists ordered event history, and performs side-effect-free historical time-travel queries for both Todo and Tic-Tac-Toe):

```bash
cargo run -- demo
```

Run scripted Todo list workflows:

```bash
cargo run -- todo "add:a:Write tests" "add:b:Build engine" "complete:a" state history "state-at:1"
```

Run scripted Tic-Tac-Toe workflows (cells addressed `0..8` in row-major order):

```bash
cargo run -- tic-tac-toe move:X:0 move:O:3 move:X:1 move:O:4 move:X:2 state history state-at:2
```

---

## Architecture and Boundaries (`SPEC.md` Section 3)

```text
src/
├── core/
│   ├── contracts.rs   # AggregateDefinition, DomainEvent, ExplicitContext, PendingEvent, EventEnvelope
│   ├── errors.rs      # DomainError, EngineError, ErrorCategory (FR-004, FR-013)
│   ├── replay.rs      # Pure replay(), replay_at_revision(), evolve_all(), stream contiguity check
│   ├── store.rs       # Replaceable EventStore<E> trait boundary (FR-007..FR-011)
│   └── runner.rs      # Generic EngineRunner<D, S> orchestrating load -> decide -> append -> evolve
├── adapters/
│   └── memory_store.rs # Thread-safe InMemoryEventStore<E> with atomic expectedRevision append (FR-014)
├── domains/
│   ├── todo.rs        # Reference Domain A: Todo list with permanent IDs (Section 5)
│   └── tic_tac_toe.rs # Reference Domain B: 3x3 Tic-Tac-Toe (Section 6)
├── cli/
│   └── mod.rs         # CLI demonstrator for Todo and Tic-Tac-Toe (FR-025)
└── main.rs            # Binary entrypoint
```

- **Strict separation (`FR-001`, `FR-011`, `FR-026`):** `src/core/` has zero imports from `src/domains/` or `src/adapters/`. Both `TodoDomain` and `TicTacToeDomain` plug directly into `EngineRunner<D, S>` without modifying a single line of `src/core/`.
- **Compile-time immutability (`FR-003`, Invariant 8):** `decide(&self, state: &Self::State, command: &Self::Command, context: &ExplicitContext)` and `evolve(&self, state: &Self::State, event: &Self::Event)` take shared immutable references, preventing in-place mutation of inputs.

---

## Enforced Invariants (`SPEC.md` Section 9)

1. **`replay(emptyHistory) == initialState` at revision `0`** (`tests/core_replay.rs`).
2. **Incremental fold equivalence:** `replay(history + events) == fold(evolve, replay(history), events)` (`tests/core_replay.rs`).
3. **Side-effect-free historical reads:** `get_state_at_revision(id, r)` (`0 <= r <= current`) reconstructs historical state without appending events, mutating state, or dispatching external effects (`tests/core_runner.rs`, `tests/todo_domain.rs`, `tests/tic_tac_toe_domain.rs`).
4. **Contiguous atomic batches:** All accepted events from a command are assigned contiguous revisions `r+1 .. r+k` and committed atomically or not at all (`tests/store_contract.rs`).
5. **Zero events on rejection:** Commands rejected by `decide` return `EngineError::DomainRejection` (`ErrorCategory::ValidationRejection`) and append zero events (`tests/core_runner.rs`).
6. **Optimistic concurrency (`FR-008`, `NFR-003`):** Concurrent or stale appends with the same `expected_revision` serialize per stream; conflicting writes fail with `EngineError::RevisionConflict`. When bounded retries (`max_retries > 0`) are configured on `EngineRunner`, the runner reloads the latest state and re-runs `decide` from scratch rather than blindly appending stale events (`tests/store_contract.rs`, `tests/core_runner.rs`).

---

## Storage Tradeoffs and Deferred Milestones (`NFR-008`, `SPEC.md` Section 10)

- **In-memory store (`InMemoryEventStore`, `FR-014`):** Uses `std::sync::RwLock<BTreeMap<String, Vec<EventEnvelope<E>>>>` to provide deterministic ordering, aggregate isolation, and serialized atomic `expected_revision` checks for quickstart and testing. **Volatile storage caveat:** All event streams reside in process memory and are lost when the process exits or restarts.
- **Deferred follow-on milestones (`M4–M8`):**
  - **M4 (`TASK-009`):** Durable event-store adapter (e.g., PostgreSQL/SQLite) verified against the shared `run_event_store_contract_suite` plus crash/restart recovery tests.
  - **M5 (`TASK-010`):** Optional snapshots (`FR-017`, `FR-018`) and event schema version upcasting (`FR-019`).
  - **M6–M8 (`TASK-011`..`TASK-013`):** Optional HTTP/UI time-travel viewer, transactional outbox/virtual lamp domain, and peer-to-peer verifiable game extension.
