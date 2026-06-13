---
id: DESIGN-38f782cc
title: Rust-First Completion and TypeScript Removal
status: approved
---

## Context

The project currently contains two parallel implementations of the Alice engine:
- A complete TypeScript prototype under `src/` and `tests/`.
- An in-progress Rust rewrite under `crates/`.

The Rust rewrite has a clean compile, zero clippy warnings, and basic integration tests, but it is still a skeleton: several systems are placeholders, `CallLLM` is a no-op, SSE is not parsed, and the CLI does not run a loop. Maintaining both implementations creates duplicated architectural effort and an unclear project identity.

## Decision

Adopt **Approach A: Remove TypeScript first, then complete Rust vertically**.

Rationale:
- Eliminates the dual-implementation maintenance burden immediately.
- Forces the Rust code to become the single source of truth.
- Each subsequent Rust task is small, verifiable, and independent.

## Scope

### Must do on this branch

1. Remove all TypeScript source, tests, and build configuration.
2. Update `AGENTS.md` to describe a Rust-only project.
3. Complete the Rust engine so that the CLI can run a minimal end-to-end loop:
   - User input → append message → call LLM → render stream → emit tool calls → execute tool → feed result back → decide whether to continue.

### Out of scope for this task

- DeepSeek provider (to be added later or explicitly dropped).
- Advanced middleware features beyond the TS-equivalent pipeline.
- Production-ready error handling beyond `anyhow`/`thiserror` basics.

## Detailed Design

### 1. TypeScript Removal

Delete the following files and directories:

- `src/`
- `tests/world.test.ts`
- `tsconfig.json`
- `vitest.config.ts`
- `package.json`
- `package-lock.json`

Update:
- `AGENTS.md` — remove TS sections, update build/test commands to Rust only.

### 2. Rust Core Additions

#### 2.1 `Effect` enum

Add `UpdateComponent` variant:

```rust
pub enum Effect {
    // existing variants
    UpdateComponent {
        entity_id: EntityId,
        component: ComponentUpdate,
    },
}
```

`ComponentUpdate` is a type-level wrapper that lets `EffectExecutor` update a specific component on an entity.

#### 2.2 Components

Define concrete component types:

- `ConfigComponent` — model, temperature, max_steps, provider name.
- `LoopComponent` — current step, should_continue flag.
- `ToolsComponent` — registered tool definitions.
- `ProviderComponent` — provider configuration / API key reference.

These components are stored in `World<Components>` via the existing `HasComponent` trait.

#### 2.3 Systems

- `InputSystem`
  - On `Event::Input { entity_id, content }`, append a user message.
  - Read `LoopComponent` to emit `Event::StepStart { step }` with the correct step number.
- `ProviderSystem`
  - On `Event::StepStart` or `Event::ToolResult`, check `LoopComponent::should_continue` and `ConfigComponent::max_steps`.
  - If continuing, read `MessagesComponent`, build the provider request, and emit `Effect::CallLLM { messages }`.
- `OutputSystem`
  - On `Event::LLM` deltas, emit `Effect::Render`.
  - On `Event::StreamEnd`, read `LoopComponent` and emit `Event::StepEnd { step }` with the correct step.
  - On `Event::ToolCall`, emit `Effect::Render` and `Effect::ExecuteTool`.
- `HookSystem`
  - `beforeStep`: emit `Effect::UpdateComponent` or additional `Effect::Emit` as configured.
  - `afterStep`: update `LoopComponent::should_continue` based on stop conditions.
  - `shouldContinue`: a dedicated hook check used by `ProviderSystem`.

#### 2.4 `EffectExecutor`

- `CallLLM`: receive a `Box<dyn StreamingProvider>` from the host, call `stream_chat`, and for each yielded chunk emit `Event::LLM` into the `EventBus`.
- `ExecuteTool`: invoke the tool handler asynchronously and emit `Event::ToolResult`.
- `UpdateComponent`: mutate the corresponding component in `World`.

#### 2.5 Provider SSE

`AnthropicProvider::stream_chat`:
- Build request body with `format_messages`.
- Stream response body line by line.
- Parse SSE `data:` lines as JSON.
- Yield `StreamChunk` variants: `Text(String)`, `Thinking(String)`, `ToolCall(ToolCall)`, `StreamEnd`.
- Map errors to a provider-specific error type.

#### 2.6 CLI Loop

`crates/alice/src/main.rs`:
- Create `World<AllComponents>`.
- Create `EventBus`, `SystemRegistry`, `ToolScheduler`, `AbortManager`, `EffectExecutor`.
- Register all systems.
- Register the echo tool.
- Instantiate `AnthropicProvider` from env var `ANTHROPIC_API_KEY`.
- Run a simple loop:
  1. Read a line from stdin.
  2. Emit `Event::Input`.
  3. Pump events through `SystemRegistry` until the event queue is empty or `LoopComponent::should_continue` becomes false.
  4. Print rendered output.

### 3. Testing Strategy

- Add unit tests for each new `Effect` variant handling.
- Add integration test for the CLI loop using a mock `StreamingProvider`.
- Keep existing tests green.
- Ensure `cargo clippy --all-targets` remains at zero warnings.

## Trade-offs

| Approach | Pros | Cons |
|---|---|---|
| Remove TS first | Single source of truth; smaller PRs later | Lose TS reference during Rust implementation |
| Complete Rust first | Can compare against running TS prototype | Prolonged duplication; larger final removal |
| Slice by slice | Very granular | High overhead; many context switches |

The chosen approach accepts the risk of losing the TS reference because the missing Rust pieces have already been identified and the TS structure is documented in the previous verification report.

## Migration Path

After this task:
1. The repository contains only Rust code.
2. The CLI runs a minimal but complete Alice loop.
3. Future providers (DeepSeek, OpenAI) and middleware enhancements are added as separate tasks.
