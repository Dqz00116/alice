---
id: REQ-82c3af0d
title: Configure Anthropic Model ID via .env
status: approved
---

# Configure Anthropic Model ID via .env

## Problem

`ConfigComponent` already has a `model` field, but the CLI currently ignores it and hardcodes the model ID when constructing `AnthropicProvider`. Users who want to switch models (for example to `claude-3-opus-20240229` or `claude-3-5-sonnet-20240620`) must edit source code.

## Goals

- Allow the Anthropic model ID to be configured via the `ANTHROPIC_MODEL` environment variable so it can be placed in `.env`.
- Keep the existing default model when no override is provided.
- Ensure `.env` loading works the same way as `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL`.

## Non-Goals

- Add a command-line flag for model selection.
- Validate that the model ID is supported by Anthropic.
- Support per-request model changes.

## Acceptance Criteria

- [ ] CLI reads `ANTHROPIC_MODEL` from the environment (after `dotenvy` loads `.env`) and stores it in `ConfigComponent.model`.
- [ ] `AnthropicProvider` is constructed using `world.get::<ConfigComponent>().model` instead of a hardcoded string.
- [ ] `.env.example` documents `ANTHROPIC_MODEL`.
- [ ] Existing tests still pass and at least one new test verifies model ID propagation.
- [ ] `cargo test --all-targets`, `cargo clippy --all-targets`, and `cargo build --release` all pass.
