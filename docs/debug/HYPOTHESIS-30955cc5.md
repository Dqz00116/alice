# Hypothesis: Preserving tool_call_id resolves Anthropic 400

## Hypothesis

**If** `Effect::ExecuteTool` carries the original `tool_call.id` from the LLM tool-use
event and `EffectExecutor` uses that exact ID both for scheduling and for writing the
`Message::Tool`, **then** the Anthropic API will receive a matching `tool_use.id` / 
`tool_result.tool_use_id` pair and the 400 Bad Request will disappear.

## Reasoning

- The 400 error explicitly states that `tool_result.tool_use_id` has no corresponding
  `tool_use` block in the previous assistant message.
- The assistant message retains the original LLM ID, so it is not corrupted.
- The only place a new ID is introduced is `EffectExecutor::ExecuteTool`, which calls
  `uuid_simple()`.
- `ToolScheduler` preserves whatever ID it receives, so fixing the ID upstream is
  sufficient.

## Minimal Test Case

A unit test for `EffectExecutor` that simulates a `ToolCall` with a known ID and
verifies that the resulting `Message::Tool` has the same ID:

```rust
#[tokio::test]
async fn execute_tool_preserves_original_tool_call_id() {
    let tool_call = ToolCall {
        id: "toolu_original_id".to_string(),
        call_type: "tool_use".to_string(),
        function: FunctionCall {
            name: "bash".to_string(),
            arguments: r#"{"command":"echo hi"}"#.to_string(),
        },
    };

    // Emit LLMStream ToolCall event and ExecuteTool effect.
    // After execution, MessagesComponent should contain:
    //   Message::Assistant { tool_calls: [tool_call] }
    //   Message::Tool { tool_call_id: "toolu_original_id", content: "hi" }

    assert_eq!(
        messages[1],
        Message::Tool {
            content: "hi".to_string(),
            tool_call_id: "toolu_original_id".to_string(),
        }
    );
}
```

Before the fix this test will fail because `Message::Tool.tool_call_id` will be a
generated `tool_<timestamp>` value instead of `"toolu_original_id"`.

## Prediction

After applying the fix:

1. The new unit test passes.
2. Existing tests continue to pass.
3. A live CLI run no longer triggers the Anthropic 400 error when a tool result is
   sent back.
