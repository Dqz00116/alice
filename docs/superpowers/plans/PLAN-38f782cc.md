# Rust-First Completion and TypeScript Removal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the TypeScript prototype and complete the Rust engine so the CLI can run a full user-input → LLM stream → tool-call loop.

**Architecture:** Keep the existing 4-crate workspace and the pure-function `System` + declarative `Effect` design. Add missing Rust components, wire `StreamingProvider` into `EffectExecutor::CallLLM`, and implement the CLI event loop.

**Tech Stack:** Rust 2021, tokio, reqwest, serde/serde_json, thiserror, anyhow.

---

## File Map

| File | Responsibility |
|---|---|
| `crates/alice-core/src/effect.rs` | `Effect` enum; add `UpdateComponent`. |
| `crates/alice-core/src/types.rs` | Core shared types (`Message`, `ToolCall`, etc.). |
| `crates/alice-core/src/world.rs` | `World<Components>` and component traits. |
| `crates/alice-core/src/components.rs` | NEW — concrete component structs. |
| `crates/alice-core/src/lib.rs` | Re-export components module. |
| `crates/alice-core/src/effect_executor.rs` | Execute effects including `CallLLM` and `UpdateComponent`. |
| `crates/alice-core/src/systems/input.rs` | Handle `Event::Input` and emit `StepStart`. |
| `crates/alice-core/src/systems/provider.rs` | Decide when to call LLM and emit `CallLLM`. |
| `crates/alice-core/src/systems/output.rs` | Render deltas and emit `StepEnd`. |
| `crates/alice-core/src/systems/hook.rs` | Implement hook lifecycle. |
| `crates/alice-providers/src/traits.rs` | `StreamingProvider` trait. |
| `crates/alice-providers/src/anthropic.rs` | Anthropic SSE parsing. |
| `crates/alice/src/main.rs` | CLI binary wiring and event loop. |
| `AGENTS.md` | Project documentation update. |

---

### Task 1: Remove TypeScript Implementation and Build Files

**Files:**
- Delete: `src/`
- Delete: `tests/world.test.ts`
- Delete: `tsconfig.json`
- Delete: `vitest.config.ts`
- Delete: `package.json`
- Delete: `package-lock.json`

- [ ] **Step 1: Delete directories and files**

  ```bash
  rm -rf src/
  rm -f tests/world.test.ts
  rm -f tsconfig.json
  rm -f vitest.config.ts
  rm -f package.json
  rm -f package-lock.json
  ```

- [ ] **Step 2: Verify no TS artifacts remain in root**

  Run:
  ```bash
  ls -la
  ```
  Expected: No `src/`, no `package*.json`, no `tsconfig.json`, no `vitest.config.ts`, no `tests/world.test.ts`.

- [ ] **Step 3: Verify cargo still builds**

  Run:
  ```bash
  cargo build --release
  ```
  Expected: success, no warnings.

- [ ] **Step 4: Commit**

  ```bash
  git add -A
  git commit -m "chore: remove TypeScript prototype and build files"
  ```

---

### Task 2: Update AGENTS.md to Reflect Rust-Only Project

**Files:**
- Modify: `AGENTS.md`

- [ ] **Step 1: Rewrite project overview**

  Replace the dual-implementation description with:
  > Alice is a data-driven, stateless, streaming-concurrent, highly customizable CLI AI Agent engine implemented in Rust. Its core responsibility is orchestrating LLM interactions and tool calls.

- [ ] **Step 2: Update tech stack table**

  Remove TypeScript/Node.js rows. Keep only Rust rows:
  - Language: Rust (2021 edition)
  - Async runtime: tokio
  - HTTP: reqwest
  - Serialization: serde / serde_json
  - Error handling: thiserror + anyhow

- [ ] **Step 3: Update repository structure section**

  Remove `src/` and `tests/` from the tree. Keep `crates/`, `docs/`, `.devflow/`, `Cargo.toml`, `Cargo.lock`.

