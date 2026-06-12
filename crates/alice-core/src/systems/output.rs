use crate::effect::{Effect, StreamType};
use crate::event::{Event, LLMStreamEvent, SystemEvent};
use crate::world::Snapshot;

pub fn output_system<C>(_snapshot: &Snapshot<&C>, event: &Event) -> Vec<Effect> {
    match event {
        Event::LLMStream(LLMStreamEvent::ThinkingDelta { delta }) => {
            vec![Effect::Render {
                content: delta.clone(),
                stream: StreamType::Thinking,
            }]
        }
        Event::LLMStream(LLMStreamEvent::TextDelta { delta }) => {
            vec![Effect::Render {
                content: delta.clone(),
                stream: StreamType::Text,
            }]
        }
        Event::LLMStream(LLMStreamEvent::ToolCall { tool_call }) => {
            vec![Effect::Render {
                content: format!("[Tool: {}]", tool_call.function.name),
                stream: StreamType::Text,
            }]
        }
        Event::LLMStream(LLMStreamEvent::StreamEnd { .. }) => {
            vec![Effect::Emit {
                event: Event::System(SystemEvent::StepEnd { step: 0 }),
            }]
        }
        Event::LLMStream(LLMStreamEvent::StreamError { error }) => {
            vec![Effect::Render {
                content: format!("Error: {error}"),
                stream: StreamType::Text,
            }]
        }
        _ => vec![],
    }
}
