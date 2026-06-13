---
id: FEAT-3bc87e1a
title: Fix Loop Step Tracking and Tool Result Message Write-Back
status: approved
---

## Overview

Focused correctness fix for the Rust engine's main loop.

## Technical Approach

1. **Loop step increment**
   - In `EffectExecutor::CallLLM`, when `StreamEnd` is received, emit an `UpdateComponent` effect to increment `LoopComponent.step`.
   - Keep `InputSystem` and `OutputSystem` reading the current step from `LoopComponent` as they do today; the increment happens at the natural boundary after the LLM response completes.

2. **Tool result write-back**
   - In `EffectExecutor::ExecuteTool`, after emitting `ToolEvent::Result`, append a `Message::Tool { content: result, tool_call_id }` to `MessagesComponent`.
   - On tool error, append a `Message::Tool { content: error, tool_call_id }` so the LLM can see the failure.

3. **Testing**
   - Extend the existing CLI integration test with a mock provider that also simulates a tool call, verifying both step increment and tool message append.
   - Add a focused unit test for `EffectExecutor::ExecuteTool` directly.

## Architecture

No new files. Changes are localized to:
- `crates/alice-core/src/effect_executor.rs`
- `crates/alice/tests/cli_loop_test.rs`
- New test file: `crates/alice-core/tests/effect_executor_test.rs`

## Constraints

- Zero warnings.
- No behavioral changes to existing passing tests.
