---
id: DESIGN-4594c98e
title: Multi-Turn CLI and Tool Support for Anthropic Provider
status: approved
---

# Multi-Turn CLI and Tool Support for Anthropic Provider

## Architecture

### Tool Definitions Flow

```
World::ToolsComponent.definitions
            │
            │ Effect::CallLLM
            ▼
EffectExecutor::CallLLM
            │
            │ provider.format_messages(messages, tools)
            ▼
AnthropicProvider serializes tools into request body
```

### Multi-Turn Flow

```
outer loop:
    print "You: "
    read stdin line
    if /exit or /quit -> break
    if empty -> continue
    push Event::Input

    inner event loop:
        process events until queue empty or abort

    if not should_continue -> break
```

## Component Changes

### `StreamingProvider` Trait

Change signature from:
```rust
fn format_messages(&self, messages: &[Message]) -> serde_json::Value;
```
to:
```rust
fn format_messages(&self, messages: &[Message], tools: &[ToolDef]) -> serde_json::Value;
```

### `AnthropicProvider`

- Accept `tools` in `format_messages`.
- Include `tools` array in JSON body only when non-empty.

### `EffectExecutor`

- In `Effect::CallLLM`, read `self.world.get::<ToolsComponent>().definitions`.
- Pass definitions to `format_messages`.

### CLI

- Move the existing event loop into a helper function or inline outer loop.
- Re-prompt after each assistant turn.
- Exit commands: `/exit`, `/quit`.

## Error Handling

- Empty input lines are ignored.
- Tool formatting errors should not crash; use `serde_json::to_value(...).unwrap_or_default()`.

## Testing

- Verify `format_messages` output contains `tools` key with correct schema.
- Verify CLI loop processes two sequential inputs.
