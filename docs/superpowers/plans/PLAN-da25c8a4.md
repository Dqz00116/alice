---
id: PLAN-da25c8a4
title: Add Configurable Anthropic Base URL
status: approved
---

# Add Configurable Anthropic Base URL

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Allow the Anthropic API base URL to be configured via environment variable while keeping the official default.

**Architecture:** Changes to `alice-core`, `alice-providers`, and `alice` crates.

**Tech Stack:** Rust, Cargo, reqwest, serde_json.

---

### Task 1: Add `base_url` to `ConfigComponent`

**Files:**
- Modify: `crates/alice-core/src/components.rs`

- [ ] **Step 1: Add field and default**

  ```rust
  pub struct ConfigComponent {
      pub model: String,
      pub temperature: f32,
      pub max_steps: u32,
      pub provider: String,
      pub api_key: Option<String>,
      pub base_url: String,
  }
  ```

  Default: `base_url: "https://api.anthropic.com".into()`.

- [ ] **Step 2: Run alice-core tests**

  ```bash
  cargo test -p alice-core
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add -A
  git commit -m "feat(core): add base_url to ConfigComponent"
  ```

---

### Task 2: Make `AnthropicProvider` URL Configurable

**Files:**
- Modify: `crates/alice-providers/src/anthropic.rs`
- Modify: `crates/alice-providers/tests/anthropic_test.rs`
- Modify: `crates/alice-providers/tests/anthropic_live_test.rs`

- [ ] **Step 1: Add `base_url` field and constructor parameter**

  ```rust
  pub struct AnthropicProvider {
      api_key: String,
      model: String,
      base_url: String,
      client: reqwest::Client,
  }

  impl AnthropicProvider {
      pub fn new(api_key: String, model: String, base_url: String) -> Self {
          Self { api_key, model, base_url, client: reqwest::Client::new() }
      }
  }
  ```

- [ ] **Step 2: Use configurable URL in `stream_chat`**

  Replace hardcoded URL with:

  ```rust
  let base_url = self.base_url.trim_end_matches('/');
  let url = format!("{}/v1/messages", base_url);
  let resp = client.post(&url)...
  ```

- [ ] **Step 3: Update existing tests**

  Update all `AnthropicProvider::new(...)` call sites to pass `"https://api.anthropic.com".into()`.

- [ ] **Step 4: Add URL construction unit test**

  Add a test in `crates/alice-providers/tests/anthropic_test.rs`:

  ```rust
  #[test]
  fn test_custom_base_url() {
      let provider = AnthropicProvider::new(
          "fake-key".into(),
          "claude-test".into(),
          "https://api.example.com/anthropic".into(),
      );
      // Use a public getter or exercise stream_chat with a known assertion.
      // Simpler: add `fn base_url(&self) -> &str` to provider and assert it.
  }
  ```

  If exposing a getter is undesirable, the test can verify behavior via the stream URL by asserting the request path does not contain the hardcoded anthropic.com host. Prefer adding a small `base_url()` getter for testability.

- [ ] **Step 5: Run alice-providers tests**

  ```bash
  cargo test -p alice-providers
  ```

- [ ] **Step 6: Commit**

  ```bash
  git add -A
  git commit -m "feat(provider): make Anthropic API base URL configurable"
  ```

---

### Task 3: Wire Base URL Through CLI

**Files:**
- Modify: `crates/alice/src/main.rs`
- Modify: `crates/alice/tests/cli_loop_test.rs`
- Modify: `.env.example`

- [ ] **Step 1: Read `ANTHROPIC_BASE_URL` in main**

  After `dotenvy::dotenv().ok()`:

  ```rust
  let base_url = std::env::var("ANTHROPIC_BASE_URL")
      .filter(|s| !s.is_empty())
      .unwrap_or_else(|| "https://api.anthropic.com".into());
  ```

- [ ] **Step 2: Pass base_url into `ConfigComponent` and provider**

  ```rust
  config: ConfigComponent {
      api_key: api_key.clone(),
      base_url: base_url.clone(),
      ..ConfigComponent::default()
  }
  ```

  ```rust
  let provider = AnthropicProvider::new(
      api_key.unwrap_or_default(),
      world.get::<ConfigComponent>().model.clone(),
      world.get::<ConfigComponent>().base_url.clone(),
  );
  ```

- [ ] **Step 3: Update `.env.example`**

  Add optional line:

  ```bash
  # Optional: override the Anthropic API base URL
  ANTHROPIC_BASE_URL=https://api.anthropic.com
  ```

- [ ] **Step 4: Update CLI loop test components**

  Add `base_url` to any `TestComponents` default and ensure `ConfigComponent` construction includes it.

- [ ] **Step 5: Run tests**

  ```bash
  cargo test --all-targets
  ```

- [ ] **Step 6: Commit**

  ```bash
  git add -A
  git commit -m "feat(cli): read ANTHROPIC_BASE_URL and pass to provider"
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
  git commit -m "chore: final verification for configurable Anthropic base URL" || true
  ```

---

### Task 5: Complete DevFlow

- [ ] Update `.devflow/state.toml` to mark workflow as `finish`.
- [ ] Keep `rust-rewrite` branch independent; do not merge into `main`.