- [ ] **Step 4: Update build/test commands**

  Keep only:
  ```bash
  cargo build
  cargo build --release
  cargo test --all-targets
  cargo clippy --all-targets
  ```

- [ ] **Step 5: Update current limitations section**

  Replace with an honest note about the Rust engine status after this task completes.

- [ ] **Step 6: Commit**

  ```bash
  git add AGENTS.md
  git commit -m "docs: update AGENTS.md for Rust-only project"
  ```

---

### Task 3: Add UpdateComponent Effect Variant

**Files:**
- Modify: `crates/alice-core/src/effect.rs`
- Modify: `crates/alice-core/src/effect_executor.rs`
- Modify: `crates/alice-core/src/world.rs`

- [ ] **Step 1: Add component update abstraction**

  In `effect.rs`, add a trait object that can update a component:

  ```rust
  pub trait ComponentUpdate: std::any::Any + Send + Sync {
      fn update(&self, world: &mut dyn AnyWorld);
      fn as_any(&self) -> &dyn std::any::Any;
  }

  impl<T: 'static + Send + Sync> ComponentUpdate for T
  where
      T: Fn(&mut World<AllComponents>),
  {
      fn update(&self, world: &mut dyn AnyWorld) {
          if let Some(w) = world.as_any_mut().downcast_mut::<World<AllComponents>>() {
              self(w);
          }
      }

      fn as_any(&self) -> &dyn std::any::Any { self }
  }
  ```

  Note: `AnyWorld` is introduced in `world.rs` as a trait that exposes `as_any_mut()`.

- [ ] **Step 2: Add UpdateComponent variant**

  ```rust
  pub enum Effect {
      // existing variants
      UpdateComponent {
          entity_id: crate::types::EntityId,
          update: Box<dyn ComponentUpdate>,
      },
  }
  ```

- [ ] **Step 3: Add AnyWorld trait in world.rs**

  ```rust
  pub trait AnyWorld {
      fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
  }

  impl<Components> AnyWorld for World<Components>
  where
      Components: 'static,
  {
      fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
  }
  ```

- [ ] **Step 4: Handle UpdateComponent in EffectExecutor**

  Add branch:
  ```rust
  Effect::UpdateComponent { entity_id, update } => {
      if let Some(_entity) = world.get_entity_mut(entity_id) {
          update.update(world);
      }
  }
  ```

