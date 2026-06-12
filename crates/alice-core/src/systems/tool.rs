use crate::effect::Effect;
use crate::event::{Event, LLMStreamEvent};
use crate::world::Snapshot;

pub fn tool_system<C>(_snapshot: &Snapshot<C>, event: &Event) -> Vec<Effect> {
    match event {
        Event::LLMStream(LLMStreamEvent::ToolCall { tool_call }) => {
            let args: serde_json::Value =
                serde_json::from_str(&tool_call.function.arguments)
                    .unwrap_or(serde_json::Value::Null);
            vec![Effect::ExecuteTool {
                tool_name: tool_call.function.name.clone(),
                args,
            }]
        }
        _ => vec![],
    }
}
