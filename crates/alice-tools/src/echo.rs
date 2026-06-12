use alice_core::types::ToolDef;

pub fn echo_def() -> ToolDef {
    ToolDef {
        name: "echo".into(),
        description: "Echo back the input".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": ["message"]
        }),
    }
}

pub fn echo_handler(args: serde_json::Value) -> String {
    args.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