- [ ] **Step 5: Verify clippy**

  Run:
  ```bash
  cargo clippy --all-targets
  ```
  Expected: zero warnings.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/alice-core/src/effect.rs crates/alice-core/src/effect_executor.rs crates/alice-core/src/world.rs
  git commit -m "feat(core): add UpdateComponent effect variant"
  ```

---

### Task 4: Define Concrete Components

**Files:**
- Create: `crates/alice-core/src/components.rs`
- Modify: `crates/alice-core/src/lib.rs`
- Modify: `crates/alice-core/src/world.rs` (if needed for re-exports)

- [ ] **Step 1: Create components.rs**

  ```rust
  use serde_json::Value;

  #[derive(Clone, Debug, Default)]
  pub struct MessagesComponent {
      pub messages: Vec<crate::types::Message>,
  }

  #[derive(Clone, Debug, Default)]
  pub struct ConfigComponent {
      pub model: String,
      pub temperature: f32,
      pub max_steps: usize,
      pub provider: String,
  }

  #[derive(Clone, Debug, Default)]
  pub struct LoopComponent {
      pub step: usize,
      pub should_continue: bool,
  }

  #[derive(Clone, Debug, Default)]
  pub struct ToolsComponent {
      pub definitions: Vec<crate::types::ToolDef>,
  }

  #[derive(Clone, Debug, Default)]
  pub struct ProviderComponent {
      pub api_key: Option<String>,
  }
  ```

- [ ] **Step 2: Re-export in lib.rs**

  Add:
  ```rust
  pub mod components;
  pub use components::*;
  ```

- [ ] **Step 3: Add component tests**

  In `crates/alice-core/tests/components_test.rs`:

  ```rust
  use alice_core::{components::*, world::World, HasComponent};

  #[derive(Default)]
  struct AllComponents {
      messages: MessagesComponent,
      config: ConfigComponent,
      loop_state: LoopComponent,
      tools: ToolsComponent,
      provider: ProviderComponent,
  }

  impl HasComponent<MessagesComponent> for AllComponents { /* ... */ }
  impl HasComponent<ConfigComponent> for AllComponents { /* ... */ }
  impl HasComponent<LoopComponent> for AllComponents { /* ... */ }
  impl HasComponent<ToolsComponent> for AllComponents { /* ... */ }
  impl HasComponent<ProviderComponent> for AllComponents { /* ... */ }

  #[test]
  fn test_components_defaults() {
      let world = World::<AllComponents>::new();
      let msgs = world.get_component::<MessagesComponent>(0).unwrap();
      assert!(msgs.messages.is_empty());
  }
  ```

- [ ] **Step 4: Run tests**

  Run:
  ```bash
  cargo test --all-targets
  ```
  Expected: all pass.

- [ ] **Step 5: Commit**

  ```bash
  git add crates/alice-core/src/components.rs crates/alice-core/src/lib.rs crates/alice-core/tests/components_test.rs
  git commit -m "feat(core): define concrete engine components"
  ```

---

### Task 5: Update InputSystem to Track Loop Step

**Files:**
- Modify: `crates/alice-core/src/systems/input.rs`
- Modify: `crates/alice-core/tests/integration.rs`

- [ ] **Step 1: Read LoopComponent from snapshot**

  ```rust
  use crate::{components::LoopComponent, event::Event, effect::Effect, world::Snapshot};

  pub fn input_system(_snapshot: &Snapshot, event: &Event) -> Vec<Effect> {
      match event {
          Event::Input { entity_id, content } => {
              // Read current step from snapshot if available
              let step = _snapshot
                  .world
                  .get_component::<LoopComponent>(*entity_id)
                  .map(|l| l.step)
                  .unwrap_or(0);

              vec![
                  Effect::AppendMessage {
                      entity_id: *entity_id,
                      message: crate::types::Message::user(content),
                  },
                  Effect::Emit {
                      event: Event::StepStart { step },
                  },
              ]
          }
          _ => vec![],
      }
  }
  ```

  Note: ensure `Snapshot` exposes `world` or add accessor.

- [ ] **Step 2: Update integration test expectations**

  Verify `StepStart { step }` carries the current step.

- [ ] **Step 3: Run tests**

  Run:
  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add crates/alice-core/src/systems/input.rs crates/alice-core/tests/integration.rs
  git commit -m "fix(input): read loop step for StepStart"
  ```

---

### Task 6: Update OutputSystem to Emit Correct StepEnd

**Files:**
- Modify: `crates/alice-core/src/systems/output.rs`

- [ ] **Step 1: Read LoopComponent for StreamEnd**

  ```rust
  Event::StreamEnd { entity_id } => {
      let step = snapshot
          .world
          .get_component::<LoopComponent>(*entity_id)
          .map(|l| l.step)
          .unwrap_or(0);
      vec![Effect::Render { text: "\n".into() }, Effect::Emit { event: Event::StepEnd { step } }]
  }
  ```

- [ ] **Step 2: Add tests**

  Add unit test verifying `StepEnd` carries the correct step.

