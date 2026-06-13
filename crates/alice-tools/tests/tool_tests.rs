use alice_tools::{bash, edit_file, glob, read_file, write_file};
use std::fs;

fn tmp_dir() -> std::path::PathBuf {
    let thread_id = format!("{:?}", std::thread::current().id());
    let dir = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("tmp")
        .join("tool_tests")
        .join(thread_id);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn clean_tmp() {
    let dir = tmp_dir();
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
}

#[test]
fn test_bash_echo() {
    let result = bash::bash_handler(serde_json::json!({
        "command": "echo hello"
    }));
    assert!(result.contains("hello"), "result: {result}");
}

#[test]
fn test_write_and_read_file() {
    clean_tmp();
    let file = "target/tmp/tool_tests/readme.txt";

    let write_result = write_file::write_file_handler(serde_json::json!({
        "file_path": file,
        "content": "Hello, tools!"
    }));
    assert!(write_result.contains("Wrote"), "write result: {write_result}");

    let read_result = read_file::read_file_handler(serde_json::json!({
        "file_path": file
    }));
    assert!(read_result.contains("Hello, tools!"), "read result: {read_result}");
}

#[test]
fn test_read_file_with_offset_and_limit() {
    clean_tmp();
    let file = "target/tmp/tool_tests/lines.txt";
    write_file::write_file_handler(serde_json::json!({
        "file_path": file,
        "content": "line1\nline2\nline3\nline4\nline5"
    }));

    let result = read_file::read_file_handler(serde_json::json!({
        "file_path": file,
        "offset": 2,
        "limit": 2
    }));
    assert!(result.contains("line2"));
    assert!(result.contains("line3"));
    assert!(!result.contains("line1"));
    assert!(!result.contains("line5"));
}

#[test]
fn test_edit_file() {
    clean_tmp();
    let file = "target/tmp/tool_tests/edit.txt";
    write_file::write_file_handler(serde_json::json!({
        "file_path": file,
        "content": "foo bar baz"
    }));

    let result = edit_file::edit_file_handler(serde_json::json!({
        "file_path": file,
        "old_string": "bar",
        "new_string": "qux"
    }));
    assert!(result.contains("Edited"), "edit result: {result}");

    let content = read_file::read_file_handler(serde_json::json!({ "file_path": file }));
    assert!(content.contains("foo qux baz"));
}

#[test]
fn test_glob() {
    clean_tmp();
    write_file::write_file_handler(serde_json::json!({
        "file_path": "target/tmp/tool_tests/a.txt",
        "content": "a"
    }));
    write_file::write_file_handler(serde_json::json!({
        "file_path": "target/tmp/tool_tests/b.txt",
        "content": "b"
    }));

    let result = glob::glob_handler(serde_json::json!({
        "pattern": "target/tmp/tool_tests/*.txt"
    }));
    assert!(result.contains("a.txt"));
    assert!(result.contains("b.txt"));
}

#[test]
fn test_path_escape_rejected() {
    let result = read_file::read_file_handler(serde_json::json!({
        "file_path": "../../../Cargo.toml"
    }));
    assert!(result.starts_with("Error:"), "result: {result}");
}
