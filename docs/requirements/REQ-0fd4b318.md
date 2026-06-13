---
id: REQ-0fd4b318
title: Complete Remaining Rust Engine Integration and Cleanup
status: approved
priority: high
---

## Description

The Rust engine now has a working core loop, SSE parsing, and basic tests. This requirement covers the remaining integration and cleanup items identified in the completion audit:

1. Wire the existing `MiddlewarePipeline` into the CLI main loop.
2. Complete the hook lifecycle by triggering `beforeStep` and fixing `afterStep` ordering.
3. Add a unit test for Anthropic SSE parsing.
4. Remove the redundant `ProviderComponent` by having the engine read the API key from configuration consistently.

## Acceptance Criteria

- [ ] `MiddlewarePipeline` is instantiated and applied to every event in the CLI main loop before dispatching to systems.
- [ ] A middleware integration test verifies that a middleware can transform or short-circuit events in the CLI loop.
- [ ] `InputSystem` emits a `beforeStep` hook trigger after appending the user message.
- [ ] `EffectExecutor` increments `LoopComponent.step` before emitting the `afterStep` hook trigger.
- [ ] `HookSystem::shouldContinue` reads `LoopComponent.should_continue` and emits an `abort` effect when the loop should stop.
- [ ] `ProviderComponent` is removed and `ConfigComponent` is used as the single configuration source.
- [ ] Anthropic SSE parsing has a unit test with a canned SSE payload.
- [ ] `cargo test --all-targets`, `cargo clippy --all-targets`, and `cargo build --release` all pass with zero warnings.

## Notes

- Keep changes minimal and focused. Do not introduce new features beyond the listed items.
- Maintain pure-function `System` design.
- The `ConfigComponent` should become the source of truth for model, temperature, max_steps, and provider name.
