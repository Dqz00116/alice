use alice_core::event::LLMStreamEvent;
use alice_core::types::Message;
use futures_core::Stream;
use std::pin::Pin;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

impl super::traits::StreamingProvider for AnthropicProvider {
    fn format_messages(&self, messages: &[Message]) -> serde_json::Value {
        let formatted: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| match m {
                Message::User { content } => {
                    serde_json::json!({ "role": "user", "content": content })
                }
                Message::Assistant { content, tool_calls } => {
                    let mut obj = serde_json::json!({ "role": "assistant", "content": content });
                    if !tool_calls.is_empty() {
                        obj["tool_calls"] = serde_json::to_value(tool_calls).unwrap();
                    }
                    obj
                }
                Message::Tool {
                    content,
                    tool_call_id,
                } => {
                    serde_json::json!({
                        "role": "tool",
                        "content": content,
                        "tool_call_id": tool_call_id
                    })
                }
            })
            .collect();

        serde_json::json!({
            "model": self.model,
            "messages": formatted,
            "max_tokens": 4096,
            "stream": true,
        })
    }

    fn stream_chat(
        &self,
        body: serde_json::Value,
    ) -> Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send + '_>> {
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        Box::pin(async_stream::stream! {
            let resp = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await;

            match resp {
                Ok(_response) => {
                    yield LLMStreamEvent::TextDelta {
                        delta: "[Anthropic provider: SSE parsing TBD]".into(),
                    };
                    yield LLMStreamEvent::StreamEnd {
                        stop_reason: "end_turn".into(),
                    };
                }
                Err(e) => {
                    yield LLMStreamEvent::StreamError {
                        error: format!("HTTP error: {e}"),
                    };
                }
            }
        })
    }
}
