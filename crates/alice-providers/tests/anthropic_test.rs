use alice_core::event::LLMStreamEvent;
use alice_core::providers::StreamingProvider;
use alice_core::types::{FunctionCall, Message, ToolCall, ToolDef};
use alice_providers::anthropic::{parse_sse_data, parse_sse_value, AnthropicProvider};

#[test]
fn test_parse_sse_done() {
    let event = parse_sse_data("[DONE]");
    assert!(
        matches!(event, Some(LLMStreamEvent::StreamEnd { stop_reason }) if stop_reason == "end_turn")
    );
}

#[test]
fn test_parse_sse_text_delta() {
    let data = r#"{"type":"content_block_delta","delta":{"text":"Hello"}}"#;
    let event = parse_sse_data(data);
    assert!(
        matches!(event, Some(LLMStreamEvent::TextDelta { delta }) if delta == "Hello")
    );
}

#[test]
fn test_parse_sse_thinking_delta() {
    let data = r#"{"type":"content_block_delta","delta":{"thinking":"I think"}}"#;
    let event = parse_sse_data(data);
    assert!(
        matches!(event, Some(LLMStreamEvent::ThinkingDelta { delta }) if delta == "I think")
    );
}

#[test]
fn test_parse_sse_message_stop() {
    let data = r#"{"type":"message_stop"}"#;
    let event = parse_sse_data(data);
    assert!(
        matches!(event, Some(LLMStreamEvent::StreamEnd { stop_reason }) if stop_reason == "end_turn")
    );
}

#[test]
fn test_parse_sse_unknown_type() {
    let data = r#"{"type":"ping"}"#;
    assert!(parse_sse_data(data).is_none());
}

#[test]
fn test_parse_sse_invalid_json() {
    assert!(parse_sse_data("not valid json").is_none());
}

#[test]
fn test_custom_base_url_is_normalized() {
    let provider = AnthropicProvider::new(
        "fake-key".into(),
        "claude-test".into(),
        "https://api.example.com/anthropic/".into(),
    );
    assert_eq!(provider.base_url(), "https://api.example.com/anthropic/");
}

#[test]
fn test_default_base_url() {
    let provider = AnthropicProvider::new(
        "fake-key".into(),
        "claude-test".into(),
        "https://api.anthropic.com".into(),
    );
    assert_eq!(provider.base_url(), "https://api.anthropic.com");
}

#[test]
fn test_custom_model_id() {
    let provider = AnthropicProvider::new(
        "fake-key".into(),
        "claude-test-model".into(),
        "https://api.anthropic.com".into(),
    );
    assert_eq!(provider.model(), "claude-test-model");
}

#[test]
fn test_format_messages_includes_tools() {
    let provider = AnthropicProvider::new(
        "fake-key".into(),
        "claude-test".into(),
        "https://api.anthropic.com".into(),
    );
    let tools = vec![ToolDef {
        name: "echo".into(),
        description: "Echoes input".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": ["message"]
        }),
    }];
    let body = provider.format_messages(&[], &tools);
    let tools_array = body.get("tools").and_then(|v| v.as_array()).expect("expected tools array");
    assert_eq!(tools_array.len(), 1);
    assert_eq!(tools_array[0].get("name").and_then(|v| v.as_str()), Some("echo"));
}

#[test]
fn test_format_messages_includes_multiple_tools() {
    let provider = AnthropicProvider::new(
        "fake-key".into(),
        "claude-test".into(),
        "https://api.anthropic.com".into(),
    );
    let tools = vec![
        ToolDef {
            name: "echo".into(),
            description: "Echoes input".into(),
            input_schema: serde_json::json!({"type": "object"}),
        },
        ToolDef {
            name: "Bash".into(),
            description: "Runs shell commands".into(),
            input_schema: serde_json::json!({"type": "object"}),
        },
        ToolDef {
            name: "FileRead".into(),
            description: "Reads files".into(),
            input_schema: serde_json::json!({"type": "object"}),
        },
    ];
    let body = provider.format_messages(&[], &tools);
    let tools_array = body.get("tools").and_then(|v| v.as_array()).expect("expected tools array");
    assert_eq!(tools_array.len(), 3);
    let names: Vec<&str> = tools_array
        .iter()
        .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"echo"));
    assert!(names.contains(&"Bash"));
    assert!(names.contains(&"FileRead"));
}

