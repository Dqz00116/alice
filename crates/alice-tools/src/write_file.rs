use crate::utils::{require_string_arg, resolve_and_validate_path};
use alice_core::types::ToolDef;
use std::fs;

pub fn write_file_def() -> ToolDef {
    ToolDef {
        name: "FileWrite".into(),
        description: "Write content to a file within the project, creating parent directories if needed.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file, relative to the project root"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write"
                }
            },
            "required": ["file_path", "content"]
        }),
    }
}

pub fn write_file_handler(args: serde_json::Value) -> String {
    match run_write_file(args) {
        Ok(msg) => msg,
        Err(err) => format!("Error: {err}"),
    }
}

fn run_write_file(args: serde_json::Value) -> Result<String, String> {
    let file_path = require_string_arg(&args, "file_path")?;
    let content = require_string_arg(&args, "content")?;

    let resolved = resolve_and_validate_path(&file_path)?;
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create directories for {}: {e}", resolved.display()))?;
    }

    fs::write(&resolved, content)
        .map_err(|e| format!("failed to write {}: {e}", resolved.display()))?;

    Ok(format!("Wrote {}", resolved.display()))
}