- [ ] **Step 3: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add crates/alice-core/src/systems/output.rs
  git commit -m "fix(output): emit StepEnd with correct loop step"
  ```

---

### Task 7: Implement Full HookSystem Lifecycle

**Files:**
- Modify: `crates/alice-core/src/systems/hook.rs`

- [ ] **Step 1: Add hook event handling**

  ```rust
  use crate::{components::LoopComponent, effect::Effect, event::Event, world::Snapshot};

  pub fn hook_system(snapshot: &Snapshot, event: &Event) -> Vec<Effect> {
      match event {
          Event::HookTrigger { hook, entity_id } => match hook.as_str() {
              "beforeStep" => {
                  // Optionally emit a system message or log
                  vec![]
              }
              "afterStep" => {
                  let should_continue = should_continue(snapshot, *entity_id);
                  vec![Effect::UpdateComponent {
                      entity_id: *entity_id,
                      update: Box::new(move |world: &mut crate::world::World<AllComponents>| {
                          if let Some(loop_state) = world.get_component_mut::<LoopComponent>(*entity_id) {
                              loop_state.should_continue = should_continue;
                          }
                      }),
                  }]
              }
              "shouldContinue" => {
                  vec![]
              }
              _ => vec![],
          },
          _ => vec![],
      }
  }

  fn should_continue(snapshot: &Snapshot, entity_id: u64) -> bool {
      let loop_state = snapshot.world.get_component::<LoopComponent>(entity_id);
      let config = snapshot.world.get_component::<ConfigComponent>(entity_id);
      match (loop_state, config) {
          (Some(loop_state), Some(config)) => {
              loop_state.step < config.max_steps && loop_state.should_continue
          }
          _ => false,
      }
  }
  ```

  Note: `AllComponents` must be available. The exact generic parameter may differ; use the project's chosen component bundle type.

- [ ] **Step 2: Add tests**

  Test `afterStep` sets `should_continue` to false when `step >= max_steps`.

- [ ] **Step 3: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add crates/alice-core/src/systems/hook.rs
  git commit -m "feat(hook): implement beforeStep, afterStep, shouldContinue lifecycle"
  ```

---

### Task 8: Implement Anthropic SSE Parsing

**Files:**
- Modify: `crates/alice-providers/src/anthropic.rs`

- [ ] **Step 1: Parse SSE lines**

  Inside `stream_chat`, after receiving the response, stream the bytes:

  ```rust
  use async_stream::try_stream;
  use futures_util::StreamExt;
  use serde_json::Value;

  pub fn stream_chat(
      &self,
      messages: &[Message],
  ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
      let request = self.build_request(messages);
      let client = self.client.clone();
      let api_key = self.api_key.clone();

      Box::pin(try_stream! {
          let response = client
              .post("https://api.anthropic.com/v1/messages")
              .header("x-api-key", api_key)
              .header("anthropic-version", "2023-06-01")
              .json(&request)
              .send()
              .await?;

          let mut stream = response.bytes_stream();
          let mut buffer = String::new();

          while let Some(chunk) = stream.next().await {
              let chunk = chunk?;
              buffer.push_str(&String::from_utf8_lossy(&chunk));

              while let Some(pos) = buffer.find('\n') {
                  let line = buffer.drain(..=pos).collect::<String>();
                  let line = line.trim();
                  if let Some(data) = line.strip_prefix("data: ") {
                      if data == "[DONE]" {
                          yield StreamChunk::StreamEnd;
                          return;
                      }
                      let json: Value = serde_json::from_str(data)?;
                      match json.get("type").and_then(|v| v.as_str()) {
                          Some("content_block_delta") => {
                              if let Some(delta) = json.get("delta") {
                                  if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                      yield StreamChunk::Text(text.to_string());
                                  }
                              }
                          }
                          Some("message_stop") => {
                              yield StreamChunk::StreamEnd;
                          }
                          _ => {}
                      }
                  }
              }
          }
          yield StreamChunk::StreamEnd;
      })
  }
  ```

- [ ] **Step 2: Update types if needed**

  Ensure `StreamChunk` includes `Text`, `Thinking`, `ToolCall`, `StreamEnd` variants.

- [ ] **Step 3: Add unit test**

  Parse a canned SSE payload and assert the correct chunks are yielded.

