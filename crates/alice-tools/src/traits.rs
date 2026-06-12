use alice_core::types::ToolDef;

pub type ToolHandler = fn(args: serde_json::Value) -> String;

pub struct Tool {
    pub def: ToolDef,
    pub handler: ToolHandler,
}
