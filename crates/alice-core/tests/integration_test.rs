use alice_core::event::{Event, InputEvent};
use alice_core::event_bus::EventBus;
use alice_core::system_registry::SystemRegistry;
use alice_core::systems::input::input_system;
use alice_core::effect_executor::{EffectExecutor, MessagesComponent};
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::world::World;

#[test]
fn test_input_to_append_message_flow() {
    let components = MessagesComponent { messages: Vec::new() };
    let mut world = World::new(components);
    let event_bus = EventBus::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();

    let mut system_registry: SystemRegistry<MessagesComponent> = SystemRegistry::new();
    system_registry.register(input_system::<MessagesComponent>, &["input.user"]);

    let mut executor = EffectExecutor::new(
        &mut world,
        &event_bus,
        &tool_scheduler,
        &mut abort_manager,
    );

    // Simulate: InputSystem receives user input
    let snapshot = world.snapshot();
    let event = Event::Input(InputEvent {
        source: "cli".into(),
        content: "Hello Alice".into(),
    });
    let systems = system_registry.get_systems_for_event(&event);
    let mut effects = Vec::new();
    for sys in systems {
        effects.extend(sys.process(&snapshot, &event));
    }
    executor.execute(effects);

    // Verify message was appended
    let msgs = &world.get::<MessagesComponent>().messages;
    assert_eq!(msgs.len(), 1);
    assert!(matches!(msgs[0], alice_core::types::Message::User { .. }));
}
