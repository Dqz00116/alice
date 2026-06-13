---
id: DESIGN-30955cc5
title: End-to-End Anthropic Tool Calling and Core Toolset
status: approved
---

# End-to-End Anthropic Tool Calling and Core Toolset

## Architecture

### Tool-Use SSE Flow

```
Anthropic SSE
   │
   │ content_block_start(tool_use)
   ▼
start tracking ToolCall { id, name, input_fragments }
   │
   │ content_block_delta(partial_json)
   ▼
accumulate partial_json string
   │
   │ content_block_stop
   ▼
parse accumulated JSON -> input object
emit LLMStreamEvent::ToolCall { tool_call }
```

### Message Serialization

`Message::Assistant`:
- No tool calls → `{ "role": "assistant", "content": "..." }`
- With tool calls → `{ "role": "assistant", "content": [{ "type": "text", "text": "..." }, { "type": "tool_use", "id": "...", "name": "...", "input": {...} }] }`

`Message::Tool`:
- `{ "role": "user", "content": [{ "type": "tool_result", "tool_use_id": "...", "content": "..." }] }`

### Core Tools

Each tool lives in `crates/alice-tools/src/` as a module exposing:
- `pub fn <name>_def() -> ToolDef`
- `pub fn <name>_handler(args: serde_json::Value) -> String`

The CLI registers them via `tool_scheduler.register(<name>_def(), <name>_handler)`.

## Error Handling

- Tool-use JSON parsing failure → emit `LLMStreamEvent::StreamError`.
- File path outside project root → return error string in tool result.
- Bash command failure → return stderr + exit code info.
- File not found → return clear error message.

## Testing

- SSE parser unit tests with canned `tool_use` event sequences.
- Serialization unit tests comparing JSON output.
- Tool handler unit tests using temporary files under `target/tmp/`.
