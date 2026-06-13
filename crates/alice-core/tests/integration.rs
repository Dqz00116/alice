use alice_core::components::{
    ConfigComponent, LoopComponent, MessagesComponent, ToolsComponent,
};
use alice_core::effect_executor::EffectExecutor;
use alice_core::event::{Event, InputEvent, LLMStreamEvent};
use alice_core::event_bus::EventBus;
use alice_core::providers::StreamingProvider;
use alice_core::system_registry::SystemRegistry;
use alice_core::systems::input::input_system;
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::types::Message;
use alice_core::abort_manager::AbortManager;
use alice_core::world::{HasComponent, World};
use futures_core::Stream;
use std::pin::Pin;

#[derive(Default)]
struct TestComponents {
    messages: MessagesComponent,
    config: ConfigComponent,
    loop_state: LoopComponent,
    tools: ToolsComponent,
}

impl HasComponent<MessagesComponent> for TestComponents {
    fn get(&self) -> &MessagesComponent { &self.messages }
    fn get_mut(&mut self) -> &mut MessagesComponent { &mut self.messages }
}

impl HasComponent<ConfigComponent> for TestComponents {
    fn get(&self) -> &ConfigComponent { &self.config }
    fn get_mut(&mut self) -> &mut ConfigComponent { &mut self.config }
}

impl HasComponent<LoopComponent> for TestComponents {
    fn get(&self) -> &LoopComponent { &self.loop_state }
    fn get_mut(&mut self) -> &mut LoopComponent { &mut self.loop_state }
}

impl HasComponent<ToolsComponent> for TestComponents {
    fn get(&self) -> &ToolsComponent { &self.tools }
    fn get_mut(&mut self) -> &mut ToolsComponent { &mut self.tools }
}

struct NullProvider;

impl StreamingProvider for NullProvider {
    fn format_messages(&self, _messages: &[Message]) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn stream_chat(
        &self,
        _body: serde_json::Value,
    ) -> Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send + '_>> {
        Box::pin(futures_util::stream::iter(vec![]))
    }
}

#[tokio::test]
async fn test_input_to_append_message_flow() {
    let components = TestComponents::default();
    let mut world = World::new(components);
    let mut event_bus = EventBus::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();
    let provider = NullProvider;

    let mut system_registry: SystemRegistry<TestComponents> = SystemRegistry::new();
    system_registry.register(input_system::<TestComponents>, &["input.user"]);

    let event = Event::Input(InputEvent {
        source: "cli".into(),
        content: "Hello Alice".into(),
    });
    let snapshot = world.snapshot();
    let systems = system_registry.get_systems_for_event(&event);
    let mut effects = Vec::new();
    for sys in systems {
        effects.extend(sys.process(&snapshot, &event));
    }

    let mut executor = EffectExecutor::new(
        &mut world,
        &mut event_bus,
        &tool_scheduler,
        &mut abort_manager,
        &provider,
    );
    executor.execute(effects).await;

    let msgs = &world.get::<MessagesComponent>().messages;
    assert_eq!(msgs.len(), 1);
    assert!(matches!(msgs[0], Message::User { .. }));
}
