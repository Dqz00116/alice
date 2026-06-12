use crate::{event::Event, types::Message};

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    CallLLM {
        messages: Vec<Message>,
    },
    ExecuteTool {
        tool_name: String,
        args: serde_json::Value,
    },
    AppendMessage {
        entity: String,
        message: Message,
    },
    Emit {
        event: Event,
    },
    Render {
        content: String,
        stream: StreamType,
    },
    Abort {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Thinking,
    Text,
}
