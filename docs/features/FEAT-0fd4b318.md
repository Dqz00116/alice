---
id: FEAT-0fd4b318
title: Complete Remaining Rust Engine Integration and Cleanup
status: approved
---

## Overview

Finish the remaining integration points and cleanup identified in the Rust engine audit.

## Technical Approach

1. **Middleware wiring**
   - Create a `MiddlewarePipeline` in `main.rs`.
   - Pass every event through `pipeline.run(event)` before looking up systems.
   - Add an integration test that registers a transforming middleware and asserts the transformed event reaches the systems.

2. **Hook lifecycle completion**
   - `InputSystem` emits a `HookTrigger { hook: "beforeStep" }` effect after appending the user message.
   - `EffectExecutor` increments `LoopComponent.step` before emitting the `afterStep` hook trigger.
   - `HookSystem::shouldContinue` emits an `Abort` effect when `should_continue` is false, cleanly terminating the loop.

3. **Anthropic SSE parser test**
   - Add a unit test in `crates/alice-providers/tests/` that feeds a canned SSE payload to a mock response stream and asserts the yielded `LLMStreamEvent`s.

4. **ProviderComponent cleanup**
   - Remove `ProviderComponent` from `components.rs` and all component bundles.
   - Move `api_key` into `ConfigComponent`.
   - Update `EffectExecutor` bounds and `ComponentAccessor` impl to no longer require `ProviderComponent`.
   - Update CLI to read the API key from `ConfigComponent`.

## Architecture

No new crates. Changes are localized to:
- `crates/alice/src/main.rs`
- `crates/alice/tests/cli_loop_test.rs`
- `crates/alice-core/src/components.rs`
- `crates/alice-core/src/effect_executor.rs`
- `crates/alice-core/src/systems/input.rs`
- `crates/alice-core/src/systems/hook.rs`
- `crates/alice-core/tests/integration.rs`
- `crates/alice-providers/tests/anthropic_test.rs`

## Constraints

- Zero warnings.
- Existing tests must continue to pass.
- Keep systems pure functions.
