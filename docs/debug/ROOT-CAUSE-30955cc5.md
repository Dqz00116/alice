# Root Cause Analysis: Anthropic 400 Bad Request on tool_use_id

## Error Message

```
unexpected `messages.2.content.0: tool_use_id` found in `tool_result` blocks: `tool_18b89717f2d48498`.
Each `tool_result` block must have a corresponding `tool_use` block in the previous message.
```

## Summary

Anthropic requires that every `tool_result` content block reference the exact `id` of a
`tool_use` block from the previous `assistant` message. In the current implementation,
the `tool_result` is written back with a **newly generated** ID instead of the original
LLM tool-use ID, so the server cannot correlate the result with the tool call.

## Data Flow

1. **Anthropic SSE response** (`crates/alice-providers/src/anthropic.rs:60-64`)
   - Parses `content_block_start` of type `tool_use`.
   - Stores the original `id` in a `ToolCall`.

2. **Assistant message stored** (`crates/alice-core/src/effect_executor.rs:127-132`)
   - `Message::Assistant { tool_calls }` keeps the original `ToolCall.id`.
   - This part is correct.

3. **ToolSystem emits effect** (`crates/alice-core/src/systems/tool.rs:11-14`)
   - Creates `Effect::ExecuteTool { tool_name, args }`.
   - **Drops the original `tool_call.id`.**

4. **EffectExecutor mints a new ID** (`crates/alice-core/src/effect_executor.rs:71-78`)
   - Builds a new `ToolCall` with `id: format!("tool_{}", uuid_simple())`.
   - Calls `tool_scheduler.schedule(&[tool_call])`.

5. **ToolScheduler returns the new ID** (`crates/alice-core/src/tool_scheduler.rs:48-50`)
   - `ToolResult.tool_call_id` is set to the `ToolCall.id` it received.
   - The scheduler is not at fault.

6. **Tool result written to messages** (`crates/alice-core/src/effect_executor.rs:100-104`)
   - `Message::Tool { tool_call_id: r.tool_call_id }` uses the generated ID.

7. **Serialization mismatch** (`crates/alice-providers/src/anthropic.rs:119-150`)
   - Assistant message serializes `tool_use.id` = **original ID**.
   - User tool-result message serializes `tool_result.tool_use_id` = **generated ID**.
   - Anthropic rejects the request because the generated ID has no matching `tool_use`.

## Root Cause

`Effect::ExecuteTool` and `EffectExecutor::ExecuteTool` do not preserve the original
`tool_call.id` emitted by the LLM. The executor generates a fresh ID, causing a mismatch
between the assistant's `tool_use` block and the subsequent `tool_result` block.

## Fix Direction

- Add `tool_call_id: String` to `Effect::ExecuteTool`.
- Pass the original `tool_call.id` from `ToolSystem`.
- Use that exact ID when constructing the `ToolCall` in `EffectExecutor` and when writing
  `Message::Tool` back to the context.
