# Verification: tool_call_id Preservation Fix

## Changes Applied

1. `crates/alice-core/src/effect.rs`
   - Added `tool_call_id: String` to `Effect::ExecuteTool`.

2. `crates/alice-core/src/systems/tool.rs`
   - `tool_system` now copies the original `tool_call.id` into the effect.

3. `crates/alice-core/src/effect_executor.rs`
   - `EffectExecutor::ExecuteTool` uses the passed-in ID for the `ToolCall` and the
     resulting `Message::Tool`.
   - Removed the now-unused `uuid_simple()` helper.

4. `crates/alice-core/tests/effect_executor_test.rs`
   - Updated existing test to assert the exact original ID is preserved.
   - Added `test_execute_tool_unknown_tool_preserves_id`.
   - Added `test_tool_system_preserves_tool_call_id`.

5. `crates/alice-tools/tests/tool_tests.rs`
   - Made `tmp_dir()` unique per thread to eliminate a parallel-test race condition
     that caused an unrelated flaky failure in `test_read_file_with_offset_and_limit`.

## Test Results

```
cargo test
```

All tests pass:

- `alice`: 0 unit tests
- `cli_loop_test`: 3 passed
- `alice_core` unit tests: 3 passed
- `effect_executor_test`: 4 passed (including new regression tests)
- `event_bus_test`: 1 passed
- `integration`: 1 passed
- `system_registry_test`: 1 passed
- `world_test`: 2 passed
- `anthropic_test`: 14 passed
- `tool_tests`: 6 passed (was flaky before tmp-dir fix)
- Doc tests: 0 passed

```
cargo clippy
```

No warnings.

## Regression Prevention

The new tests enforce that:

- A known `tool_call_id` survives from `Event::LLMStream(ToolCall)` through
  `tool_system`, `EffectExecutor`, and into `Message::Tool`.
- Unknown tools still produce a `Message::Tool` with the same ID, ensuring error
  paths also satisfy Anthropic's pairing requirement.

## Conclusion

The fix eliminates the `tool_use_id` mismatch. The Anthropic API should now accept
tool-result messages because `tool_result.tool_use_id` matches the preceding
assistant `tool_use.id`.
