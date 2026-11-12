# Portable State Engine

Language-independent specification for an event-driven state engine with Todo and Tic-Tac-Toe as reference domains.

- [`SPEC.md`](SPEC.md) is the **product contract**: requirements, invariants and acceptance criteria.
- [`AGENTS.md`](AGENTS.md) adds project-specific instructions to the global agent rules.
- [`AGENT_WORKFLOW.md`](AGENT_WORKFLOW.md) contains project-specific execution and verification details.
- [`TASK_TEMPLATE.md`](TASK_TEMPLATE.md) is a fallback issue template when Linear is not configured.

## Starting an implementation

Install the reusable documents from `../global-agent-rules/` in your agent's global instruction location, following its supported mechanism. Keep those rules separate from the project's specification. Open this repository and ask your coding agent to read `AGENTS.md` and `SPEC.md`, plan the earliest milestone, create small dependency-aware issues in Linear (or a local `TASKS.md` fallback), and then implement verified slices with TDD, review and frequent working commits. Use separate Git worktrees for independent concurrent coding agents.

No particular programming language, framework, data store or agent is required by the core specification.
