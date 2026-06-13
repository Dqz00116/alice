---
id: FEAT-82c3af0d
title: Configure Anthropic Model ID via .env
status: approved
---

# Configure Anthropic Model ID via .env

## Feature 1: Read `ANTHROPIC_MODEL` in CLI

- After `dotenvy::dotenv().ok()`, read `ANTHROPIC_MODEL` from the environment.
- Fall back to the existing default (`claude-3-5-sonnet-20241022`) when not set or empty.
- Store the value in `ConfigComponent.model`.

## Feature 2: Use Configured Model in Provider

- Construct `AnthropicProvider` with `world.get::<ConfigComponent>().model.clone()`.
- Remove any remaining hardcoded model IDs from CLI provider construction.

## Feature 3: Update `.env.example`

- Add optional `ANTHROPIC_MODEL=` line to `.env.example`.

## Feature 4: Tests

- Update live Anthropic test to use the configured model (or keep a fixed default if simpler).
- Add a unit test asserting that a custom model ID propagates to the provider.
- Update any tests that inspect `ConfigComponent.model`.

## Files Changed

- `crates/alice/src/main.rs`
- `crates/alice-providers/src/anthropic.rs` (add model getter if needed for tests)
- `crates/alice-providers/tests/anthropic_test.rs`
- `crates/alice-providers/tests/anthropic_live_test.rs`
- `.env.example`
