use crate::components::{ConfigComponent, LoopComponent};
use crate::effect::Effect;
use crate::event::{Event, InputEvent, SystemEvent};
use crate::types::Message;
use crate::world::Snapshot;

pub fn input_system<C>(snapshot: &Snapshot<&C>, event: &Event) -> Vec<Effect>
where
    C: crate::world::HasComponent<LoopComponent> + crate::world::HasComponent<ConfigComponent>,
{
    match event {
        Event::Input(InputEvent { content, .. }) => {
            let step = snapshot.get::<LoopComponent>().step;
            let should_continue = snapshot.get::<LoopComponent>().should_continue
                && step < snapshot.get::<ConfigComponent>().max_steps;

            vec![
                Effect::AppendMessage {
                    entity: "agent".into(),
                    message: Message::User {
                        content: content.clone(),
                    },
                },
                Effect::Emit {
                    event: Event::System(SystemEvent::HookTrigger {
                        hook: "beforeStep".into(),
                    }),
                },
                Effect::Emit {
                    event: Event::System(SystemEvent::StepStart { step }),
                },
                Effect::UpdateComponent {
                    entity: "agent".into(),
                    update: crate::effect::UpdateFn::new(move |accessor| {
                        accessor.loop_mut().should_continue = should_continue;
                    }),
                },
            ]
        }
        _ => vec![],
    }
}
