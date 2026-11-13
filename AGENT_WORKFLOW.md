# Implementation Agent Playbook — Planning, TDD, Review and Delivery

**Companion to:** `SPEC.md` (PSE-001)  
**Goal:** A repeatable working agreement for any coding agent implementing the specification, independent of programming language and tooling.

## 1. Agent mandate

Implement `SPEC.md` incrementally. Treat that specification as the product contract and this playbook as the execution contract. Prefer simple, correct, comprehensible code over novelty, excessive frameworks or premature distributed systems. Work autonomously within the requested scope, but never hide uncertainty, failed checks or unimplemented requirements.

**Order of precedence:** explicit human instruction > approved specification/change decisions > this playbook > agent preferences.

## 2. End-to-end workflow (repeat for every feature)

```text
Understand specification
      ↓
Plan milestone and risks
      ↓
Refine into small independent tasks
      ↓
Select an unblocked task
      ↓
Define testable acceptance criteria
      ↓
RED: write a failing test
      ↓
GREEN: minimal implementation
      ↓
REFACTOR: improve design, stay green
      ↓
Review against spec + maintainability + security
      ↓
Run targeted tests, then full relevant checks
      ↓
Document decision / status; commit if appropriate
      ↓
Select next unblocked task
```

This is **Scrum-inspired iterative planning**, not a claim that Scrum rituals or timeboxed sprints are mandatory for a solo agent.

## 2A. Execution principles and checkpoints

Use **specification-driven, risk-first, incremental delivery** rather than rigid sprint ceremonies. The objective is a continuously working product, not maximum task throughput.

- **Specification is the source of intent, tests are executable evidence, and running software is the final proof.** None alone substitutes for the others.
- Prefer the **smallest vertical slice** that exercises the real path end-to-end before generalizing. Avoid spending an entire milestone creating unused interfaces.
- Keep the project in a **working, reproducible state after each integrated task**. Prefer short-lived branches and continuous integration over large, deferred merges.
- Explicitly distinguish **verified**, **assumed**, **blocked**, and **out of scope**. Never infer success from code that merely looks correct.
- When changes affect externally persisted event formats, treat them as compatibility-sensitive changes: version schemas, test old-event replay, and document migration or upcasting.
- For implementation choices with meaningful tradeoffs, write a short Architecture Decision Record (ADR): context, alternatives, decision, consequences. Do not document trivial choices as ADRs.
- Start with an unoptimized, correct reference replay implementation. Compare any optimized/snapshot path against it in tests.
- Keep a tiny executable demonstration working from the earliest useful milestone; it is the integration anchor.

### Stop / escalation rules

Request a decision only when an ambiguity changes externally visible behavior, security, correctness guarantees, or irreversible data design and no specification default resolves it. Otherwise choose the simplest reversible option, document the assumption, and proceed. Never silently weaken invariants, alter acceptance criteria, or claim a partial task is finished.

## 3. Planning and backlog refinement

Before changing code:

1. Read the whole `SPEC.md`, current code, tests, README, and any existing decisions.
2. Build a requirement traceability matrix: requirement ID -> task -> test -> status.
3. Identify the smallest vertical slice that delivers demonstrable behavior; start with M0/M1 contracts and a working replay test, not a complex general-purpose framework.
4. Break work down into small tasks. Each task SHOULD take one focused change, preferably a narrow reviewable diff; avoid mixing domains, persistence and UI in one task.
5. For each task record: ID, outcome, requirement IDs, dependencies, acceptance tests, known risks, estimated relative size, and status.
6. Refine further if a task lacks an unambiguous observable result, exceeds a single coherent change, has unclear dependencies, or combines independent concerns.
7. Mark tasks **Ready** only if they satisfy the Definition of Ready below.
8. Prioritize architectural risks early: deterministic replay, atomic append, version conflicts, and avoiding effects during replay.

**Definition of Ready (DoR):**

- Clear user-observable outcome and acceptance criteria.
- Exact relevant specification IDs are linked.
- Preconditions, dependencies and interfaces are known.
- Examples or tests illustrate expected behavior including failure cases.
- A verification method exists.
- Remaining assumptions are minor and recorded.

If material ambiguity blocks correctness, record it as a decision needed. Otherwise choose the smallest reasonable interpretation, document it, and continue without speculative features.

## 4. Task management and parallel work

Maintain a lightweight task board in `TASKS.md`:

`Backlog → Refined → Ready → In Progress → Review → Verified → Done`

- Respect dependencies. Never start a blocked task simply to keep busy.
- Parallel work is allowed **only** for tasks with independent files/contracts, reproducible test setup, and no shared mutable ownership.
- Stabilize interfaces/contracts before parallelizing implementations against them.
- Good parallel candidates after M1: Todo domain, Tic-Tac-Toe domain, durable store integration tests, documentation.
- Do not parallelize two changes to the same core interface, migrations, or shared build configuration without a coordination/merge plan.
- Give each agent/subtask a narrow scope, ownership, acceptance tests and integration boundary.
- Integrate frequently; after merging, rerun all affected tests and cross-domain acceptance tests.
- Do not infer parallel execution from the mere existence of a task list; it requires available tooling and independent workstreams.

