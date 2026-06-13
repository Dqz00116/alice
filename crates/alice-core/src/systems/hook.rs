use crate::components::{ConfigComponent, LoopComponent};
use crate::effect::{Effect, UpdateFn};
use crate::event::{Event, SystemEvent};
use crate::world::Snapshot;

pub fn hook_system<C>(snapshot: &Snapshot<&C>, event: &Event) -> Vec<Effect>
where
    C: crate::world::HasComponent<LoopComponent> + crate::world::HasComponent<ConfigComponent>,
{
    match event {
        Event::System(SystemEvent::HookTrigger { hook }) => match hook.as_str() {
            "beforeStep" => {
                // Reserved for future use: logging, metrics, input transforms.
                vec![]
            }
            "afterStep" => {
                let loop_state = snapshot.get::<LoopComponent>();
                let config = snapshot.get::<ConfigComponent>();
                let should_continue = loop_state.should_continue
                    && loop_state.step < config.max_steps;

                vec![
                    Effect::UpdateComponent {
                        entity: "agent".into(),
                        update: UpdateFn::new(move |accessor| {
                            accessor.loop_mut().should_continue = should_continue;
                        }),
                    },
                    Effect::Emit {
                        event: Event::System(SystemEvent::HookTrigger {
                            hook: "shouldContinue".into(),
                        }),
                    },
                ]
            }
            "shouldContinue" => {
                let should_continue = snapshot.get::<LoopComponent>().should_continue;
                if should_continue {
                    vec![]
                } else {
                    vec![Effect::Abort {
                        reason: "shouldContinue returned false".into(),
                    }]
                }
            }
            _ => vec![],
        },
        _ => vec![],
    }
}
