use crate::utils::{require_string_arg, resolve_and_validate_path, truncate_output};
use alice_core::types::ToolDef;
use std::fs;

pub fn read_file_def() -> ToolDef {
    ToolDef {
        name: "FileRead".into(),
        description: "Read the contents of a text file within the project.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file, relative to the project root"
                },
                "offset": {
                    "type": "number",
                    "description": "Optional 1-based line number to start reading from"
                },
                "limit": {
                    "type": "number",
                    "description": "Optional maximum number of lines to read"
                }
            },
            "required": ["file_path"]
        }),
    }
}

pub fn read_file_handler(args: serde_json::Value) -> String {
    match run_read_file(args) {
        Ok(out) => out,
        Err(err) => format!("Error: {err}"),
    }
}

fn run_read_file(args: serde_json::Value) -> Result<String, String> {
    let file_path = require_string_arg(&args, "file_path")?;
    let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(1);
    let limit = args.get("limit").and_then(|v| v.as_u64());

    let resolved = resolve_and_validate_path(&file_path)?;
    let content = fs::read_to_string(&resolved)
        .map_err(|e| format!("failed to read {}: {e}", resolved.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let start = offset.saturating_sub(1).min(lines.len() as u64) as usize;
    let end = limit
        .map(|l| start + l as usize)
        .unwrap_or(lines.len())
        .min(lines.len());

    let selected: Vec<&str> = lines[start..end].to_vec();
    let result = selected.join("\n");
    Ok(truncate_output(result, 50_000))
}
