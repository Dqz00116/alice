use crate::effect::Effect;
use crate::event::Event;
use crate::world::Snapshot;

pub fn provider_system<C>(_snapshot: &Snapshot<&C>, event: &Event) -> Vec<Effect> {
    match event {
        Event::System(crate::event::SystemEvent::StepStart { .. })
        | Event::Tool(crate::event::ToolEvent::Result { .. }) => {
            vec![Effect::CallLLM { messages: vec![] }]
        }
        _ => vec![],
    }
}
