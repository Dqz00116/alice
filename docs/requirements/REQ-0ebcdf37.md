---
id: REQ-0ebcdf37
title: .env Support and Live LLM API Tests
status: approved
---

# .env Support and Live LLM API Tests

## Problem

1. The Alice CLI currently reads `ANTHROPIC_API_KEY` only from the process environment. Developers who want to keep secrets out of shell history must manually `export` the key every session.
2. There are no automated tests that exercise a real LLM provider API. We only have unit tests for SSE parsing and mocked integration tests, so regressions in live HTTP behavior are not caught before shipping.

## Goals

- Load environment variables from a `.env` file at CLI startup, falling back to the process environment.
- Keep `.env` files out of version control.
- Add an opt-in integration test that calls the real Anthropic Messages API when `ANTHROPIC_API_KEY` is available.
- Ensure the live test is skipped by default in normal `cargo test` runs so CI without a key does not fail.

## Non-Goals

- Support multiple providers in live tests (only Anthropic for now).
- Add a full CLI-level live test that reads stdin.
- Validate response content semantically; only verify the stream yields expected event shapes.

## Acceptance Criteria

- [ ] `cargo run -p alice` reads `ANTHROPIC_API_KEY` from a `.env` file in the workspace root.
- [ ] A `.env.example` file documents the required variable.
- [ ] `.env` is listed in `.gitignore`.
- [ ] A new ignored integration test calls `AnthropicProvider::stream_chat` against the real API and asserts at least one `TextDelta` and a final `StreamEnd` event.
- [ ] The new test is marked `#[ignore]` so `cargo test --all-targets` passes without a key.
