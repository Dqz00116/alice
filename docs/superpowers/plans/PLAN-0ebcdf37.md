---
id: PLAN-0ebcdf37
title: Add .env Support and Live LLM API Tests
status: approved
---

# Add .env Support and Live LLM API Tests

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Load environment variables from `.env` at CLI startup and add an opt-in live Anthropic API integration test.

**Architecture:** Focused changes to `crates/alice` and `crates/alice-providers`.

**Tech Stack:** Rust, Cargo, dotenvy, reqwest, serde_json.

---

### Task 1: Add `.env` Support to CLI

**Files:**
- Modify: `crates/alice/Cargo.toml`
- Modify: `crates/alice/src/main.rs`
- Modify: `.gitignore`
- Create: `.env.example`

- [ ] **Step 1: Add `dotenvy` dependency**

  In `crates/alice/Cargo.toml`:

  ```toml
  dotenvy = "0.15"
  ```

- [ ] **Step 2: Load `.env` in main**

  In `crates/alice/src/main.rs`, as the first statement inside `main()`:

  ```rust
  dotenvy::dotenv().ok();
  ```

- [ ] **Step 3: Add `.env.example`**

  Create `.env.example` in the workspace root:

  ```bash
  # Copy to .env and fill in your key
  ANTHROPIC_API_KEY=sk-ant-api03-...
  ```

- [ ] **Step 4: Ignore `.env`**

  Add `.env` to `.gitignore`.

- [ ] **Step 5: Verify build**

  ```bash
  cargo build -p alice
  ```

- [ ] **Step 6: Commit**

  ```bash
  git add -A
  git commit -m "feat(cli): load environment from .env file"
  ```

---

### Task 2: Add Live Anthropic API Integration Test

**Files:**
- Create: `crates/alice-providers/tests/anthropic_live_test.rs`

- [ ] **Step 1: Create live test file**

  ```rust
  use alice_core::event::LLMStreamEvent;
  use alice_core::types::Message;
  use alice_providers::anthropic::AnthropicProvider;
  use futures_util::StreamExt;

  #[tokio::test]
  #[ignore = "requires ANTHROPIC_API_KEY"]
  async fn test_anthropic_live_stream_chat() {
      let api_key = std::env::var("ANTHROPIC_API_KEY")
          .expect("ANTHROPIC_API_KEY must be set to run this test");

      let provider = AnthropicProvider::new(
          api_key,
          "claude-3-5-sonnet-20241022".into(),
      );

      let body = provider.format_messages(&[Message::User {
          content: "Say hello in one word.".into(),
      }]);

      let mut stream = provider.stream_chat(body);
      let mut events = Vec::new();
      while let Some(event) = stream.next().await {
          events.push(event);
      }

      assert!(
          events.iter().any(|e| matches!(e, LLMStreamEvent::TextDelta { .. })),
          "expected at least one text delta"
      );
      assert!(
          matches!(events.last(), Some(LLMStreamEvent::StreamEnd { .. })),
          "expected stream to end"
      );
  }
  ```

- [ ] **Step 2: Verify ignored by default**

  ```bash
  cargo test --all-targets
  ```

  The live test should be listed as `ignored`.

- [ ] **Step 3: Run live test with key (optional manual verification)**

  ```bash
  cargo test --ignored --test anthropic_live_test
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add -A
  git commit -m "test(provider): add opt-in live Anthropic API integration test"
  ```

---

### Task 3: Final Verification

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
  git commit -m "chore: final verification for .env support and live API tests" || true
  ```

---

### Task 4: Complete DevFlow

- [ ] Update `.devflow/state.toml` to mark workflow as `finish`.
- [ ] Keep `rust-rewrite` branch independent; do not merge into `main`.
