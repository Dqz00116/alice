use alice_core::components::{
    ConfigComponent, LoopComponent, MessagesComponent, ProviderComponent, ToolsComponent,
};
use alice_core::effect_executor::EffectExecutor;
use alice_core::event::{Event, InputEvent};
use alice_core::event_bus::EventBus;
use alice_core::system_registry::SystemRegistry;
use alice_core::systems::input::input_system;
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::world::{HasComponent, World};

#[derive(Default)]
struct TestComponents {
    messages: MessagesComponent,
    config: ConfigComponent,
    loop_state: LoopComponent,
    tools: ToolsComponent,
    provider: ProviderComponent,
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

impl HasComponent<ProviderComponent> for TestComponents {
    fn get(&self) -> &ProviderComponent { &self.provider }
    fn get_mut(&mut self) -> &mut ProviderComponent { &mut self.provider }
}

#[test]
fn test_input_to_append_message_flow() {
    let components = TestComponents::default();
    let mut world = World::new(components);
    let event_bus = EventBus::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();

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
        &event_bus,
        &tool_scheduler,
        &mut abort_manager,
    );
    executor.execute(effects);

    let msgs = &world.get::<MessagesComponent>().messages;
    assert_eq!(msgs.len(), 1);
    assert!(matches!(msgs[0], alice_core::types::Message::User { .. }));
}