- [ ] **Step 4: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add crates/alice-providers/src/anthropic.rs
  git commit -m "feat(provider): implement Anthropic SSE parsing"
  ```

---

### Task 9: Wire StreamingProvider into EffectExecutor::CallLLM

**Files:**
- Modify: `crates/alice-core/src/effect_executor.rs`

- [ ] **Step 1: Accept provider in executor construction**

  ```rust
  pub struct EffectExecutor<P> {
      provider: P,
  }

  impl<P> EffectExecutor<P>
  where
      P: alice_providers::traits::StreamingProvider + Send + Sync + 'static,
  {
      pub fn new(provider: P) -> Self {
          Self { provider }
      }
  }
  ```

- [ ] **Step 2: Implement CallLLM**

  ```rust
  Effect::CallLLM { entity_id, messages } => {
      let provider = self.provider.clone();
      let event_bus = event_bus.clone();
      let entity_id = *entity_id;
      tokio::spawn(async move {
          let mut stream = provider.stream_chat(&messages).await;
          while let Some(chunk) = stream.next().await {
              match chunk {
                  Ok(chunk) => {
                      let event = match chunk {
                          StreamChunk::Text(text) => Event::LLM { entity_id, delta: LLMDelta::Text(text) },
                          StreamChunk::Thinking(text) => Event::LLM { entity_id, delta: LLMDelta::Thinking(text) },
                          StreamChunk::ToolCall(tc) => Event::LLM { entity_id, delta: LLMDelta::ToolCall(tc) },
                          StreamChunk::StreamEnd => Event::StreamEnd { entity_id },
                      };
                      event_bus.emit(event);
                  }
                  Err(e) => {
                      event_bus.emit(Event::Error { entity_id, message: e.to_string() });
                  }
              }
          }
      });
  }
  ```

  Note: exact types may need adjustment. Ensure `EffectExecutor` is async-capable and has access to `EventBus`.

- [ ] **Step 3: Update tests**

  Add mock provider and verify `CallLLM` emits `Event::LLM` deltas.

- [ ] **Step 4: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add crates/alice-core/src/effect_executor.rs
  git commit -m "feat(executor): wire StreamingProvider into CallLLM"
  ```

---

### Task 10: Update ProviderSystem to Read Messages and Emit CallLLM

**Files:**
- Modify: `crates/alice-core/src/systems/provider.rs`

- [ ] **Step 1: Read messages and loop state**

  ```rust
  use crate::{components::{LoopComponent, MessagesComponent}, effect::Effect, event::Event, world::Snapshot};

  pub fn provider_system(snapshot: &Snapshot, event: &Event) -> Vec<Effect> {
      match event {
          Event::StepStart { step } | Event::ToolResult { step, .. } => {
              let entity_id = 0; // Use the primary entity ID from the event if available
              let should_continue = snapshot
                  .world
                  .get_component::<LoopComponent>(entity_id)
                  .map(|l| l.should_continue && l.step < l.max_steps)
                  .unwrap_or(false);

              if !should_continue {
                  return vec![];
              }

              let messages = snapshot
                  .world
                  .get_component::<MessagesComponent>(entity_id)
                  .map(|m| m.messages.clone())
                  .unwrap_or_default();

              vec![Effect::CallLLM { entity_id, messages }]
          }
          _ => vec![],
      }
  }
  ```

  Note: `Event::ToolResult` needs a `step` field; add it if missing.

- [ ] **Step 2: Add tests**

  Verify `ProviderSystem` emits `CallLLM` only when `should_continue` is true.