## 5. Test-driven development (TDD)

Every behavior change SHOULD follow **Red → Green → Refactor**:

**RED**
- Write a specific test that demonstrates a missing behavior or regression and watch it fail for the expected reason.
- Test observable behavior, not implementation details or private methods.

**GREEN**
- Implement the minimum production code needed to satisfy the test.
- Do not add abstractions until a current test or explicit specification requires them.

**REFACTOR**
- Remove duplication, clarify names, simplify control flow, and separate pure domain code from adapters.
- Run tests after refactoring. Do not change behavior under the label of refactoring.

### Mandatory test categories

- **Pure unit tests:** `decide`, `evolve`, initial state, replay and validation for every reference domain.
- **Property/invariant tests (where practical):** deterministic replay; incremental fold equivalence; state immutability; snapshots equal full replay; revision monotonicity.
- **Store contract tests:** append ordering, atomic batch, expected-revision conflicts, aggregate isolation and no partial commits. Run the same contract test suite against every store adapter where feasible.
- **Integration tests:** real database behavior for durable adapter, schema migration and restart recovery.
- **Failure tests:** invalid commands, duplicate IDs/moves, simultaneous writes, interrupted operations, unknown event schema versions.
- **End-to-end demonstration:** execute commands, inspect history, time-travel, confirm no effects; for both Todo and Tic-Tac-Toe.

Prefer table-driven tests for examples and property-based tests for invariants. Add mutation or fault-injection testing when it usefully checks test strength; do not mandate tools without value. A test that fails for an unrelated setup error is not useful RED evidence.

Never weaken assertions or delete failing tests simply to obtain a green build. If a test is outdated due to an approved requirement change, update it together with the documented specification change.

## 6. Clean code and architecture rules

### Naming and functions

- Use small, single-purpose functions with names expressing intent (e.g., `isMoveLegal`, `appendIfVersionMatches`, `replayFromSnapshot`). No arbitrary line limit: split when one function handles multiple concerns.
- Prefer explicit domain terminology over generic `process`, `handleThing`, `data`, `utils`, or unclear abbreviations.
- Favor guard clauses, simple control flow, exhaustive handling of commands/events, and clear error types.
- Avoid Boolean-argument ambiguity in APIs; use domain-specific commands or named options.
- Minimize comments. Code should explain *what*; comments are reserved for non-obvious *why*, safety invariants, protocol contracts and architectural tradeoffs. Avoid duplicating code in comments.

### State and side effects

- Favor immutable domain values and pure `decide/evolve/replay` functions.
- Never read wall-clock time or generate random IDs inside pure transitions; inject input explicitly.
- Keep persistence, logging, networking, threads, telemetry and external calls at adapters/boundaries.
- Never dispatch device/network effects during replay or historical queries.
- Use typed/recoverable errors rather than silently swallowing failures.
- Do not introduce a generic abstraction unless it serves the current two domain implementations or an explicit requirement.

### Design quality

- Keep modules cohesive and public APIs minimal. Prefer concrete, locally understandable code over unnecessary interface indirection.
- Use names that express intent and domain meaning. Short functions are a consequence of single responsibility, **not** a fixed line-count target.
- Comments are justified for non-obvious why, invariants, concurrency decisions and interoperability; avoid comments that paraphrase the code.
- Measure complexity through readability, coupling and change impact, not raw comment counts or lines of code.

- Prefer composition and clear interfaces over deep inheritance.
- Avoid speculative plugin systems, service locators, unnecessary factories, reflection-heavy wiring and "clever" meta-programming.
- Keep event data versioned, stable and independently inspectable; do not serialize executable functions.
- Keep persistence implementation and schema details out of domain rules.
- Measure before optimizing. Add snapshots only after correct unsnapshotted replay exists.
- Review data races, authorization, input validation, SQL safety and secret handling when relevant.

## 7. Review and quality gates

A task can move from **In Progress** to **Review** only after local targeted tests pass.

During review, verify:

1. Meets every linked requirement and acceptance scenario, including negative cases.
2. Smallest sensible implementation; no unnecessary dependencies or feature creep.
3. Function/module naming, cohesion, readability, immutable data, and separation of side effects.
4. State/event schema compatibility and replay invariants are preserved.
5. Persistence atomicity, optimistic concurrency, idempotency and failure modes are addressed where relevant.
6. Tests cover behavior and adversarial/edge cases rather than merely happy paths.
7. Documentation and examples match actual behavior.
8. No debug artifacts, temporary hacks, unreviewed generated code, secrets or unrelated changes remain.

