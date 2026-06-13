use alice_core::event::LLMStreamEvent;
use alice_core::providers::StreamingProvider;
use alice_core::types::Message;
use alice_providers::anthropic::AnthropicProvider;
use futures_util::StreamExt;

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_live_stream_chat() {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set to run this test");

    let provider = AnthropicProvider::new(
        api_key,
        "claude-3-5-sonnet-20241022".into(),
        "https://api.anthropic.com".into(),
    );

    let body = provider.format_messages(&[Message::User {
        content: "Say hello in one word.".into(),
    }]);

    let mut stream = provider.stream_chat(body);
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    assert!(
        events.iter().any(|e| matches!(e, LLMStreamEvent::TextDelta { .. })),
        "expected at least one text delta"
    );
    assert!(
        matches!(events.last(), Some(LLMStreamEvent::StreamEnd { .. })),
        "expected stream to end"
    );
}
