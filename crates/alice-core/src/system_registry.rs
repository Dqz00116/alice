use crate::{effect::Effect, event::Event};

/// A type-erased system that operates on `Snapshot<&Components>`.
pub struct ErasedSystem<Components: ?Sized> {
    process_fn: Box<dyn Fn(&crate::world::Snapshot<&Components>, &Event) -> Vec<Effect> + Send + Sync>,
}

impl<Components: ?Sized> ErasedSystem<Components> {
    pub fn new<S>(system: S) -> Self
    where
        S: Fn(&crate::world::Snapshot<&Components>, &Event) -> Vec<Effect> + Send + Sync + 'static,
    {
        Self {
            process_fn: Box::new(system),
        }
    }

    pub fn process(
        &self,
        snapshot: &crate::world::Snapshot<&Components>,
        event: &Event,
    ) -> Vec<Effect> {
        (self.process_fn)(snapshot, event)
    }
}

pub struct SystemRegistry<Components> {
    entries: Vec<(ErasedSystem<Components>, Vec<String>)>,
}

impl<Components> SystemRegistry<Components> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register<S>(&mut self, system: S, event_types: &[&str])
    where
        S: Fn(&crate::world::Snapshot<&Components>, &Event) -> Vec<Effect> + Send + Sync + 'static,
    {
        self.entries.push((
            ErasedSystem::new(system),
            event_types.iter().map(|s| s.to_string()).collect(),
        ));
    }

    pub fn get_systems_for_event(&self, event: &Event) -> Vec<&ErasedSystem<Components>> {
        let event_type = event.event_type();
        self.entries
            .iter()
            .filter(|(_, types)| types.iter().any(|t| t == event_type))
            .map(|(sys, _)| sys)
            .collect()
    }
}
