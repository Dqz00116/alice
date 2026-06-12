use crate::event::Event;
use std::collections::HashMap;

type Handler = Box<dyn Fn(&Event) + Send + Sync>;

pub struct EventBus {
    subscribers: HashMap<String, Vec<Handler>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe<F>(&mut self, event_type: &str, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.subscribers
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    pub fn emit(&self, event: &Event) {
        let event_type = event.event_type();
        if let Some(handlers) = self.subscribers.get(event_type) {
            for handler in handlers {
                handler(event);
            }
        }
    }
}
