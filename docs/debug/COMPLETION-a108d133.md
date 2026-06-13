# Debug Completion: LLM Only Saw `echo` Tool

## Bug Fixed

- **Root cause:** `ToolsComponent.definitions` in `crates/alice/src/main.rs` was
  initialized with only `echo::echo_def()`, while `ToolScheduler` had handlers for
  six tools. The LLM only received one tool definition and therefore only called
  `echo`.
- **Fix applied:** Synchronized `ToolsComponent.definitions` with the tools
  registered in `ToolScheduler` by adding `bash`, `read_file`, `write_file`,
  `edit_file`, and `glob` definitions.
- **All tests pass** (`cargo test`) and `cargo clippy` reports zero warnings.

## Files Changed

- `crates/alice/src/main.rs`
- `crates/alice-providers/tests/anthropic_test.rs` (added multi-tool regression test)

## Verification

See `docs/debug/VERIFICATION-a108d133.md` for full test results.
