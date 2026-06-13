:
---
id: PLAN-0fd4b318
title: Complete Remaining Rust Engine Integration and Cleanup
status: approved
---

# Complete Remaining Rust Engine Integration and Cleanup

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire middleware, complete hook lifecycle, add Anthropic SSE test, and remove redundant `ProviderComponent`.

**Architecture:** Focused changes to existing crates; no new crates.

**Tech Stack:** Rust, Cargo, tokio, reqwest, serde_json.

---

### Task 1: Remove ProviderComponent and Move api_key to ConfigComponent

**Files:**
- Modify: `crates/alice-core/src/components.rs`
- Modify: `crates/alice-core/src/effect_executor.rs`
- Modify: `crates/alice-core/src/lib.rs` (if re-export)
- Modify: `crates/alice/src/main.rs`
- Modify: `crates/alice-core/tests/integration.rs`
- Modify: `crates/alice-core/tests/effect_executor_test.rs`
- Modify: `crates/alice/tests/cli_loop_test.rs`

- [ ] **Step 1: Update ConfigComponent and remove ProviderComponent**

  In `components.rs`:

  ```rust
  #[derive(Clone, Debug)]
  pub struct ConfigComponent {
      pub model: String,
      pub temperature: f32,
      pub max_steps: u32,
      pub provider: String,
      pub api_key: Option<String>,
  }

  impl Default for ConfigComponent {
      fn default() -> Self {
          Self {
              model: "claude-3-5-sonnet-20241022".into(),
              temperature: 0.7,
              max_steps: 10,
              provider: "anthropic".into(),
              api_key: None,
          }
      }
  }
  ```

  Delete `ProviderComponent` struct and its `Default` impl.

- [ ] **Step 2: Update ComponentAccessor trait**

  Remove `provider_mut` from `ComponentAccessor`.

- [ ] **Step 3: Update EffectExecutor bounds**

  In `effect_executor.rs`, remove `ProviderComponent` from `Components` bounds and from the `ComponentAccessor` impl for `World<T>`.

- [ ] **Step 4: Update CLI and tests**

  Remove `ProviderComponent` from `AllComponents`/`TestComponents` bundles and their `HasComponent` impls. Set `ConfigComponent.api_key` where needed.

- [ ] **Step 5: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 6: Commit**

  ```bash
  git add -A
  git commit -m "refactor(core): move api_key into ConfigComponent and remove ProviderComponent"
  ```

---

### Task 2: Fix Hook Lifecycle

**Files:**
- Modify: `crates/alice-core/src/systems/input.rs`
- Modify: `crates/alice-core/src/systems/hook.rs`
- Modify: `crates/alice-core/src/effect_executor.rs`

- [ ] **Step 1: Emit beforeStep from InputSystem**

  In `input_system`, after appending user message and emitting `StepStart`, add:

  ```rust
  Effect::Emit {
      event: Event::System(SystemEvent::HookTrigger { hook: "beforeStep".into() }),
  }
  ```

- [ ] **Step 2: Increment step before afterStep in EffectExecutor**

  In `EffectExecutor::CallLLM`, in the `StreamEnd` branch, reorder:

  ```rust
  self.world.get_mut::<LoopComponent>().step += 1;
  self.event_sink.emit(Event::System(SystemEvent::HookTrigger {
      hook: "afterStep".into(),
  }));
  ```

- [ ] **Step 3: Implement shouldContinue in HookSystem**

  In `hook_system`, in the `shouldContinue` branch:

  ```rust
  "shouldContinue" => {
      let should_continue = snapshot.get::<LoopComponent>().should_continue;
      if !should_continue {
          vec![Effect::Abort {
              reason: "shouldContinue returned false".into(),
          }]
      } else {
          vec![]
      }
  }
  ```

- [ ] **Step 4: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add -A
  git commit -m "fix(core): complete hook lifecycle with beforeStep and shouldContinue"
  ```

---

### Task 3: Wire MiddlewarePipeline into CLI Loop

**Files:**
- Modify: `crates/alice/src/main.rs`
- Modify: `crates/alice/tests/cli_loop_test.rs`

- [ ] **Step 1: Add middleware import and pipeline creation**

  In `main.rs`:

  ```rust
  use alice_core::middleware::MiddlewarePipeline;

  let mut pipeline = MiddlewarePipeline::new();
  ```

- [ ] **Step 2: Apply pipeline to each event**

  In the main loop:

  ```rust
  while let Some(raw_event) = queue.pop_front() {
      if abort_manager.is_aborted() { break; }
      let event = pipeline.run(raw_event);
      // ... rest of dispatch logic uses `event`
  }
  ```

- [ ] **Step 3: Add middleware integration test**

  In `cli_loop_test.rs`, add a test where a middleware transforms `StepStart { step: 0 }` into `StepStart { step: 42 }`, then assert the transformed step is observable.

- [ ] **Step 4: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add -A
  git commit -m "feat(cli): wire MiddlewarePipeline into main loop"
  ```

---

### Task 4: Add Anthropic SSE Parser Unit Test

**Files:**
- Create: `crates/alice-providers/tests/anthropic_test.rs`

- [ ] **Step 1: Create test file**

  ```rust
  use alice_core::event::LLMStreamEvent;
  use alice_providers::anthropic::AnthropicProvider;
  use futures_util::StreamExt;

  #[tokio::test]
  async fn test_anthropic_sse_parsing() {
      let provider = AnthropicProvider::new("fake-key".into(), "claude-test".into());
      let body = serde_json::json!({"model": "claude-test", "messages": [], "stream": true});
      let mut stream = provider.stream_chat(body);

      let mut events = Vec::new();
      while let Some(event) = stream.next().await {
          events.push(event);
      }

      assert!(matches!(events.last(), Some(LLMStreamEvent::StreamEnd { .. })));
  }
  ```

  Note: because `stream_chat` makes a real HTTP request, this test needs a mock server or a refactor to allow injecting a response stream. Prefer refactoring `AnthropicProvider` to accept an optional HTTP client / response builder for testability, or use `wiremock`.

  Simpler approach: extract SSE parsing into a pure function `parse_sse_line(line: &str) -> Option<LLMStreamEvent>` and test that function directly.

- [ ] **Step 2: Extract SSE parsing helper (optional but recommended)**

  In `anthropic.rs`:

  ```rust
  fn parse_sse_data(data: &str) -> Option<LLMStreamEvent> {
      if data == "[DONE]" {
          return Some(LLMStreamEvent::StreamEnd { stop_reason: "end_turn".into() });
      }
      let value: serde_json::Value = serde_json::from_str(data).ok()?;
      match value.get("type").and_then(|v| v.as_str()) {
          Some("content_block_delta") => {
              value.get("delta")?.get("text")?.as_str().map(|t| {
                  LLMStreamEvent::TextDelta { delta: t.to_string() }
              })
          }
          Some("message_stop") => Some(LLMStreamEvent::StreamEnd { stop_reason: "end_turn".into() }),
          _ => None,
      }
  }
  ```

  Then test `parse_sse_data` with canned JSON strings.

- [ ] **Step 3: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add -A
  git commit -m "test(provider): add Anthropic SSE parsing unit tests"
  ```

---

### Task 5: Final Verification

- [ ] **Step 1: Run full test suite**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 2: Run clippy**

  ```bash
  cargo clippy --all-targets
  ```

- [ ] **Step 3: Run release build**

  ```bash
  cargo build --release
  ```

- [ ] **Step 4: Commit any final changes**

  ```bash
  git add -A
  git commit -m "chore: final verification for remaining integration and cleanup" || true
  ```
