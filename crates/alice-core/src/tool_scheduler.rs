use crate::types::{ToolCall, ToolDef, ToolHandler};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub result: Option<String>,
    pub error: Option<String>,
}

pub struct ToolScheduler {
    definitions: Vec<ToolDef>,
    handlers: HashMap<String, ToolHandler>,
}

impl ToolScheduler {
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, def: ToolDef, handler: ToolHandler) {
        self.handlers.insert(def.name.clone(), handler);
        self.definitions.push(def);
    }

    pub fn definitions(&self) -> &[ToolDef] {
        &self.definitions
    }

    pub fn schedule(&self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        tool_calls
            .iter()
            .map(|tc| {
                let args: serde_json::Value =
                    serde_json::from_str(&tc.function.arguments).unwrap_or(serde_json::Value::Null);
                match self.handlers.get(&tc.function.name) {
                    Some(handler) => {
                        let result = handler(args);
                        ToolResult {
                            tool_call_id: tc.id.clone(),
                            result: Some(result),
                            error: None,
                        }
                    }
                    None => ToolResult {
                        tool_call_id: tc.id.clone(),
                        result: None,
                        error: Some(format!("unknown tool: {}", tc.function.name)),
                    },
                }
            })
            .collect()
    }
}
