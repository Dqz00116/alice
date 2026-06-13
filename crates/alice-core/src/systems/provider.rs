use crate::components::{ConfigComponent, LoopComponent, MessagesComponent};
use crate::effect::Effect;
use crate::event::{Event, SystemEvent, ToolEvent};
use crate::world::Snapshot;

pub fn provider_system<C>(snapshot: &Snapshot<&C>, event: &Event) -> Vec<Effect>
where
    C: crate::world::HasComponent<LoopComponent>
        + crate::world::HasComponent<ConfigComponent>
        + crate::world::HasComponent<MessagesComponent>,
{
    match event {
        Event::System(SystemEvent::StepStart { .. })
        | Event::Tool(ToolEvent::Result { .. })
        | Event::Tool(ToolEvent::Error { .. }) => {
            let loop_state = snapshot.get::<LoopComponent>();
            let config = snapshot.get::<ConfigComponent>();

            if !loop_state.should_continue || loop_state.step >= config.max_steps {
                return vec![];
            }

            let messages = snapshot.get::<MessagesComponent>().messages.clone();
            vec![Effect::CallLLM { messages }]
        }
        _ => vec![],
    }
}
