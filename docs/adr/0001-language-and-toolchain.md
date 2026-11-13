# ADR-0001: Programming Language, Toolchain, and Zero-Dependency Core

- **Status:** Accepted
- **Date:** 2026-11-13
- **Requirement IDs:** `SPEC.md` Section 10 (M0), Section 11, Section 12, `FR-001`, `FR-003`, `FR-011`, `NFR-001`, `NFR-002`, `NFR-007`, `NFR-010`

## Context

`SPEC.md` (PSE-001) defines a language-independent event-driven state engine that models deterministic domain behavior via pure `decide` and `evolve` functions, immutable state, ordered event streams, optimistic revision checks (`expectedRevision`), and read-only historical time travel across at least two domains (Todo and Tic-Tac-Toe) without altering the engine core.

Section 12 requires choosing simple defaults, recording tradeoffs in an Architecture Decision Record, and avoiding unnecessary dependencies.

## Alternatives Considered

1. **Rust (Edition 2021, `rustc` / `cargo` 1.85.1 via Debian `trixie` packages)**
   - *Pros:*
     - Algebraic data types (`enum` commands, events, and errors) with compiler-enforced exhaustive pattern matching (`match`).
     - Ownership and shared references (`&State`, `&Command`, `&Event`) enforce state and event immutability (`FR-003`, Invariant 8) at compile time without defensive deep-cloning overhead or runtime freeze hacks.
     - Generic traits (`AggregateDefinition`) verify domain separation (`FR-001`, `FR-012`, `FR-026`) with zero runtime reflection.
     - Built-in test runner (`cargo test`), linter (`cargo clippy`), and formatter (`rustfmt`) require **zero external crates** for the M0–M3 baseline (`std` only), guaranteeing offline reproducibility inside the sandbox.
     - Standard library `std::sync::Mutex` / `RwLock` provides explicit per-stream serialization and atomic `expectedRevision` checks for the in-memory store (`FR-008`, `NFR-003`).
   - *Cons:*
     - Strict borrow-checker rules require explicit ownership modeling compared to garbage-collected scripting languages.

2. **TypeScript on Node.js**
   - *Pros:* Discriminated unions, easy JSON/HTTP integration for optional M6 UI.
   - *Cons:* Requires runtime discipline (`Object.freeze`) to enforce deep immutability; user preferred not to use Node.js.

3. **Python 3 (`dataclasses(frozen=True)`)**
   - *Pros:* Concise syntax, fast prototyping.
   - *Cons:* Static typing and exhaustiveness checks are external (`mypy`/`pyright`) rather than enforced by the compiler.

4. **Go**
   - *Pros:* Fast compilation and simple concurrency.
   - *Cons:* Lacks native sum types/enums and compiler-enforced immutability for slices/maps.

## Decision

We select **Rust (1.85.1, 2021 edition)** with **zero third-party dependencies** for the M0–M3 baseline:

- **Package & build tool:** `cargo` (`Cargo.toml` at repository root).
- **Single verification command:** `cargo test` (supplemented by `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`).
- **Executable CLI demo command:** `cargo run -- <args>` (or `cargo run -- demo`).
- **Module layout:**
  - `src/core/`: Domain-independent traits, envelope/error types, pure `replay`, `EventStore` trait, and generic `EngineRunner`. Must never import from `src/domains/`.
  - `src/adapters/`: Replaceable infrastructure adapters (`memory_store` in M1).
  - `src/domains/`: Independent reference domain modules (`todo` in M2, `tic_tac_toe` in M3).
  - `src/bin/` / `src/main.rs`: CLI demonstrator (M2/M3).

## Consequences

- **Positive:** Compile-time guarantees for immutability (`&State`), exhaustive command/event handling, typed error categories (`Result<T, EngineError>`), and 100% offline builds with zero external crate downloads for M0–M3.
- **Negative / Tradeoffs:** When M4 (PostgreSQL/SQLite durable store) or M8 (cryptographic P2P envelopes) are implemented later, explicit serializer/database adapter crates will be introduced at the adapter boundary while keeping `src/core` free of I/O dependencies.
