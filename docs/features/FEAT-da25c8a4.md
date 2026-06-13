---
id: FEAT-da25c8a4
title: Configurable Anthropic Base URL
status: approved
---

# Configurable Anthropic Base URL

## Feature 1: `base_url` in `ConfigComponent`

- Add `pub base_url: String` to `ConfigComponent`.
- Default value: `"https://api.anthropic.com"`.

## Feature 2: `base_url` in `AnthropicProvider`

- Add `base_url: String` field to `AnthropicProvider`.
- Update `AnthropicProvider::new(api_key, model, base_url)`.
- In `stream_chat`, construct the request URL as:
  ```rust
  format!("{}/v1/messages", self.base_url)
  ```
- Trim trailing slashes from `base_url` before formatting to avoid double slashes.

## Feature 3: CLI Wiring

- In `crates/alice/src/main.rs`, after `dotenvy::dotenv().ok()`:
  - Read `ANTHROPIC_BASE_URL` from environment, falling back to the default.
  - Store it in `ConfigComponent.base_url`.
  - Pass it to `AnthropicProvider::new`.

## Feature 4: `.env.example` Update

- Add optional `ANTHROPIC_BASE_URL=` line to `.env.example`.

## Feature 5: Tests

- Update existing tests that construct `AnthropicProvider` to pass the default base URL.
- Add a unit test for `AnthropicProvider` URL construction with a custom base URL.
- Update `cli_loop_test.rs` and other tests that build `ConfigComponent` to include `base_url`.

## Files Changed

- `crates/alice-core/src/components.rs`
- `crates/alice-providers/src/anthropic.rs`
- `crates/alice/src/main.rs`
- `.env.example`
- Test files in `crates/alice-providers/tests/` and `crates/alice/tests/`
