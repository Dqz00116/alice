use alice_core::event::{Event, InputEvent, SystemEvent};
use alice_core::event_bus::EventBus;
use alice_core::system_registry::SystemRegistry;
use alice_core::systems::{
    hook::hook_system,
    input::input_system,
    output::output_system,
    provider::provider_system,
    tool::tool_system,
};
use alice_core::effect_executor::{EffectExecutor, MessagesComponent};
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::world::World;

fn main() {
    // World with MessagesComponent for conversation history
    let components = MessagesComponent { messages: Vec::new() };
    let mut world = World::new(components);

    let event_bus = EventBus::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();

    // Register all built-in Systems
    let mut system_registry: SystemRegistry<MessagesComponent> = SystemRegistry::new();
    system_registry.register(input_system::<MessagesComponent>, &["input.user"]);
    system_registry.register(provider_system::<MessagesComponent>, &["system.step_start", "tool.result"]);
    system_registry.register(tool_system::<MessagesComponent>, &["llm.tool_call"]);
    system_registry.register(
        output_system::<MessagesComponent>,
        &[
            "llm.thinking_delta",
            "llm.text_delta",
            "llm.tool_call",
            "llm.stream_end",
            "llm.stream_error",
        ],
    );
    system_registry.register(hook_system::<MessagesComponent>, &["system.hook_trigger"]);

    // Wire dispatch: EventBus -> SystemRegistry -> EffectExecutor
    // Note: In the full engine, this dispatch is internal to the run loop.
    // Here we show the wiring structure.

    let mut executor = EffectExecutor::new(
        &mut world,
        &event_bus,
        &tool_scheduler,
        &mut abort_manager,
    );

    println!("Alice Agent ready.");

    // Kick off the first step
    let snapshot = world.snapshot();
    let hook_event = Event::System(SystemEvent::HookTrigger {
        hook: "beforeStep".into(),
    });
    let systems = system_registry.get_systems_for_event(&hook_event);
    let mut effects = Vec::new();
    for sys in systems {
        effects.extend(sys.process(&snapshot, &hook_event));
    }
    executor.execute(effects);

    // Simulate user input for demonstration
    let input_event = Event::Input(InputEvent {
        source: "cli".into(),
        content: "Hello, Alice!".into(),
    });
    let snapshot = world.snapshot();
    let systems = system_registry.get_systems_for_event(&input_event);
    let mut effects = Vec::new();
    for sys in systems {
        effects.extend(sys.process(&snapshot, &input_event));
    }
    executor.execute(effects);

    // Print final state
    let msgs = world.get::<MessagesComponent>();
    println!("Messages in world: {}", msgs.messages.len());
    for msg in &msgs.messages {
        println!("  {:?}", msg);
    }
}
