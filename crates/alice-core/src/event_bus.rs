use crate::event::Event;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type Handler = Arc<dyn Fn(&Event) + Send + Sync>;

/// Sink for events produced by `EffectExecutor`.
pub trait EventSink {
    /// Deliver or queue an event.
    fn emit(&mut self, event: Event);
}

impl EventSink for EventBus {
    fn emit(&mut self, event: Event) {
        EventBus::emit(self, &event);
    }
}

impl EventSink for Vec<Event> {
    fn emit(&mut self, event: Event) {
        self.push(event);
    }
}

impl EventSink for std::collections::VecDeque<Event> {
    fn emit(&mut self, event: Event) {
        self.push_back(event);
    }
}

#[derive(Clone)]
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<String, Vec<Handler>>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe<F>(&mut self, event_type: &str, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.write().unwrap();
        subs.entry(event_type.to_string())
            .or_default()
            .push(Arc::new(handler));
    }

    pub fn emit(&self, event: &Event) {
        let event_type = event.event_type();
        let subs = self.subscribers.read().unwrap();
        if let Some(handlers) = subs.get(event_type) {
            for handler in handlers {
                handler(event);
            }
        }
    }
}
