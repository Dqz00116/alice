use alice_core::event_bus::EventBus;
use alice_core::effect_executor::MessagesComponent;
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::world::World;

fn main() {
    // World with MessagesComponent for conversation history
    let components = MessagesComponent { messages: Vec::new() };
    let _world = World::new(components);

    let _event_bus = EventBus::new();
    let _tool_scheduler = ToolScheduler::new();
    let _abort_manager = AbortManager::new();

    println!("Alice Agent ready.");
    println!("All infrastructure created: World, EventBus, ToolScheduler, AbortManager.");
    println!("Dynamic dispatch loop will be added in a future batch.");
}
