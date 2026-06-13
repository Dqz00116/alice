---
id: DESIGN-0fd4b318
title: Complete Remaining Rust Engine Integration and Cleanup
status: approved
---

## Context

The Rust engine core is functional, but several integration and cleanup items remain from the audit:

1. Middleware pipeline is not wired into the CLI loop.
2. Hook lifecycle is incomplete (`beforeStep` not triggered, `afterStep` runs before step increment, `shouldContinue` is a no-op).
3. Anthropic SSE parsing lacks unit tests.
4. `ProviderComponent` is redundant alongside `ConfigComponent`.

## Decision

Make four focused, independent changes:

### 1. Middleware wiring

In `main.rs`:

```rust
let mut pipeline = MiddlewarePipeline::new();
// optionally pipeline.add(|event, next| next.run(event));

while let Some(raw_event) = queue.pop_front() {
    let event = pipeline.run(raw_event);
    // dispatch event
}
```

The pipeline receives the raw event and returns the processed event. Systems see the processed event.

### 2. Hook lifecycle

- `InputSystem` appends user message and then emits `HookTrigger { hook: "beforeStep" }` along with `StepStart`.
- `EffectExecutor::CallLLM` increments `LoopComponent.step` immediately when `StreamEnd` arrives, then emits `HookTrigger { hook: "afterStep" }`.
- `HookSystem::shouldContinue` checks `LoopComponent.should_continue`. If false, emits `Effect::Abort { reason: "shouldContinue returned false".into() }`.

### 3. Anthropic SSE test

Create `crates/alice-providers/tests/anthropic_test.rs` with a helper that constructs an SSE response string and feeds it through `AnthropicProvider::stream_chat`. Assert the yielded events match expected `TextDelta` and `StreamEnd`.

### 4. ProviderComponent cleanup

- Remove `ProviderComponent` from `components.rs`.
- Add `api_key: Option<String>` to `ConfigComponent`.
- Remove `ProviderComponent` from all `HasComponent` bounds in `EffectExecutor` and `ComponentAccessor`.
- Update CLI and tests to use `ConfigComponent.api_key`.

## Testing

- Existing CLI integration test updated to assert middleware transform.
- New `anthropic_test.rs` for SSE parsing.
- Existing tests updated to remove `ProviderComponent` from bundles.
- All tests pass with zero clippy warnings.

## Trade-offs

- Removing `ProviderComponent` reduces component count and uses `ConfigComponent` as the single config source.
- Emitting `beforeStep` adds one extra event per turn but makes the lifecycle complete.
- `shouldContinue` emitting `Abort` is the simplest way to terminate; alternatively it could set `should_continue` to false, but `Abort` is more explicit.
