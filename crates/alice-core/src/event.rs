use crate::types::ToolCall;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Input(InputEvent),
    LLMStream(LLMStreamEvent),
    Tool(ToolEvent),
    System(SystemEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputEvent {
    pub source: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LLMStreamEvent {
    ThinkingDelta { delta: String },
    TextDelta { delta: String },
    ToolCall { tool_call: ToolCall },
    StreamEnd { stop_reason: String },
    StreamError { error: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolEvent {
    CallRequest { tool_calls: Vec<ToolCall> },
    Result { tool_call_id: String, result: String },
    Error { tool_call_id: String, error: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemEvent {
    StepStart { step: u32 },
    StepEnd { step: u32 },
    HookTrigger { hook: String },
}

impl Event {
    pub fn event_type(&self) -> &'static str {
        match self {
            Event::Input(_) => "input.user",
            Event::LLMStream(e) => match e {
                LLMStreamEvent::ThinkingDelta { .. } => "llm.thinking_delta",
                LLMStreamEvent::TextDelta { .. } => "llm.text_delta",
                LLMStreamEvent::ToolCall { .. } => "llm.tool_call",
                LLMStreamEvent::StreamEnd { .. } => "llm.stream_end",
                LLMStreamEvent::StreamError { .. } => "llm.stream_error",
            },
            Event::Tool(e) => match e {
                ToolEvent::CallRequest { .. } => "tool.call_request",
                ToolEvent::Result { .. } => "tool.result",
                ToolEvent::Error { .. } => "tool.error",
            },
            Event::System(e) => match e {
                SystemEvent::StepStart { .. } => "system.step_start",
                SystemEvent::StepEnd { .. } => "system.step_end",
                SystemEvent::HookTrigger { .. } => "system.hook_trigger",
            },
        }
    }
}
