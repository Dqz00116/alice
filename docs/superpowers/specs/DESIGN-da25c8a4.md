---
id: DESIGN-da25c8a4
title: Configurable Anthropic Base URL
status: approved
---

# Configurable Anthropic Base URL

## Architecture

The configuration flows from `.env` / shell environment into the CLI binary, then into `ConfigComponent`, and finally into `AnthropicProvider` at construction time.

```
.env / env
   │
   │ ANTHROPIC_BASE_URL
   ▼
main.rs
   │
   │ ConfigComponent { base_url, ... }
   ▼
AnthropicProvider::new(api_key, model, base_url)
   │
   │ format!("{}/v1/messages", base_url)
   ▼
HTTP request
```

## Component Changes

### `ConfigComponent`

- Add `base_url: String`.
- Default: `"https://api.anthropic.com"`.

### `AnthropicProvider`

- Add `base_url: String`.
- Constructor signature: `new(api_key: String, model: String, base_url: String)`.
- In `stream_chat`, replace hardcoded URL with formatted URL.
- Normalize `base_url` by trimming trailing `/` to avoid `https://host//v1/messages`.

### CLI

- After `dotenvy::dotenv().ok()`:
  ```rust
  let base_url = std::env::var("ANTHROPIC_BASE_URL")
      .unwrap_or_else(|_| "https://api.anthropic.com".into());
  ```
- Store in `ConfigComponent.base_url`.
- Pass to provider constructor.

## Error Handling

- If `ANTHROPIC_BASE_URL` is missing or empty, use the official Anthropic default.
- No validation of URL format at this stage; invalid URLs will surface as HTTP errors from `reqwest`.

## Testing

- Unit test for URL construction with custom base URL.
- Update all existing provider tests to compile with new constructor.
- Full test suite, clippy, and release build must pass.
