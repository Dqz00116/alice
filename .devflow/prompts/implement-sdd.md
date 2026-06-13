## Stage 3: Implementation with Parallel SDD

Implement feature using **Parallel Subagent-Driven Development**.

### Input:
- PLAN-{workflow_run_id}.md
- DESIGN-{workflow_run_id}.md
- REQ-{workflow_run_id}.md

### Skills Required:
- `subagent-driven-development`
- `dispatching-parallel-agents`
- `test-driven-development` (used by subagents)

### Preparation

1. Read PLAN, DESIGN and REQ in full.
2. Extract every task from PLAN:
   - Task ID
   - Full task description
   - Affected files / modules
   - Dependencies on other tasks
3. Build a dependency graph.
4. Group tasks into **waves**:
   - A wave contains only tasks with no unresolved dependencies.
   - Tasks in the same wave must touch **disjoint file sets**.
   - If two tasks would edit the same file, put them in **different waves**.

### Per-Wave Execution

Repeat for each wave until all waves are complete.

#### 1. Parallel Implementation

Dispatch one implementer subagent per task **in parallel**.

Each implementer receives:
- Task ID and full task text
- PLAN and DESIGN context relevant to this task
- List of files it may modify
- Explicit constraint: **DO NOT modify files assigned to other tasks**

Subagent uses `test-driven-development` skill:

```
RED: Write failing test
  ↓
GREEN: Write minimal code to pass
  ↓
REFACTOR: Improve while keeping tests green
  ↓
Repeat
```

Collect each subagent's status:
- **DONE** → proceed to review
- **DONE_WITH_CONCERNS** → read concerns, decide if review is enough
- **NEEDS_CONTEXT** → provide context and re-dispatch
- **BLOCKED** → resolve blocker or escalate to human

#### 2. Parallel Spec Review

After all implementers in the wave report DONE/DONE_WITH_CONCERNS, dispatch one spec reviewer subagent per completed task **in parallel**.

Reviewer checks:
- All requirements from task are implemented
- Nothing extra is added
- Tests cover the spec

If any reviewer reports issues:
- Re-dispatch the corresponding implementer with the feedback
- Re-run spec review for that task
- Do NOT start code quality review for a task until its spec review passes

#### 3. Parallel Code Quality Review

After all spec reviews in the wave pass, dispatch one code quality reviewer subagent per task **in parallel**.

Reviewer checks:
- Code style matches project conventions
- No duplication or unnecessary complexity
- Tests are meaningful and maintainable

If any reviewer reports issues:
- Re-dispatch the corresponding implementer with the feedback
- Re-run code quality review for that task

#### 4. Mark Wave Complete

Only mark the wave complete when:
- Every task in the wave passed spec review
- Every task in the wave passed code quality review
- All tests introduced in this wave pass

### Cross-Wave Rules

- A task in wave N may only start after all tasks it depends on have passed review.
- If a later wave discovers that an earlier wave's implementation is insufficient, reopen the earlier task and re-run its full SDD cycle.
- Keep total work-in-progress bounded: do not dispatch more than one wave at a time.

### Integration

After all waves complete:

1. Run the full test command: `{test_command}`
2. Fix any integration issues that did not appear in per-task tests.
3. Dispatch a final code reviewer over the entire implementation if the plan touched more than 3 files or multiple subsystems.

### Iron Law:

**NO production code without failing test first.**
If violated: DELETE code, restart.

### Gate:
- All tests pass: `{test_command}`
