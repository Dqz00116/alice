pub use alice_core::types::ToolHandler;

use alice_core::types::ToolDef;

pub struct Tool {
    pub def: ToolDef,
    pub handler: ToolHandler,
}
