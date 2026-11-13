# Portable State Engine

Language-independent specification for an event-driven state engine with Todo and Tic-Tac-Toe as reference domains.

- [`SPEC.md`](SPEC.md) is the **product contract**: requirements, invariants and acceptance criteria.
- [`AGENTS.md`](AGENTS.md) adds project-specific instructions to the global agent rules.
- [`AGENT_WORKFLOW.md`](AGENT_WORKFLOW.md) contains project-specific execution and verification details.
- [`TASK_TEMPLATE.md`](TASK_TEMPLATE.md) is the task template for [`TASKS.md`](TASKS.md).
- [`docs/adr/0001-language-and-toolchain.md`](docs/adr/0001-language-and-toolchain.md) records the Rust 1.85 toolchain and zero-dependency core decision.

## Build and test

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Starting an implementation

Install the reusable documents from `../global-agent-rules/` in your agent's global instruction location, following its supported mechanism. Keep those rules separate from the project's specification. Open this repository and ask your coding agent to read `AGENTS.md` and `SPEC.md`, plan the earliest milestone, create small dependency-aware issues in `TASKS.md`, and then implement verified slices with TDD, review and frequent working commits. Use separate Git worktrees for independent concurrent coding agents.

No particular programming language, framework, data store or agent is required by the core specification.