- [ ] **Step 3: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add crates/alice-core/src/systems/provider.rs
  git commit -m "feat(provider): read messages and emit CallLLM"
  ```

---

### Task 11: Implement Middleware Trait and Pipeline

**Files:**
- Create: `crates/alice-core/src/middleware.rs`
- Modify: `crates/alice-core/src/lib.rs`

- [ ] **Step 1: Define middleware types**

  ```rust
  use crate::event::Event;

  pub type MiddlewareFn = Box<dyn Fn(Event, Next) -> Event + Send + Sync>;

  pub struct Next {
      f: Box<dyn FnOnce(Event) -> Event + Send + Sync>,
  }

  impl Next {
      pub fn run(self, event: Event) -> Event {
          (self.f)(event)
      }
  }

  pub struct MiddlewarePipeline {
      middlewares: Vec<MiddlewareFn>,
  }

  impl MiddlewarePipeline {
      pub fn new() -> Self {
          Self { middlewares: vec![] }
      }

      pub fn add(&mut self, mw: MiddlewareFn) {
          self.middlewares.push(mw);
      }

      pub fn run(&self, event: Event) -> Event {
          self.run_internal(event, 0)
      }

      fn run_internal(&self, event: Event, index: usize) -> Event {
          if index >= self.middlewares.len() {
              return event;
          }
          let mw = &self.middlewares[index];
          let next = Next {
              f: Box::new(move |event| self.run_internal(event, index + 1)),
          };
          mw(event, next)
      }
  }
  ```

- [ ] **Step 2: Re-export**

  Add `pub mod middleware;` to `lib.rs`.

- [ ] **Step 3: Add tests**

  Test that middleware can transform events in sequence.

- [ ] **Step 4: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add crates/alice-core/src/middleware.rs crates/alice-core/src/lib.rs
  git commit -m "feat(core): add middleware pipeline"
  ```

---

### Task 12: Wire CLI Main Loop

**Files:**
- Modify: `crates/alice/src/main.rs`

- [ ] **Step 1: Create AllComponents bundle**

  ```rust
  use alice_core::{
      components::*,
      effect_executor::EffectExecutor,
      event::Event,
      event_bus::EventBus,
      system_registry::SystemRegistry,
      tool_scheduler::ToolScheduler,
      abort_manager::AbortManager,
      systems::{input, provider, tool, output, hook},
      world::{World, HasComponent},
  };
  use alice_providers::anthropic::AnthropicProvider;
  use alice_tools::echo;

  #[derive(Default)]
  struct AllComponents {
      messages: MessagesComponent,
      config: ConfigComponent,
      loop_state: LoopComponent,
      tools: ToolsComponent,
      provider: ProviderComponent,
  }

  impl HasComponent<MessagesComponent> for AllComponents { /* ... */ }
  // ... etc
  ```

- [ ] **Step 2: Assemble engine in main**

  ```rust
  #[tokio::main]
  async fn main() -> anyhow::Result<()> {
      let mut world = World::<AllComponents>::new();
      let entity_id = world.create_entity();

      let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
      world.set_component(entity_id, ProviderComponent { api_key });
      world.set_component(entity_id, ConfigComponent {
          model: "claude-3-5-sonnet-20241022".into(),
          temperature: 0.7,
          max_steps: 10,
          provider: "anthropic".into(),
      });
      world.set_component(entity_id, LoopComponent { step: 0, should_continue: true });
      world.set_component(entity_id, ToolsComponent {
          definitions: vec![alice_tools::echo::echo_def()],
      });

      let event_bus = EventBus::new();
      let mut registry = SystemRegistry::new();
      registry.register(Box::new(input::input_system));
      registry.register(Box::new(provider::provider_system));
      registry.register(Box::new(tool::tool_system));
      registry.register(Box::new(output::output_system));
      registry.register(Box::new(hook::hook_system));

      let tool_scheduler = ToolScheduler::new();
      tool_scheduler.register("echo", Box::new(alice_tools::echo::echo_handler));

      let abort_manager = AbortManager::new();

      let provider = AnthropicProvider::new(api_key.unwrap_or_default());
      let mut executor = EffectExecutor::new(provider, event_bus.clone(), tool_scheduler, abort_manager);

      let stdin = std::io::stdin();
      let mut line = String::new();
      println!("Alice CLI ready. Type a message:");
      stdin.read_line(&mut line)?;

      event_bus.emit(Event::Input { entity_id, content: line.trim().to_string() });

      loop {
          let events = event_bus.drain(); // or equivalent API
          if events.is_empty() { break; }
          for event in events {
              let effects = registry.dispatch(&world.snapshot(), &event);
              executor.execute(&mut world, effects).await?;
          }
          if let Some(loop_state) = world.get_component::<LoopComponent>(entity_id) {
              if !loop_state.should_continue { break; }
          }
      }

      Ok(())
  }
  ```

  Note: exact APIs may differ; adjust to match existing `EventBus`, `SystemRegistry`, `EffectExecutor` signatures.

