# Root Cause Analysis: CLI Only Uses the `echo` Tool

## Symptom

In the interactive CLI, the LLM repeatedly calls only `[Tool: echo]` even when the
user asks for filesystem operations (e.g., list files in a directory). It does not
attempt to use `Bash`, `FileRead`, `Glob`, etc.

## Data Flow Investigation

1. **Tool handlers are registered** (`crates/alice/src/main.rs:76-82`)
   - `tool_scheduler.register(...)` is called for `echo`, `Bash`, `FileRead`,
     `FileWrite`, `FileEdit`, and `Glob`.
   - This makes all six tools executable inside `EffectExecutor::ExecuteTool`.

2. **Tool definitions sent to the LLM come from `ToolsComponent`**
   - `EffectExecutor::CallLLM` reads `world.get::<ToolsComponent>().definitions`
     and passes them to `provider.format_messages()`.
   - `AnthropicProvider::format_messages` includes these definitions in the API
     request body under the `tools` field.

3. **`ToolsComponent` is initialized with only `echo`**
   - `crates/alice/src/main.rs:70-73`:
     ```rust
     tools: ToolsComponent {
         definitions: vec![echo::echo_def()],
     },
     ```

4. **Result**
   - The Anthropic API receives a tool list containing only `echo`.
   - The model therefore believes only `echo` is available and repeatedly invokes it,
     producing behavior like the observed loop.

## Root Cause

`ToolsComponent.definitions` is not synchronized with the tools actually registered in
`ToolScheduler`. The LLM only sees the subset explicitly listed in the component, so it
can only call `echo`.

## Fix Direction

Populate `ToolsComponent.definitions` with the same set of `ToolDef`s that are
registered in `ToolScheduler`, or share a single source of truth between the
scheduler and the component.
