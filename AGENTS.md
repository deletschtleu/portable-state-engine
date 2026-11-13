# Portable State Engine — project agent instructions

First follow the globally installed `AGENTS.md`, `AGENT_WORKFLOW.md`, and `SANDBOX.md` where available. Then read this file, `SPEC.md`, and the project `AGENT_WORKFLOW.md`. Project requirements in `SPEC.md` are authoritative.

- Keep `decide`/`evolve` pure and deterministic; never execute external effects during replay.
- The same generic engine must support both Todo and Tic-Tac-Toe without domain-specific changes.
- Preserve immutable domain state, ordered events, and correct expected-version checks when adding persistence.
- Begin with the smallest in-memory vertical slice. PostgreSQL, snapshots, UI and networking are optional later milestones unless specified otherwise.
- Track tasks in a checked-in `TASKS.md` using `TASK_TEMPLATE.md`. Do not maintain two conflicting backlogs.
- Follow global Git worktree and incremental verified commit policies; link issue, branch, PR, test evidence, and review results before marking Done.
- Document architectural decisions that affect contracts, correctness, or portability.
