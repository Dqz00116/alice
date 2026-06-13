# Pattern Analysis: Anthropic tool_use / tool_result Pairing

## Expected Working Pattern

Anthropic Messages API requires `tool_use` and `tool_result` blocks to be paired by
exact `id` / `tool_use_id` within the message sequence.

### Round 1: assistant asks for a tool

```json
{
  "role": "assistant",
  "content": [
    { "type": "text", "text": "I will look that up." },
    {
      "type": "tool_use",
      "id": "toolu_01A7q3FLK...",
      "name": "bash",
      "input": { "command": "ls" }
    }
  ]
}
```

### Round 2: user returns tool result

```json
{
  "role": "user",
  "content": [
    {
      "type": "tool_result",
      "tool_use_id": "toolu_01A7q3FLK...",
      "content": "src\nCargo.toml\n"
    }
  ]
}
```

The IDs must match byte-for-byte.

## What Alice Currently Does

`crates/alice-providers/src/anthropic.rs` already serializes both blocks in the shape
above. The serialization code is not the bug.

The bug is upstream: the `Message::Tool` that gets serialized carries a generated ID
instead of the original tool-use ID.

## Differences

| Aspect | Expected | Current |
|--------|----------|---------|
| `tool_use.id` source | LLM SSE `content_block_start.id` | LLM SSE `content_block_start.id` ✓ |
| `tool_result.tool_use_id` source | Same `tool_use.id` | New ID from `EffectExecutor::uuid_simple()` ✗ |
| Where ID is lost | N/A | `Effect::ExecuteTool` / `systems/tool.rs` |
| Where wrong ID is minted | N/A | `EffectExecutor::ExecuteTool` line 72 |

## Reference Implementation Pattern

The correct pattern is to thread the original `ToolCall.id` through the entire
execution chain:

```text
LLM SSE ToolCall.id
  → Message::Assistant.tool_calls[].id
  → Event::LLMStream(ToolCall { id, ... })
  → Effect::ExecuteTool { tool_call_id, tool_name, args }
  → ToolCall { id: tool_call_id, ... }
  → ToolResult { tool_call_id }
  → Message::Tool { tool_call_id }
  → Anthropic tool_result.tool_use_id
```

Any break in this chain causes the API to reject the request.

## Conclusion

The serialization pattern in `anthropic.rs` is already correct. The data-flow pattern
in `alice-core` fails to preserve the tool-call ID. The fix should not change the
Anthropic serialization; it should change how the ID is carried through effects and
the executor.
