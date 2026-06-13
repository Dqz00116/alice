use crate::utils::{require_string_arg, resolve_and_validate_path};
use alice_core::types::ToolDef;
use std::fs;

pub fn edit_file_def() -> ToolDef {
    ToolDef {
        name: "FileEdit".into(),
        description: "Replace a string in a file within the project.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file, relative to the project root"
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact text to replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The replacement text"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences instead of just the first"
                }
            },
            "required": ["file_path", "old_string", "new_string"]
        }),
    }
}

pub fn edit_file_handler(args: serde_json::Value) -> String {
    match run_edit_file(args) {
        Ok(msg) => msg,
        Err(err) => format!("Error: {err}"),
    }
}

fn run_edit_file(args: serde_json::Value) -> Result<String, String> {
    let file_path = require_string_arg(&args, "file_path")?;
    let old_string = require_string_arg(&args, "old_string")?;
    let new_string = require_string_arg(&args, "new_string")?;
    let replace_all = args.get("replace_all").and_then(|v| v.as_bool()).unwrap_or(false);

    if old_string == new_string {
        return Err("old_string and new_string must be different".into());
    }

    let resolved = resolve_and_validate_path(&file_path)?;
    let content = fs::read_to_string(&resolved)
        .map_err(|e| format!("failed to read {}: {e}", resolved.display()))?;

    let updated = if replace_all {
        content.replace(&old_string, &new_string)
    } else {
        content.replacen(&old_string, &new_string, 1)
    };

    if updated == content {
        return Err("old_string was not found in the file".into());
    }

    fs::write(&resolved, updated)
        .map_err(|e| format!("failed to write {}: {e}", resolved.display()))?;

    Ok(format!("Edited {}", resolved.display()))
}
