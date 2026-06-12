use alice_core::world::{HasComponent, World};

struct TestComponents {
    counter: u32,
    label: String,
}

impl HasComponent<u32> for TestComponents {
    fn get(&self) -> &u32 { &self.counter }
    fn get_mut(&mut self) -> &mut u32 { &mut self.counter }
}

impl HasComponent<String> for TestComponents {
    fn get(&self) -> &String { &self.label }
    fn get_mut(&mut self) -> &mut String { &mut self.label }
}

#[test]
fn test_world_set_and_get_component() {
    let comps = TestComponents { counter: 0, label: "hello".into() };
    let world = World::new(comps);
    let snapshot = world.snapshot();
    assert_eq!(snapshot.get::<u32>(), &0);
    assert_eq!(snapshot.get::<String>(), "hello");
}

#[test]
fn test_world_mutate_component() {
    let comps = TestComponents { counter: 0, label: "hello".into() };
    let mut world = World::new(comps);
    *world.get_mut::<u32>() = 42;
    assert_eq!(world.get::<u32>(), &42);
}
