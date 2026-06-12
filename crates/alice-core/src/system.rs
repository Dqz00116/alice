use crate::{effect::Effect, event::Event, world::Snapshot};

pub trait System<C> {
    fn process(&self, snapshot: &Snapshot<C>, event: &Event) -> Vec<Effect>;
}

impl<C, F> System<C> for F
where
    F: Fn(&Snapshot<C>, &Event) -> Vec<Effect>,
{
    fn process(&self, snapshot: &Snapshot<C>, event: &Event) -> Vec<Effect> {
        self(snapshot, event)
    }
}

pub type SystemFn<C> = fn(&Snapshot<C>, &Event) -> Vec<Effect>;
