use alice_core::event::LLMStreamEvent;
use alice_providers::anthropic::parse_sse_data;

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