#[test]
fn test_parse_tool_use_sse_events() {
    let mut tool: Option<(String, String, String)> = None;

    let start = serde_json::json!({
        "type": "content_block_start",
        "content_block": { "type": "tool_use", "id": "tool_1", "name": "echo" }
    });
    assert!(parse_sse_value(&start, &mut tool).is_none());

    let delta = serde_json::json!({
        "type": "content_block_delta",
        "delta": { "partial_json": r#"{"message":"hi"}"# }
    });
    assert!(parse_sse_value(&delta, &mut tool).is_none());

    let stop = serde_json::json!({ "type": "content_block_stop" });
    let event = parse_sse_value(&stop, &mut tool);
    match event {
        Some(LLMStreamEvent::ToolCall { tool_call }) => {
            assert_eq!(tool_call.id, "tool_1");
            assert_eq!(tool_call.function.name, "echo");
            let input: serde_json::Value = serde_json::from_str(&tool_call.function.arguments).unwrap();
            assert_eq!(input["message"], "hi");
        }
        _ => panic!("expected ToolCall event, got {:?}", event),
    }
}

#[test]
fn test_parse_tool_use_with_split_partial_json() {
    let mut tool: Option<(String, String, String)> = None;

    let start = serde_json::json!({
        "type": "content_block_start",
        "content_block": { "type": "tool_use", "id": "tool_2", "name": "echo" }
    });
    parse_sse_value(&start, &mut tool);

    parse_sse_value(
        &serde_json::json!({"type":"content_block_delta","delta":{"partial_json":"{\"mes"}}),
        &mut tool,
    );
    parse_sse_value(
        &serde_json::json!({"type":"content_block_delta","delta":{"partial_json":"sage\":\"hi"}}),
        &mut tool,
    );
    parse_sse_value(
        &serde_json::json!({"type":"content_block_delta","delta":{"partial_json":"\"}"}}),
        &mut tool,
    );

    let event = parse_sse_value(&serde_json::json!({"type":"content_block_stop"}), &mut tool);
    match event {
        Some(LLMStreamEvent::ToolCall { tool_call }) => {
            assert_eq!(tool_call.id, "tool_2");
            let input: serde_json::Value = serde_json::from_str(&tool_call.function.arguments).unwrap();
            assert_eq!(input["message"], "hi");
        }
        _ => panic!("expected ToolCall event"),
    }
}

#[test]
fn test_assistant_message_with_tool_use() {
    let provider = AnthropicProvider::new(
        "fake-key".into(),
        "claude-test".into(),
        "https://api.anthropic.com".into(),
    );
    let messages = vec![Message::Assistant {
        content: "I'll echo that.".into(),
        tool_calls: vec![ToolCall {
            id: "tool_1".into(),
            call_type: "tool_use".into(),
            function: FunctionCall {
                name: "echo".into(),
                arguments: r#"{"message":"hello"}"#.into(),
            },
        }],
    }];
    let body = provider.format_messages(&messages, &[]);
    let formatted = body.get("messages").and_then(|v| v.as_array()).unwrap();
    let msg = formatted[0].as_object().unwrap();
    assert_eq!(msg["role"], "assistant");
    let content = msg["content"].as_array().expect("expected content array");
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[1]["type"], "tool_use");
    assert_eq!(content[1]["name"], "echo");
    assert_eq!(content[1]["input"]["message"], "hello");
}

#[test]
fn test_tool_result_message() {
    let provider = AnthropicProvider::new(
        "fake-key".into(),
        "claude-test".into(),
        "https://api.anthropic.com".into(),
    );
    let messages = vec![Message::Tool {
        content: "hello".into(),
        tool_call_id: "tool_1".into(),
    }];
    let body = provider.format_messages(&messages, &[]);
    let formatted = body.get("messages").and_then(|v| v.as_array()).unwrap();
    let msg = &formatted[0];
    assert_eq!(msg["role"], "user");
    let content = msg["content"].as_array().expect("expected content array");
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "tool_result");
    assert_eq!(content[0]["tool_use_id"], "tool_1");
    assert_eq!(content[0]["content"], "hello");
}
