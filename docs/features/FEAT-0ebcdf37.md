---
id: FEAT-0ebcdf37
title: .env Support and Live LLM API Tests
status: approved
---

# .env Support and Live LLM API Tests

## Feature 1: `.env` File Loading

- Use the `dotenvy` crate in the `alice` binary crate.
- Call `dotenvy::dotenv().ok()` near the start of `main()` before reading `ANTHROPIC_API_KEY` from `std::env`.
- Process environment variables take precedence over `.env` file values (dotenvy default behavior).

## Feature 2: `.env.example` and Ignore Rules

- Add `.env.example` to the workspace root with a documented example:
  ```bash
  ANTHROPIC_API_KEY=sk-ant-api03-...
  ```
- Add `.env` to `.gitignore`.

## Feature 3: Live Anthropic API Integration Test

- Add `crates/alice-providers/tests/anthropic_live_test.rs`.
- The test reads `ANTHROPIC_API_KEY` and is annotated with `#[ignore = "requires ANTHROPIC_API_KEY"]`.
- When run with `cargo test --ignored --test anthropic_live_test`, it constructs `AnthropicProvider`, calls `stream_chat` with a single user message, and collects events.
- Assertions:
  - At least one `LLMStreamEvent::TextDelta` is yielded.
  - The final event is `LLMStreamEvent::StreamEnd`.
- No assertions on exact response text.

## Files Changed

- `crates/alice/Cargo.toml` — add `dotenvy` dependency.
- `crates/alice/src/main.rs` — call `dotenvy::dotenv().ok()` before reading env var.
- `.gitignore` — ignore `.env`.
- `.env.example` — new file.
- `crates/alice-providers/tests/anthropic_live_test.rs` — new live test.
