//! Provider-facing traits used by the core engine.

use crate::event::LLMStreamEvent;
use crate::types::Message;
use futures_core::Stream;
use std::pin::Pin;

/// Abstraction over an LLM provider that can format messages and stream chat completions.
pub trait StreamingProvider: Send + Sync {
    /// Format a slice of engine messages into a provider-specific request body.
    fn format_messages(&self, messages: &[Message]) -> serde_json::Value;

    /// Start a streaming chat completion request and yield deltas as `LLMStreamEvent`s.
    fn stream_chat(
        &self,
        body: serde_json::Value,
    ) -> Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send + '_>>;
}
