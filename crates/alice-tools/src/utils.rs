use std::path::{Component, Path, PathBuf};

pub fn project_root() -> PathBuf {
    std::env::current_dir().expect("failed to get current directory")
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            _ => out.push(comp.as_os_str()),
        }
    }
    out
}

pub fn resolve_and_validate_path(path_str: &str) -> Result<PathBuf, String> {
    let root = project_root();
    let path = Path::new(path_str);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let normalized = normalize_path(&resolved);
    if !normalized.starts_with(&root) {
        return Err(format!(
            "path escapes project root: {}",
            normalized.display()
        ));
    }
    Ok(normalized)
}

pub fn require_string_arg(args: &serde_json::Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing or invalid required argument: {key}"))
}

pub fn truncate_output(output: String, max_len: usize) -> String {
    if output.len() > max_len {
        format!(
            "{}\n... (truncated, {} bytes total)",
            &output[..max_len],
            output.len()
        )
    } else {
        output
    }
}
