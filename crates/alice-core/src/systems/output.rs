use crate::components::LoopComponent;
use crate::effect::{Effect, StreamType};
use crate::event::{Event, LLMStreamEvent, SystemEvent};
use crate::world::Snapshot;

pub fn output_system<C>(snapshot: &Snapshot<&C>, event: &Event) -> Vec<Effect>
where
    C: crate::world::HasComponent<LoopComponent>,
{
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
            let step = snapshot.get::<LoopComponent>().step;
            vec![
                Effect::Render {
                    content: "\n".into(),
                    stream: StreamType::Text,
                },
                Effect::Emit {
                    event: Event::System(SystemEvent::StepEnd { step }),
                },
            ]
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
