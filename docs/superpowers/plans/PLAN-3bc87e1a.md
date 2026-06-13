# Loop Step and Tool Result Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `LoopComponent.step` increment and tool result write-back to `MessagesComponent`.

**Architecture:** Minimal changes localized to `EffectExecutor`, plus focused regression tests.

**Tech Stack:** Rust, Cargo test, tokio.

---

### Task 1: Increment Loop Step on StreamEnd

**Files:**
- Modify: `crates/alice-core/src/effect_executor.rs`

- [ ] **Step 1: Add step increment after StreamEnd**

  In `EffectExecutor::apply`, inside the `LLMStreamEvent::StreamEnd { .. }` branch, after appending the assistant message and emitting the `afterStep` hook, increment `LoopComponent.step`:

  ```rust
  self.world.get_mut::<LoopComponent>().step += 1;
  ```

- [ ] **Step 2: Verify clippy**

  Run:
  ```bash
  cargo clippy --all-targets
  ```
  Expected: zero warnings.

- [ ] **Step 3: Commit**

  ```bash
  git add crates/alice-core/src/effect_executor.rs
  git commit -m "fix(executor): increment loop step after StreamEnd"
  ```

---

### Task 2: Append Tool Result to MessagesComponent

**Files:**
- Modify: `crates/alice-core/src/effect_executor.rs`

- [ ] **Step 1: Append tool result on success**

  In `EffectExecutor::apply`, inside the `Effect::ExecuteTool` branch, after emitting `ToolEvent::Result`, append a tool message:

  ```rust
  self.world.get_mut::<MessagesComponent>().messages.push(
      crate::types::Message::Tool {
          content: result.clone(),
          tool_call_id: r.tool_call_id.clone(),
      },
  );
  ```

- [ ] **Step 2: Append tool error on failure**

  In the error branch, append:

  ```rust
  self.world.get_mut::<MessagesComponent>().messages.push(
      crate::types::Message::Tool {
          content: format!("Error: {err}"),
          tool_call_id: r.tool_call_id.clone(),
      },
  );
  ```

- [ ] **Step 3: Run tests**

  Run:
  ```bash
  cargo test --all-targets
  ```
  Expected: all existing tests still pass.

- [ ] **Step 4: Commit**

  ```bash
  git add crates/alice-core/src/effect_executor.rs
  git commit -m "fix(executor): append tool results to MessagesComponent"
  ```

---

### Task 3: Add Unit Test for ExecuteTool Write-Back

**Files:**
- Create: `crates/alice-core/tests/effect_executor_test.rs`

- [ ] **Step 1: Create test file**

  ```rust
  use alice_core::components::{
      ConfigComponent, LoopComponent, MessagesComponent, ProviderComponent, ToolsComponent,
  };
  use alice_core::effect::Effect;
  use alice_core::effect_executor::EffectExecutor;
  use alice_core::event_bus::EventBus;
  use alice_core::providers::StreamingProvider;
  use alice_core::tool_scheduler::ToolScheduler;
  use alice_core::abort_manager::AbortManager;
  use alice_core::types::{Message, ToolDef};
  use alice_core::world::{HasComponent, World};
  use futures_core::Stream;
  use std::pin::Pin;

  #[derive(Default)]
  struct TestComponents { /* ... */ }
  // impl HasComponent<...> for TestComponents ...

  struct NullProvider;
  impl StreamingProvider for NullProvider { /* ... */ }

  #[tokio::test]
  async fn test_execute_tool_appends_tool_message() {
      let mut world = World::new(TestComponents::default());
      let mut event_bus = EventBus::new();
      let tool_scheduler = ToolScheduler::new();
      tool_scheduler.register(
          ToolDef {
              name: "echo".into(),
              description: "echo".into(),
              input_schema: serde_json::json!({
                  "type": "object",
                  "properties": { "message": { "type": "string" } },
                  "required": ["message"]
              }),
          },
          |args| args.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string(),
      );
      let mut abort_manager = AbortManager::new();
      let provider = NullProvider;

      let mut executor = EffectExecutor::new(
          &mut world,
          &mut event_bus,
          &tool_scheduler,
          &mut abort_manager,
          &provider,
      );

      executor.execute(vec![Effect::ExecuteTool {
          tool_name: "echo".into(),
          args: serde_json::json!({ "message": "hello" }),
      }]).await;

      let msgs = &world.get::<MessagesComponent>().messages;
      assert_eq!(msgs.len(), 1);
      assert!(matches!(msgs[0], Message::Tool { .. }));
  }
  ```

- [ ] **Step 2: Run test**

  Run:
  ```bash
  cargo test --test effect_executor_test
  ```
  Expected: PASS.

- [ ] **Step 3: Commit**

  ```bash
  git add crates/alice-core/tests/effect_executor_test.rs
  git commit -m "test(executor): add tool result write-back test"
  ```

---

### Task 4: Extend CLI Integration Test for Step Increment

**Files:**
- Modify: `crates/alice/tests/cli_loop_test.rs`

- [ ] **Step 1: Assert step is incremented**

  At the end of `test_full_loop_with_mock_provider`, add:

  ```rust
  assert_eq!(world.get::<LoopComponent>().step, 1);
  ```

- [ ] **Step 2: Run test**

  Run:
  ```bash
  cargo test --test cli_loop_test
  ```
  Expected: PASS.

- [ ] **Step 3: Commit**

  ```bash
  git add crates/alice/tests/cli_loop_test.rs
  git commit -m "test(cli): assert loop step increments"
  ```

---

### Task 5: Final Verification

- [ ] **Step 1: Run full test suite**

  ```bash
  cargo test --all-targets
  ```
  Expected: all pass.

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

- [ ] **Step 4: Commit any final changes**

  ```bash
  git add -A
  git commit -m "chore: final verification for loop step and tool result fix" || true
  ```
