use alice_core::event::LLMStreamEvent;
use alice_core::providers::StreamingProvider;
use alice_core::types::ToolDef;
use alice_providers::anthropic::{parse_sse_data, AnthropicProvider};

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
