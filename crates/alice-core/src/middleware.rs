//! Onion-style middleware pipeline for events.
//!
//! Middleware can inspect, transform, or short-circuit events before they reach
//! the engine systems.

use crate::event::Event;
use std::sync::Arc;

/// A middleware function receives an event and a `Next` continuation.
/// It may transform the event, call `next.run(event)`, or return without calling next.
pub type MiddlewareFn = Arc<dyn Fn(Event, Next) -> Event + Send + Sync>;

/// Continuation passed to each middleware.
pub struct Next {
    f: Box<dyn FnOnce(Event) -> Event + Send + Sync>,
}

impl Next {
    /// Invoke the next middleware (or the engine if this is the last one).
    pub fn run(self, event: Event) -> Event {
        (self.f)(event)
    }
}

/// Onion-style middleware pipeline.
#[derive(Clone, Default)]
pub struct MiddlewarePipeline {
    middlewares: Arc<Vec<MiddlewareFn>>,
}

impl MiddlewarePipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self {
            middlewares: Arc::new(Vec::new()),
        }
    }

    /// Add a middleware to the end of the pipeline.
    pub fn add<F>(&mut self, mw: F)
    where
        F: Fn(Event, Next) -> Event + Send + Sync + 'static,
    {
        let mut middlewares = (*self.middlewares).clone();
        middlewares.push(Arc::new(mw));
        self.middlewares = Arc::new(middlewares);
    }

    /// Run an event through the pipeline.
    pub fn run(&self, event: Event) -> Event {
        self.run_internal(event, 0)
    }

    fn run_internal(&self, event: Event, index: usize) -> Event {
        if index >= self.middlewares.len() {
            return event;
        }
        let mw = self.middlewares[index].clone();
        let pipeline = self.clone();
        let next = Next {
            f: Box::new(move |event| pipeline.run_internal(event, index + 1)),
        };
        mw(event, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pipeline_passes_event_through() {
        let pipeline = MiddlewarePipeline::new();
        let event = Event::System(crate::event::SystemEvent::StepStart { step: 1 });
        assert_eq!(pipeline.run(event.clone()), event);
    }

    #[test]
    fn test_middleware_can_transform_event() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add(|event, next| {
            if let Event::System(crate::event::SystemEvent::StepStart { step }) = event {
                next.run(Event::System(crate::event::SystemEvent::StepStart { step: step + 1 }))
            } else {
                next.run(event)
            }
        });

        let event = Event::System(crate::event::SystemEvent::StepStart { step: 1 });
        let result = pipeline.run(event);
        assert_eq!(
            result,
            Event::System(crate::event::SystemEvent::StepStart { step: 2 })
        );
    }

    #[test]
    fn test_middleware_can_short_circuit() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add(|event, _next| {
            if let Event::System(crate::event::SystemEvent::StepStart { .. }) = event {
                Event::System(crate::event::SystemEvent::StepEnd { step: 99 })
            } else {
                event
            }
        });

        let event = Event::System(crate::event::SystemEvent::StepStart { step: 1 });
        let result = pipeline.run(event);
        assert_eq!(
            result,
            Event::System(crate::event::SystemEvent::StepEnd { step: 99 })
        );
    }
}
