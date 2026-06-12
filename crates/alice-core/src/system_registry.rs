use crate::{event::Event, system::System};

pub struct SystemRegistry<C> {
    entries: Vec<(Box<dyn System<C> + Send + Sync>, Vec<String>)>,
}

impl<C> SystemRegistry<C> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register<S>(&mut self, system: S, event_types: &[&str])
    where
        S: System<C> + Send + Sync + 'static,
    {
        self.entries.push((
            Box::new(system),
            event_types.iter().map(|s| s.to_string()).collect(),
        ));
    }

    pub fn get_systems_for_event(&self, event: &Event) -> Vec<&dyn System<C>> {
        let event_type = event.event_type();
        self.entries
            .iter()
            .filter(|(_, types)| types.iter().any(|t| t == event_type))
            .map(|(sys, _)| sys.as_ref())
            .collect()
    }
}
