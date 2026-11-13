# Task template

Copy the block below into `TASKS.md` for each refined task. Keep each task small enough to implement, review, and verify as one coherent change.

```markdown
## TASK-XXX — <outcome-oriented title>
Status: Backlog | Refined | Ready | In Progress | Review | Verified | Done | Blocked
Linear issue / Local ID: <identifier>
Requirement IDs: <SPEC references>
Dependencies: <task IDs or None>
Owner / files in scope: <modules/files>

### User-visible outcome
<One observable behavior.>

### Acceptance criteria
- [ ] Given <context>, when <action>, then <result>.
- [ ] Invalid/edge scenario: ...

### Risks and decisions
<Relevant uncertainty; ADR link if needed.>

### Test plan
- RED test(s): ...
- Integration/contract checks: ...
- Full verification command(s): ...

### Completion evidence
- Changes: ...
- Test results (commands, pass/fail): ...
- Review findings/resolution: ...
- Passing incremental commits / PR: ...
- Linear issue link and status (if configured): ...
- Remaining limitations / follow-up: ...
```

**Definition of Ready:** clear acceptance criteria, dependencies, interfaces, test plan, and no unresolved correctness-critical ambiguity.

**Definition of Done:** implemented, reviewed, all relevant checks re-run successfully after the last edit, spec and task status updated, and limitations explicit.
