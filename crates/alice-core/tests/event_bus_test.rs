use std::sync::Mutex;
use alice_core::event::{Event, SystemEvent};
use alice_core::event_bus::EventBus;

#[test]
fn test_subscribe_and_emit() {
    let mut bus = EventBus::new();
    let received: Mutex<Vec<Event>> = Mutex::new(Vec::new());
    bus.subscribe("system.step_start", |event| {
        received.lock().unwrap().push(event.clone());
    });
    let event = Event::System(SystemEvent::StepStart { step: 1 });
    bus.emit(&event);
    let events = received.lock().unwrap();
    assert_eq!(events.len(), 1);
}
