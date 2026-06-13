//! Data components stored in the World.
//!
//! All components are plain data. Systems read them via Snapshot and request
//! mutations by returning Effects.

use crate::types::{Message, ToolDef};

/// Conversation history.
#[derive(Clone, Debug, Default)]
pub struct MessagesComponent {
    pub messages: Vec<Message>,
}

/// Engine configuration for a single run.
#[derive(Clone, Debug)]
pub struct ConfigComponent {
    pub model: String,
    pub temperature: f32,
    pub max_steps: u32,
    pub provider: String,
}

impl Default for ConfigComponent {
    fn default() -> Self {
        Self {
            model: "claude-3-5-sonnet-20241022".into(),
            temperature: 0.7,
            max_steps: 10,
            provider: "anthropic".into(),
        }
    }
}

/// Mutable loop state.
#[derive(Clone, Debug, Default)]
pub struct LoopComponent {
    pub step: u32,
    pub should_continue: bool,
}

/// Registered tool definitions.
#[derive(Clone, Debug, Default)]
pub struct ToolsComponent {
    pub definitions: Vec<ToolDef>,
}

/// Provider configuration for the active entity.
#[derive(Clone, Debug, Default)]
pub struct ProviderComponent {
    pub api_key: Option<String>,
}

/// Mutable accessor used by `Effect::UpdateComponent`.
///
/// Implement this trait for any component bundle that contains all engine
/// components. This keeps the `Effect` enum non-generic while still allowing
/// type-safe component updates.
pub trait ComponentAccessor {
    fn messages_mut(&mut self) -> &mut MessagesComponent;
    fn config_mut(&mut self) -> &mut ConfigComponent;
    fn loop_mut(&mut self) -> &mut LoopComponent;
    fn tools_mut(&mut self) -> &mut ToolsComponent;
    fn provider_mut(&mut self) -> &mut ProviderComponent;
}
