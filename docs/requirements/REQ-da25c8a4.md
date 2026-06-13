---
id: REQ-da25c8a4
title: Configurable Anthropic Base URL
status: done
---

# Configurable Anthropic Base URL

## Problem

`AnthropicProvider` hardcodes `https://api.anthropic.com/v1/messages`. Users who need to route requests through a proxy or a compatible endpoint (for example `https://api.deepseek.com/anthropic`) cannot do so without editing source code.

## Goals

- Allow the Anthropic API base URL to be configured at runtime.
- Support reading the URL from an environment variable (`ANTHROPIC_BASE_URL`) so it can be placed in `.env`.
- Keep the existing default (`https://api.anthropic.com`) when no override is provided.
- Ensure `.env` support added previously continues to work without changes.

## Non-Goals

- Support per-request base URL changes after the provider is constructed.
- Add a command-line flag for the base URL.
- Implement a DeepSeek-specific provider or adapter.

## Acceptance Criteria

- [ ] `AnthropicProvider` accepts a `base_url` parameter and builds the request URL as `{base_url}/v1/messages`.
- [ ] `ConfigComponent` stores a `base_url` field with a sensible default.
- [ ] CLI reads `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL` from the environment (after `dotenvy` loads `.env`) and passes them to the provider.
- [ ] Existing tests still pass and at least one new test verifies custom base URL propagation.
- [ ] `cargo test --all-targets`, `cargo clippy --all-targets`, and `cargo build --release` all pass.
