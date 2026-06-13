use alice_core::event::LLMStreamEvent;
use alice_core::types::Message;
use futures_core::Stream;
use futures_util::StreamExt;
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

            let response = match resp {
                Ok(r) => r,
                Err(e) => {
                    yield LLMStreamEvent::StreamError { error: format!("HTTP error: {e}") };
                    return;
                }
            };

            if let Err(e) = response.error_for_status_ref() {
                let text = response.text().await.unwrap_or_default();
                yield LLMStreamEvent::StreamError {
                    error: format!("Anthropic API error ({e}): {text}"),
                };
                return;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        yield LLMStreamEvent::StreamError { error: format!("stream error: {e}") };
                        return;
                    }
                };
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer.drain(..=pos).collect::<String>();
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            yield LLMStreamEvent::StreamEnd { stop_reason: "end_turn".into() };
                            return;
                        }

                        let value: serde_json::Value = match serde_json::from_str(data) {
                            Ok(v) => v,
                            Err(e) => {
                                yield LLMStreamEvent::StreamError {
                                    error: format!("invalid JSON in SSE data: {e}"),
                                };
                                return;
                            }
                        };

                        match value.get("type").and_then(|v| v.as_str()) {
                            Some("content_block_delta") => {
                                if let Some(delta) = value.get("delta") {
                                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                        yield LLMStreamEvent::TextDelta { delta: text.to_string() };
                                    }
                                    if let Some(thinking) = delta.get("thinking").and_then(|v| v.as_str()) {
                                        yield LLMStreamEvent::ThinkingDelta { delta: thinking.to_string() };
                                    }
                                }
                            }
                            Some("message_stop") => {
                                yield LLMStreamEvent::StreamEnd { stop_reason: "end_turn".into() };
                                return;
                            }
                            _ => {}
                        }
                    }
                }
            }

            yield LLMStreamEvent::StreamEnd { stop_reason: "end_turn".into() };
        })
    }
}
