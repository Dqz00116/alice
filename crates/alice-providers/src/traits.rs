use alice_core::event::LLMStreamEvent;
use alice_core::types::Message;
use futures_core::Stream;
use std::pin::Pin;

pub trait StreamingProvider {
    fn format_messages(&self, messages: &[Message]) -> serde_json::Value;
    fn stream_chat(
        &self,
        body: serde_json::Value,
    ) -> Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send + '_>>;
}
