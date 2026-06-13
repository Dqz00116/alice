---
id: DESIGN-3bc87e1a
title: Loop Step and Tool Result Fix
status: approved
---

## Context

The Rust engine's main loop currently functions for a single user-input → assistant-response turn, but two correctness issues prevent robust multi-turn or tool-using behavior:

1. `LoopComponent.step` is never incremented, so `max_steps` and per-step hooks are effectively broken.
2. Tool results are emitted as events but never stored in `MessagesComponent`, so the LLM cannot observe them.

## Decision

Make the minimal targeted changes in `EffectExecutor`:

- On `StreamEnd`, increment `LoopComponent.step` via `UpdateComponent`.
- On `ExecuteTool` success or error, append a `Message::Tool` to `MessagesComponent`.

## Detailed Design

### Loop step increment

In `EffectExecutor::CallLLM`, after appending `Message::Assistant` on `StreamEnd`:

```rust
self.event_sink.emit(Event::System(SystemEvent::HookTrigger {
    hook: "afterStep".into(),
}));
self.event_sink.emit(Effect::UpdateComponent { ... step += 1 ... });
```

Actually, `UpdateComponent` is an `Effect`, not an `Event`. Since we are inside `EffectExecutor`, we can directly mutate `LoopComponent`:

```rust
self.world.get_mut::<LoopComponent>().step += 1;
```

This is simpler and avoids re-entrancy. The `afterStep` hook is still emitted so `HookSystem` can update `should_continue` based on the new step.

### Tool result write-back

In `EffectExecutor::ExecuteTool`, after emitting `ToolEvent::Result`:

```rust
self.world.get_mut::<MessagesComponent>().messages.push(Message::Tool {
    content: result.clone(),
    tool_call_id: tool_call_id.clone(),
});
```

For errors, push the error string as content.

### Ordering

The order for a tool-call turn is:
1. LLM streams `ToolCall`.
2. `tool_system` emits `ExecuteTool`.
3. `EffectExecutor` runs the tool and appends `Message::Tool`.
4. `EffectExecutor` emits `ToolEvent::Result`.
5. `provider_system` sees `ToolEvent::Result` and emits `CallLLM`.
6. Next LLM call includes the tool result message.

## Testing

- Add a mock provider test where the LLM returns a tool call; verify the tool result message is appended and `LoopComponent.step` is incremented.
- Add a direct `EffectExecutor` unit test for `ExecuteTool`.

## Trade-offs

- Directly mutating `LoopComponent` in `EffectExecutor` is simpler than producing an `UpdateComponent` effect, and is consistent with how `MessagesComponent` is already mutated for `AppendMessage`.
- The fix keeps `System` functions pure; only `EffectExecutor` performs mutation.
