---
id: REQ-38f782cc
title: Complete Rust Rewrite and Remove TypeScript Implementation
status: approved
priority: high
---

## Description

The project currently maintains two parallel implementations of the Alice engine: a complete TypeScript prototype under `src/` and an in-progress Rust rewrite under `crates/`. The Rust skeleton compiles and has basic tests, but it is not yet a functionally complete replacement for the TS engine. Maintaining both implementations creates duplicated architectural effort and misleading project structure.

This requirement covers two goals:
1. Finish the Rust implementation so that it reaches functional parity with the original TS prototype.
2. Remove the TypeScript implementation and its build tooling from the repository, leaving a single Rust-first codebase.

## Acceptance Criteria

- [ ] Rust `Effect` enum includes the missing `UpdateComponent` variant.
- [ ] Rust `EffectExecutor` implements a real `CallLLM` effect that wires `StreamingProvider` into the event loop.
- [ ] Rust `AnthropicProvider::stream_chat` parses SSE chunks instead of returning placeholder text.
- [ ] Rust `ProviderSystem` reads the messages component and calls the LLM when appropriate.
- [ ] Rust `InputSystem` and `OutputSystem` track the current loop step correctly.
- [ ] Rust `HookSystem` implements the full lifecycle: `beforeStep`, `afterStep`, and `shouldContinue`.
- [ ] Rust defines the missing components: `ConfigComponent`, `LoopComponent`, `ToolsComponent`, and `ProviderComponent`.
- [ ] Rust CLI binary (`crates/alice/src/main.rs`) registers all systems, provider, tools, and runs a stdin/stdout event loop.
- [ ] Rust Middleware trait and pipeline are implemented.
- [ ] DeepSeek provider exists in Rust (or is explicitly dropped with documented rationale).
- [ ] TypeScript source directory `src/` is removed.
- [ ] TypeScript tests `tests/world.test.ts` are removed.
- [ ] TypeScript build configuration files (`tsconfig.json`, `vitest.config.ts`) are removed.
- [ ] `package.json` and `package-lock.json` are removed or stripped of TS-only scripts and dependencies.
- [ ] `AGENTS.md` is updated to describe the project as a single Rust implementation.
- [ ] `cargo test --all-targets`, `cargo clippy --all-targets`, and `cargo build --release` all pass with zero warnings.

## Notes

- Do not introduce runtime mixing of Rust and TS code. The goal is a clean cutover.
- Keep the existing zero-warnings constraint enforced by `.devflow/config.toml`.
- The TS implementation may be referenced during development, but must not remain in the final tree.
- New tests should be added in Rust for every system/effect that is completed.
