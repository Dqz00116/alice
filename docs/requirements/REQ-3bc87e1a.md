---
id: REQ-3bc87e1a
title: Fix Loop Step Tracking and Tool Result Message Write-Back
status: approved
priority: high
---

## Description

The Rust engine now runs a complete stdin-driven conversation loop, but two core correctness issues remain:

1. `LoopComponent.step` is read by `InputSystem`, `OutputSystem`, and `HookSystem` but is never incremented. As a result, `max_steps` termination never triggers and `StepStart`/`StepEnd` always report step 0.
2. When a tool is executed, its result is emitted as a `ToolEvent::Result` event but is never appended to `MessagesComponent` as a `Message::Tool`. The LLM therefore cannot see tool output in subsequent turns.

This requirement covers a focused fix for both issues, plus regression tests.

## Acceptance Criteria

- [ ] `LoopComponent.step` is incremented after each completed LLM stream (`StreamEnd`).
- [ ] `InputSystem` emits `StepStart` with the current step before increment.
- [ ] `OutputSystem` emits `StepEnd` with the current step before increment.
- [ ] `HookSystem::afterStep` uses the updated step value when computing `should_continue`.
- [ ] `max_steps` correctly terminates the loop when `step >= max_steps`.
- [ ] `EffectExecutor::ExecuteTool` appends a `Message::Tool` containing the tool result to `MessagesComponent`.
- [ ] Tool errors are handled gracefully (e.g., append a tool error message or emit a clear error event).
- [ ] New tests verify step increment and tool result write-back.
- [ ] `cargo test --all-targets`, `cargo clippy --all-targets`, and `cargo build --release` all pass with zero warnings.

## Notes

- Keep changes minimal and focused. Do not refactor unrelated systems.
- Maintain the pure-function `System` + declarative `Effect` design.
- The fix should not break the existing CLI integration test.
