# Completion Summary — REQ-0ebcdf37

## Feature

.env Support and Live LLM API Tests

## Status

Complete

## Summary

Added `.env` file loading to the Alice CLI and created an opt-in live integration test for the Anthropic provider.

## Key Changes

- Added `dotenvy` dependency to `crates/alice/Cargo.toml`.
- Called `dotenvy::dotenv().ok()` at the start of `crates/alice/src/main.rs`.
- Created `.env.example` documenting `ANTHROPIC_API_KEY`.
- Added `.env` to `.gitignore`.
- Added `crates/alice-providers/tests/anthropic_live_test.rs` with a `#[ignore]` live test.
- Added `tokio` dev-dependency to `alice-providers` for async tests.

## Evidence

- `docs/evidence/EVIDENCE-0ebcdf37.md` (to be added if required)

## Verification

- `cargo test --all-targets` — PASS (live test ignored by default)
- `cargo clippy --all-targets` — PASS (zero warnings)
- `cargo build --release` — PASS

## Known Limitations

- Live test only covers Anthropic; other providers are not exercised.
- No CLI-level live test that reads stdin.