When feasible, use a reviewer separate from the implementing agent or at minimum a deliberate second-pass review against the acceptance criteria. Reviewers should challenge failure paths, concurrency assumptions and whether each abstraction is needed.

After review, run the **full relevant check suite again**, including compilation/typechecking, lint/format, unit tests, integration tests where applicable, and example smoke tests. A review fix invalidates prior verification and requires rerunning affected checks.

If tools cannot run, report **Not verified** with the exact blocker; do not call the task done.

## 8. Definition of Done (DoD)

A task is **Done** only when all conditions hold:

- Requirement IDs and acceptance criteria satisfied.
- TDD evidence or a documented reason TDD was impractical; tests pass.
- Review performed and findings resolved.
- Relevant full checks rerun after the last code change.
- Documentation, migration/compatibility notes and decision log updated if needed.
- Task board (`TASKS.md`) and requirement traceability updated with issue status, test results, and relevant commits.
- Diff is focused and ready to integrate; if Git is available and the project workflow allows it, create a descriptive commit. Never force push or overwrite unrelated work.
- Any remaining limitations are explicit and separately tracked, not concealed.

After finishing a task, select the next **Ready, unblocked** task; prioritize a dependency that unlocks multiple tasks. Stop only at the requested milestone or a real unresolved blocker.

## 9. Git and change hygiene

- **Commit frequently after each meaningful, working increment**; multiple atomic commits per task are encouraged. Avoid one giant commit at task completion.
- Every commit must build and pass relevant fast tests; run affected integration tests when applicable. TDD RED changes stay uncommitted until GREEN.
- Use descriptive imperative messages such as `feat(store): add optimistic version checks`; include issue IDs when practical.
- Work on a feature branch when the repository workflow provides one; avoid editing shared branches unexpectedly.
- Review `git diff` and `git status` before committing. Do not commit generated caches, secrets or temporary database volumes.
- Preserve existing user changes; do not reset or clean working files without explicit authorization.
- For parallel branches, document interface versions and merge order. Resolve conflicts deliberately and rerun tests after integration.
- Do not claim a commit or deployment happened unless the command actually succeeded.

## 10. Progress communication

At the start of a milestone provide:

```text
Goal:
Deliverable:
Requirements:
Small tasks and dependencies:
Testing approach:
Risks / assumptions:
```

For each completed task provide a compact report:

```text
Task / requirement IDs:
Changed behavior:
Tests added (RED/GREEN evidence):
Review findings and fixes:
Verification commands + results:
Known limitations:
Commit/reference (if any):
Next unblocked task:
```

Reports should state facts; avoid empty status narration. Surface a blocker immediately if it impacts correctness, safety, irreversible actions or the milestone's scope.

## 11. Recommended starter backlog

The following is a **starting refinement input**, not an immutable task list. Split further as necessary.

| Task | Focus | Depends on | Proof |
|---|---|---|---|
| T-001 | Project skeleton, build/test command | — | Clean setup passes |
| T-002 | Core typed contracts and errors | T-001 | Compile + contract unit tests |
| T-003 | Pure `replay` and historical revisions | T-002 | Replay property tests |
| T-004 | Memory store with atomic expected-version append | T-002 | Store contract tests |
| T-005 | Generic command runner | T-003, T-004 | Runner behavior/error tests |
| T-006 | Todo commands/events/rules | T-005 | Todo scenarios pass |
| T-007 | Tic-Tac-Toe commands/events/rules | T-005 | Game scenarios pass; core untouched |
| T-008 | CLI/history/time-travel demo | T-006 | Smoke test including read-only replay |
| T-009 | Durable adapter + migration/restart tests | T-004 | Real-store contract tests |
| T-010 | Snapshots and compatibility | T-009 | Snapshot/full-replay equivalence |
| T-011 | Optional HTTP/UI | T-006 | End-to-end history/time-travel demo |
| T-012 | Optional outbox + virtual lamp | T-009 | Failure/retry/idempotency tests |
| T-013 | Optional P2P game extension | T-007 | Authenticated/replayed moves, fork policy |

**Parallelization example:** Once T-005 is stable, T-006 and T-007 can run independently. T-009 can run alongside them once T-004's store contract is frozen. Do not start T-010 before the durable adapter works.

## 12. First instructions to an implementing agent

> Read `SPEC.md` and `AGENT_WORKFLOW.md` in full. Build a requirement-to-test map and refine milestones M0–M3 into small, dependency-aware tasks. Begin with the smallest TDD slice. Implement the language-independent contracts in a language you choose and document why, using only needed dependencies. Deliver working Todo and Tic-Tac-Toe examples on the same unchanged core, with replay and time travel. Do not implement PostgreSQL, snapshots, UI, networking or P2P until the baseline is complete unless explicitly requested. Run review and tests after every task, report evidence and limitations, and commit focused changes when permitted by the repository workflow.
