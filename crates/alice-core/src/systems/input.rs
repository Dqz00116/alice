use crate::effect::Effect;
use crate::event::{Event, InputEvent, SystemEvent};
use crate::types::Message;
use crate::world::Snapshot;

pub fn input_system<C>(_snapshot: &Snapshot<&C>, event: &Event) -> Vec<Effect> {
    match event {
        Event::Input(InputEvent { content, .. }) => {
            vec![
                Effect::AppendMessage {
                    entity: "agent".into(),
                    message: Message::User {
                        content: content.clone(),
                    },
                },
                Effect::Emit {
                    event: Event::System(SystemEvent::StepStart { step: 0 }),
                },
            ]
        }
        _ => vec![],
    }
}
