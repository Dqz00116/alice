# Hypothesis: Synchronizing Tool Definitions Fixes the Echo-Only Behavior

## Hypothesis

**If** `ToolsComponent.definitions` is populated with the same six `ToolDef` instances
that are registered in `ToolScheduler`, **then** the Anthropic API request will include
all available tools, the model will be aware of `Bash`/`FileRead`/`Glob`/etc., and it
will stop defaulting to `echo` for every request.

## Reasoning

- `ToolScheduler` already has handlers for `echo`, `Bash`, `FileRead`, `FileWrite`,
  `FileEdit`, and `Glob`.
- `EffectExecutor::CallLLM` reads `ToolsComponent.definitions` and passes them to the
  provider, which serializes them into the API request's `tools` array.
- Currently only `echo::echo_def()` is placed into `ToolsComponent.definitions`.
- The model's behavior (calling only `echo`) is consistent with seeing only one tool.

## Test Plan

1. **Compile check**: `cargo check -p alice` must pass after updating `main.rs`.
2. **Lint check**: `cargo clippy -p alice` must report zero warnings.
3. **Full test suite**: `cargo test` must pass.
4. **Regression prevention**: Add or update a test that verifies the provider's
   `format_messages` serializes all provided tool definitions (already covered by
   existing `anthropic_test`, but we will ensure the count and names are checked).

## Prediction

After the fix, a live CLI run will show the LLM selecting tools other than `echo`
when the user's request requires filesystem or shell operations.
