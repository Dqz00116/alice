use crate::components::ComponentAccessor;
use crate::event::Event;
use crate::types::Message;

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    CallLLM {
        messages: Vec<Message>,
    },
    ExecuteTool {
        tool_name: String,
        args: serde_json::Value,
    },
    AppendMessage {
        entity: String,
        message: Message,
    },
    UpdateComponent {
        entity: String,
        update: UpdateFn,
    },
    Emit {
        event: Event,
    },
    Render {
        content: String,
        stream: StreamType,
    },
    Abort {
        reason: String,
    },
}

/// Type-erased update closure used by `Effect::UpdateComponent`.
///
/// Systems produce these closures over the `ComponentAccessor` trait so that
/// the `Effect` enum can remain non-generic while still permitting type-safe
/// mutations of the concrete component bundle.
pub struct UpdateFn {
    f: std::sync::Arc<dyn Fn(&mut dyn ComponentAccessor) + Send + Sync>,
}

impl UpdateFn {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut dyn ComponentAccessor) + Send + Sync + 'static,
    {
        Self { f: std::sync::Arc::new(f) }
    }

    pub fn apply(&self, accessor: &mut dyn ComponentAccessor) {
        (self.f)(accessor);
    }
}

impl std::fmt::Debug for UpdateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateFn").finish_non_exhaustive()
    }
}

impl Clone for UpdateFn {
    fn clone(&self) -> Self {
        Self { f: self.f.clone() }
    }
}

impl PartialEq for UpdateFn {
    fn eq(&self, _other: &Self) -> bool {
        // Two type-erased closures cannot be compared meaningfully.
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Thinking,
    Text,
}
