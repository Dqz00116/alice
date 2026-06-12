use alice_core::effect::Effect;
use alice_core::event::{Event, SystemEvent};
use alice_core::system_registry::SystemRegistry;
use alice_core::world::{HasComponent, Snapshot};

struct InnerComponent {
    _value: i32,
}

struct TestComponents {
    inner: InnerComponent,
}

impl HasComponent<InnerComponent> for TestComponents {
    fn get(&self) -> &InnerComponent { &self.inner }
    fn get_mut(&mut self) -> &mut InnerComponent { &mut self.inner }
}

fn dummy_system(_snapshot: &Snapshot<&TestComponents>, _event: &Event) -> Vec<Effect> {
    vec![]
}

#[test]
fn test_register_and_lookup() {
    let mut registry: SystemRegistry<TestComponents> = SystemRegistry::new();
    registry.register(dummy_system, &["system.step_start"]);
    let event = Event::System(SystemEvent::StepStart { step: 1 });
    let systems = registry.get_systems_for_event(&event);
    assert_eq!(systems.len(), 1);
}
