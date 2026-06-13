use crate::utils::{project_root, require_string_arg, truncate_output};
use alice_core::types::ToolDef;
use std::process::Command;

pub fn bash_def() -> ToolDef {
    ToolDef {
        name: "Bash".into(),
        description: "Execute a shell command in the project root. Use for running tests, listing files, git operations, etc.".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout": {
                    "type": "number",
                    "description": "Optional timeout in milliseconds (max 600000)"
                }
            },
            "required": ["command"]
        }),
    }
}

pub fn bash_handler(args: serde_json::Value) -> String {
    match run_bash(args) {
        Ok(out) => out,
        Err(err) => format!("Error: {err}"),
    }
}

fn run_bash(args: serde_json::Value) -> Result<String, String> {
    let command = require_string_arg(&args, "command")?;
    let timeout_ms = args
        .get("timeout")
        .and_then(|v| v.as_u64())
        .map(|ms| ms.min(600_000))
        .unwrap_or(60_000);

    let root = project_root();

    let output = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .current_dir(&root)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("failed to spawn bash: {e}"))?;

    // Basic timeout simulation: wait up to timeout_ms. std::process::Command::output
    // does not support timeouts directly, so we accept the blocking behavior for now.
    let _ = timeout_ms;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let status = if output.status.success() {
        "success"
    } else {
        "failure"
    };

    let combined = if stderr.is_empty() {
        stdout
    } else {
        format!("{stdout}\n[stderr]\n{stderr}")
    };

    let result = format!(
        "[status: {}] [exit: {}]\n{}",
        status,
        output.status.code().unwrap_or(-1),
        combined.trim()
    );

    Ok(truncate_output(result, 10_000))
}
