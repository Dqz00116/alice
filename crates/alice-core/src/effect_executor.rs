use crate::components::{
    ComponentAccessor, ConfigComponent, LoopComponent, MessagesComponent, ProviderComponent,
    ToolsComponent,
};
use crate::effect::Effect;
use crate::event::{Event, ToolEvent};
use crate::event_bus::EventBus;
use crate::tool_scheduler::ToolScheduler;
use crate::abort_manager::AbortManager;
use crate::types::{FunctionCall, ToolCall};
use crate::world::{HasComponent, World};

pub struct EffectExecutor<'a, Components> {
    world: &'a mut World<Components>,
    event_bus: &'a EventBus,
    tool_scheduler: &'a ToolScheduler,
    abort_manager: &'a mut AbortManager,
}

impl<'a, Components> EffectExecutor<'a, Components>
where
    Components: HasComponent<MessagesComponent>
        + HasComponent<LoopComponent>
        + HasComponent<ConfigComponent>
        + HasComponent<ToolsComponent>
        + HasComponent<ProviderComponent>,
    World<Components>: ComponentAccessor,
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
            Effect::UpdateComponent { entity: _, update } => {
                update.apply(self.world);
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
                // Placeholder: wired when StreamingProvider is connected in later batch.
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

impl<T> ComponentAccessor for World<T>
where
    T: HasComponent<MessagesComponent>
        + HasComponent<LoopComponent>
        + HasComponent<ConfigComponent>
        + HasComponent<ToolsComponent>
        + HasComponent<ProviderComponent>,
{
    fn messages_mut(&mut self) -> &mut MessagesComponent {
        self.get_mut::<MessagesComponent>()
    }

    fn config_mut(&mut self) -> &mut ConfigComponent {
        self.get_mut::<ConfigComponent>()
    }

    fn loop_mut(&mut self) -> &mut LoopComponent {
        self.get_mut::<LoopComponent>()
    }

    fn tools_mut(&mut self) -> &mut ToolsComponent {
        self.get_mut::<ToolsComponent>()
    }

    fn provider_mut(&mut self) -> &mut ProviderComponent {
        self.get_mut::<ProviderComponent>()
    }
}
