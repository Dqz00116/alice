use crate::effect::Effect;
use crate::event::{Event, SystemEvent};
use crate::world::Snapshot;

pub fn hook_system<C>(_snapshot: &Snapshot<C>, event: &Event) -> Vec<Effect> {
    match event {
        Event::System(SystemEvent::HookTrigger { hook }) => match hook.as_str() {
            "afterStep" => vec![Effect::Emit {
                event: Event::System(SystemEvent::HookTrigger {
                    hook: "shouldContinue".into(),
                }),
            }],
            _ => vec![],
        },
        _ => vec![],
    }
}
