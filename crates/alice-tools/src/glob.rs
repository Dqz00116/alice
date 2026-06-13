use crate::utils::{resolve_and_validate_path, require_string_arg, truncate_output};
use alice_core::types::ToolDef;

pub fn glob_def() -> ToolDef {
    ToolDef {
        name: "Glob".into(),
        description: "List files matching a glob pattern under a directory within the project.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern (e.g. \"src/**/*.rs\")"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in, relative to the project root. Defaults to project root."
                }
            },
            "required": ["pattern"]
        }),
    }
}

pub fn glob_handler(args: serde_json::Value) -> String {
    match run_glob(args) {
        Ok(out) => out,
        Err(err) => format!("Error: {err}"),
    }
}

fn run_glob(args: serde_json::Value) -> Result<String, String> {
    let pattern = require_string_arg(&args, "pattern")?;
    let base_path = args
        .get("path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| ".".into());

    let resolved_base = resolve_and_validate_path(&base_path)?;
    let full_pattern = resolved_base.join(&pattern);
    let pattern_str = full_pattern
        .to_str()
        .ok_or_else(|| "invalid glob pattern encoding".to_string())?
        .replace('\\', "/");

    let mut matches: Vec<String> = Vec::new();
    for entry in glob::glob(&pattern_str).map_err(|e| format!("invalid glob pattern: {e}"))? {
        match entry {
            Ok(path) => {
                if let Some(s) = path.to_str() {
                    matches.push(s.to_string());
                }
            }
            Err(e) => matches.push(format!("error reading match: {e}")),
        }
    }

    matches.sort();
    let result = matches.join("\n");
    Ok(truncate_output(result, 50_000))
}
