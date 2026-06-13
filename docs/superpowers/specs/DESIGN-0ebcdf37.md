---
id: DESIGN-0ebcdf37
title: .env Support and Live LLM API Tests
status: approved
---

# .env Support and Live LLM API Tests

## Architecture

### `.env` Loading

The `alice` binary is the only process entry point, so `.env` loading belongs there and not in `alice-core` or `alice-providers`.

```
workspace-root/.env
       │
       │ dotenvy::dotenv().ok()
       ▼
   process env
       │
       │ std::env::var("ANTHROPIC_API_KEY")
       ▼
   AnthropicProvider::new(...)
```

`dotenvy::dotenv()` returns an error when the file is missing; we intentionally call `.ok()` so the CLI works with only shell-exported environment variables.

### Live Test Design

Live tests are isolated in their own test file so they can be:

- Ignored by default (`#[ignore]`).
- Run independently with `cargo test --ignored --test anthropic_live_test`.

The test uses the existing `AnthropicProvider` and exercises `stream_chat` end-to-end over HTTP. It validates event shape, not content, to avoid flakiness from model wording changes.

## Error Handling

- Missing `.env`: silently continue (use shell env).
- Missing `ANTHROPIC_API_KEY` in live test: the test is ignored by default; if explicitly run without the key, it should fail with a clear message.

## Testing Strategy

1. Unit / mock tests remain unchanged.
2. Run `cargo test --all-targets` to confirm ignored test does not execute.
3. Run `cargo test --ignored --test anthropic_live_test` with a real key to confirm live behavior.
4. Run `cargo clippy --all-targets` and `cargo build --release`.
