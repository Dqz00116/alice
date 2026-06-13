use crate::components::{
    ComponentAccessor, ConfigComponent, LoopComponent, MessagesComponent, ToolsComponent,
};
use crate::effect::Effect;
use crate::event::{Event, LLMStreamEvent, SystemEvent, ToolEvent};
use crate::event_bus::EventSink;
use crate::providers::StreamingProvider;
use crate::tool_scheduler::ToolScheduler;
use crate::abort_manager::AbortManager;
use crate::types::{FunctionCall, ToolCall};
use crate::world::{HasComponent, World};
use futures_util::StreamExt;

pub struct EffectExecutor<'a, Components, P> {
    world: &'a mut World<Components>,
    event_sink: &'a mut dyn EventSink,
    tool_scheduler: &'a ToolScheduler,
    abort_manager: &'a mut AbortManager,
    provider: &'a P,
}

impl<'a, Components, P> EffectExecutor<'a, Components, P>
where
    Components: HasComponent<MessagesComponent>
        + HasComponent<LoopComponent>
        + HasComponent<ConfigComponent>
        + HasComponent<ToolsComponent>,
    World<Components>: ComponentAccessor,
    P: StreamingProvider,
{
    pub fn new(
        world: &'a mut World<Components>,
        event_sink: &'a mut dyn EventSink,
        tool_scheduler: &'a ToolScheduler,
        abort_manager: &'a mut AbortManager,
        provider: &'a P,
    ) -> Self {
        Self {
            world,
            event_sink,
            tool_scheduler,
            abort_manager,
            provider,
        }
    }

    pub async fn execute(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            self.apply(effect).await;
        }
    }

    async fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::AppendMessage { entity: _, message } => {
                self.world.get_mut::<MessagesComponent>().messages.push(message);
            }
            Effect::UpdateComponent { entity: _, update } => {
                update.apply(self.world);
            }
            Effect::Emit { event } => {
                self.event_sink.emit(event);
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
                            self.event_sink.emit(Event::Tool(ToolEvent::Error {
                                tool_call_id: r.tool_call_id.clone(),
                                error: err.clone(),
                            }));
                            self.world.get_mut::<MessagesComponent>().messages.push(
                                crate::types::Message::Tool {
                                    content: format!("Error: {err}"),
                                    tool_call_id: r.tool_call_id,
                                },
                            );
                        }
                        None => {
                            let result = r.result.unwrap_or_default();
                            self.event_sink.emit(Event::Tool(ToolEvent::Result {
                                tool_call_id: r.tool_call_id.clone(),
                                result: result.clone(),
                            }));
                            self.world.get_mut::<MessagesComponent>().messages.push(
                                crate::types::Message::Tool {
                                    content: result,
                                    tool_call_id: r.tool_call_id,
                                },
                            );
                        }
                    }
                }
            }
            Effect::CallLLM { messages } => {
                let body = self.provider.format_messages(&messages);
                let mut stream = self.provider.stream_chat(body);
                let mut assistant_content = String::new();
                let mut tool_calls = Vec::new();
                while let Some(event) = stream.next().await {
                    match &event {
                        LLMStreamEvent::TextDelta { delta } => {
                            assistant_content.push_str(delta);
                        }
                        LLMStreamEvent::ThinkingDelta { .. } => {}
                        LLMStreamEvent::ToolCall { tool_call } => {
                            tool_calls.push(tool_call.clone());
                        }
                        LLMStreamEvent::StreamEnd { .. } => {
                            if !assistant_content.is_empty() || !tool_calls.is_empty() {
                                self.world.get_mut::<MessagesComponent>().messages.push(
                                    crate::types::Message::Assistant {
                                        content: assistant_content.clone(),
                                        tool_calls: tool_calls.clone(),
                                    },
                                );
                            }
                            self.world.get_mut::<LoopComponent>().step += 1;
                            self.event_sink.emit(Event::System(SystemEvent::HookTrigger {
                                hook: "afterStep".into(),
                            }));
                        }
                        LLMStreamEvent::StreamError { .. } => {}
                    }
                    self.dispatch_stream_event(event);
                }
            }
        }
    }

    fn dispatch_stream_event(&mut self, event: LLMStreamEvent) {
        self.event_sink.emit(Event::LLMStream(event));
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
        + HasComponent<ToolsComponent>,
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
}
