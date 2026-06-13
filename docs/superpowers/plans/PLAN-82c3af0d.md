---
id: PLAN-82c3af0d
title: Configure Anthropic Model ID via .env
status: approved
---

# Configure Anthropic Model ID via .env

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Allow the Anthropic model ID to be set via `ANTHROPIC_MODEL` in `.env`.

**Architecture:** Changes to `alice` and `alice-providers` crates.

**Tech Stack:** Rust, Cargo, dotenvy.

---

### Task 1: Read `ANTHROPIC_MODEL` in CLI and Store in ConfigComponent

**Files:**
- Modify: `crates/alice/src/main.rs`

- [ ] **Step 1: Read environment variable**

  After `dotenvy::dotenv().ok()`:

  ```rust
  let model = match std::env::var("ANTHROPIC_MODEL") {
      Ok(m) if !m.is_empty() => m,
      _ => "claude-3-5-sonnet-20241022".into(),
  };
  ```

- [ ] **Step 2: Store in ConfigComponent**

  ```rust
  config: ConfigComponent {
      api_key: api_key.clone(),
      base_url: base_url.clone(),
      model: model.clone(),
      ..ConfigComponent::default()
  }
  ```

- [ ] **Step 3: Pass model to provider**

  ```rust
  let provider = AnthropicProvider::new(
      api_key.unwrap_or_default(),
      world.get::<ConfigComponent>().model.clone(),
      world.get::<ConfigComponent>().base_url.clone(),
  );
  ```

- [ ] **Step 4: Run tests**

  ```bash
  cargo test -p alice
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add -A
  git commit -m "feat(cli): read ANTHROPIC_MODEL from env into ConfigComponent"
  ```

---

### Task 2: Add Model Getter and Update Tests

**Files:**
- Modify: `crates/alice-providers/src/anthropic.rs`
- Modify: `crates/alice-providers/tests/anthropic_test.rs`
- Modify: `crates/alice-providers/tests/anthropic_live_test.rs`

- [ ] **Step 1: Add `model()` getter**

  ```rust
  pub fn model(&self) -> &str {
      &self.model
  }
  ```

- [ ] **Step 2: Add model propagation unit test**

  ```rust
  #[test]
  fn test_custom_model_id() {
      let provider = AnthropicProvider::new(
          "fake-key".into(),
          "claude-test-model".into(),
          "https://api.anthropic.com".into(),
      );
      assert_eq!(provider.model(), "claude-test-model");
  }
  ```

- [ ] **Step 3: Update live test if needed**

  Ensure `AnthropicProvider::new` receives a model string (already does).

- [ ] **Step 4: Run tests**

  ```bash
  cargo test -p alice-providers
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add -A
  git commit -m "test(provider): add model getter and custom model test"
  ```

---

### Task 3: Update `.env.example`

**Files:**
- Modify: `.env.example`

- [ ] **Step 1: Add model line**

  ```bash
  # Optional: override the Anthropic model ID
  ANTHROPIC_MODEL=claude-3-5-sonnet-20241022
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add -A
  git commit -m "docs: document ANTHROPIC_MODEL in .env.example"
  ```

---

### Task 4: Final Verification

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
  git commit -m "chore: final verification for configurable Anthropic model ID" || true
  ```

---

### Task 5: Complete DevFlow

- [ ] Update `.devflow/state.toml` to mark workflow as `finish`.
- [ ] Keep `rust-rewrite` branch independent; do not merge into `main`.
