# Debug Completion: Anthropic tool_use_id Mismatch

## Bug Fixed

- **Root cause:** `Effect::ExecuteTool` discarded the LLM's original `tool_use` ID.
  `EffectExecutor` then minted a new ID, so the `tool_result` sent back to Anthropic
  referenced a non-existent `tool_use` block.
- **Fix applied:**
  - Added `tool_call_id` to `Effect::ExecuteTool`.
  - `tool_system` now forwards the original ID.
  - `EffectExecutor` uses that ID for scheduling and writes it back into
    `Message::Tool`.
- **All tests pass** (`cargo test`) and `cargo clippy` reports zero warnings.

## Files Changed

- `crates/alice-core/src/effect.rs`
- `crates/alice-core/src/systems/tool.rs`
- `crates/alice-core/src/effect_executor.rs`
- `crates/alice-core/tests/effect_executor_test.rs`
- `crates/alice-tools/tests/tool_tests.rs` (fixed unrelated parallel-test flakiness)

## Verification

See `docs/debug/VERIFICATION-30955cc5.md` for full test results.