- [ ] **Step 3: Verify build**

  Run:
  ```bash
  cargo build --release
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add crates/alice/src/main.rs
  git commit -m "feat(cli): wire systems, provider, tools and stdin loop"
  ```

---

### Task 13: Add CLI Loop Integration Test with Mock Provider

**Files:**
- Create: `crates/alice/tests/cli_loop_test.rs`

- [ ] **Step 1: Create mock provider**

  ```rust
  use alice_core::types::{Message, StreamChunk};
  use alice_providers::traits::{ProviderError, StreamingProvider};
  use async_trait::async_trait;
  use std::pin::Pin;

  struct MockProvider;

  #[async_trait]
  impl StreamingProvider for MockProvider {
      fn stream_chat(
          &self,
          _messages: &[Message],
      ) -> Pin<Box<dyn futures::Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
          Box::pin(futures::stream::iter(vec![
              Ok(StreamChunk::Text("Hello".into())),
              Ok(StreamChunk::StreamEnd),
          ]))
      }
  }
  ```

- [ ] **Step 2: Test full loop**

  Build the engine with `MockProvider`, emit `Event::Input`, pump events, assert final messages contain assistant response.

- [ ] **Step 3: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add crates/alice/tests/cli_loop_test.rs
  git commit -m "test(cli): add integration test with mock provider"
  ```

---

### Task 14: Final Verification

**Files:**
- All crates

- [ ] **Step 1: Run full test suite**

  ```bash
  cargo test --all-targets
  ```
  Expected: all tests pass.

- [ ] **Step 2: Run clippy**

  ```bash
  cargo clippy --all-targets
  ```
  Expected: zero warnings.

- [ ] **Step 3: Run release build**

  ```bash
  cargo build --release
  ```
  Expected: success.

- [ ] **Step 4: Verify no TS artifacts remain**

  ```bash
  ls src/ 2>/dev/null || echo "src removed"
  test ! -f tsconfig.json && echo "tsconfig removed"
  test ! -f package.json && echo "package.json removed"
  ```

- [ ] **Step 5: Update completion document**

  Create `docs/completion/COMPLETION-38f782cc.md` summarizing the completed work.

- [ ] **Step 6: Final commit**

  ```bash
  git add -A
  git commit -m "feat: complete Rust engine and remove TypeScript implementation"
  ```

---

## Spec Coverage Check

| Spec Requirement | Implementing Task |
|---|---|
| Remove TS source | Task 1 |
| Update AGENTS.md | Task 2 |
| `UpdateComponent` effect | Task 3 |
| Concrete components | Task 4 |
| `InputSystem` step tracking | Task 5 |
| `OutputSystem` step tracking | Task 6 |
| Hook lifecycle | Task 7 |
| Anthropic SSE parsing | Task 8 |
| `CallLLM` wiring | Task 9 |
| `ProviderSystem` logic | Task 10 |
| Middleware | Task 11 |
| CLI loop | Task 12 |
| Integration tests | Task 13 |
| Final verification | Task 14 |

## Placeholder Scan

- No TBD/TODO left.
- No vague "add error handling" steps.
- Exact file paths are specified.
