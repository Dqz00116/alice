use crate::effect::Effect;
use crate::event::Event;
use crate::event::ToolEvent;
use crate::event_bus::EventBus;
use crate::tool_scheduler::ToolScheduler;
use crate::abort_manager::AbortManager;
use crate::types::{Message, ToolCall, FunctionCall};
use crate::world::{HasComponent, World};

// Component for storing messages in World
pub struct MessagesComponent {
    pub messages: Vec<Message>,
}

impl HasComponent<MessagesComponent> for MessagesComponent {
    fn get(&self) -> &MessagesComponent { self }
    fn get_mut(&mut self) -> &mut MessagesComponent { self }
}

pub struct EffectExecutor<'a, Components> {
    world: &'a mut World<Components>,
    event_bus: &'a EventBus,
    tool_scheduler: &'a ToolScheduler,
    abort_manager: &'a mut AbortManager,
}

impl<'a, Components> EffectExecutor<'a, Components>
where
    Components: HasComponent<MessagesComponent>,
{
    pub fn new(
        world: &'a mut World<Components>,
        event_bus: &'a EventBus,
        tool_scheduler: &'a ToolScheduler,
        abort_manager: &'a mut AbortManager,
    ) -> Self {
        Self {
            world,
            event_bus,
            tool_scheduler,
            abort_manager,
        }
    }

    pub fn execute(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            self.apply(effect);
        }
    }

    fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::AppendMessage { entity: _, message } => {
                self.world.get_mut::<MessagesComponent>().messages.push(message);
            }
            Effect::Emit { event } => {
                self.event_bus.emit(&event);
            }
            Effect::Render { content, stream: _ } => {
                print!("{}", content);
            }
            Effect::Abort { reason } => {
                self.abort_manager.abort(reason);
            }
            Effect::ExecuteTool { tool_name, args } => {
                let tool_call = ToolCall {
                    id: format!("tool_{}", uuid_simple()),
                    call_type: "function".into(),
                    function: FunctionCall {
                        name: tool_name,
                        arguments: serde_json::to_string(&args).unwrap_or_default(),
                    },
                };
                let results = self.tool_scheduler.schedule(&[tool_call]);
                if let Some(r) = results.into_iter().next() {
                    match r.error {
                        Some(err) => {
                            self.event_bus.emit(&Event::Tool(ToolEvent::Error {
                                tool_call_id: r.tool_call_id,
                                error: err,
                            }));
                        }
                        None => {
                            self.event_bus.emit(&Event::Tool(ToolEvent::Result {
                                tool_call_id: r.tool_call_id,
                                result: r.result.unwrap_or_default(),
                            }));
                        }
                    }
                }
            }
            Effect::CallLLM { messages: _ } => {
                // Placeholder: wired when ProviderComponent is connected in later batch
            }
        }
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", ts)
}
