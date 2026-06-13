---
id: DESIGN-82c3af0d
title: Configure Anthropic Model ID via .env
status: approved
---

# Configure Anthropic Model ID via .env

## Architecture

Configuration flows from `.env` / shell environment into the CLI binary, then into `ConfigComponent`, and finally into `AnthropicProvider` at construction time.

```
.env / env
   │
   │ ANTHROPIC_MODEL
   ▼
main.rs
   │
   │ ConfigComponent { model, ... }
   ▼
AnthropicProvider::new(api_key, model, base_url)
```

## Component Changes

### CLI

- After `dotenvy::dotenv().ok()`:
  ```rust
  let model = std::env::var("ANTHROPIC_MODEL")
      .filter(|s| !s.is_empty())
      .unwrap_or_else(|| "claude-3-5-sonnet-20241022".into());
  ```
- Store `model` in `ConfigComponent.model`.
- Pass `world.get::<ConfigComponent>().model.clone()` to `AnthropicProvider::new`.

### `AnthropicProvider`

- Already stores `model` and uses it in `format_messages`. No changes needed beyond possibly adding a `model()` getter for tests.

## Error Handling

- Missing or empty `ANTHROPIC_MODEL` falls back to the default model.
- Invalid model IDs surface as API errors from Anthropic.

## Testing

- Add unit test verifying custom model ID is stored in the provider.
- Update live test to construct provider with a configurable model.
- Full test suite, clippy, and release build must pass.
