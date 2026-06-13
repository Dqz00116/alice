---
id: FEAT-38f782cc
title: Rust-First Completion and TypeScript Removal
status: approved
---

## Overview

Finish the Rust rewrite of the Alice engine and remove the legacy TypeScript prototype, resulting in a single Rust-first codebase.

## Technical Approach

1. **Complete Rust core parity**
   - Add the missing `UpdateComponent` effect variant.
   - Implement real `CallLLM` in `EffectExecutor` by delegating to a configured `StreamingProvider`.
   - Parse SSE streams in `AnthropicProvider::stream_chat` and emit real `Event::LLM` deltas.
   - Fill in `ProviderSystem`, `InputSystem`, `OutputSystem`, and `HookSystem` so the main event loop functions.
   - Introduce `ConfigComponent`, `LoopComponent`, `ToolsComponent`, and `ProviderComponent` as needed by the systems.

2. **CLI event loop**
   - Wire all systems into `SystemRegistry`.
   - Register the echo tool and any available provider.
   - Read user input from stdin, emit `input.user`, and drive the loop until `shouldContinue` is false.

3. **Middleware and DeepSeek**
   - Implement a middleware trait and onion-style pipeline equivalent to the TS version.
   - Either port the DeepSeek provider or document its removal if it is not required for the first milestone.

4. **Remove TypeScript**
   - Delete `src/` and `tests/world.test.ts`.
   - Delete `tsconfig.json` and `vitest.config.ts`.
   - Remove `package.json` and `package-lock.json` (no runtime TS dependencies remain).
   - Update `AGENTS.md` to describe the Rust-only architecture.

## Architecture

The final tree mirrors the existing 4-crate workspace:

- `crates/alice-core` — EventBus, World, Systems, EffectExecutor, ToolScheduler, AbortManager.
- `crates/alice-providers` — `StreamingProvider` trait, Anthropic (and optionally DeepSeek) provider.
- `crates/alice-tools` — Built-in tools such as `echo`.
- `crates/alice` — CLI binary that assembles and runs the engine.

## Constraints

- `cargo clippy --all-targets` must remain at zero warnings.
- Every completed system/effect must have a corresponding Rust test.
- No TS artifacts remain in the repository root.
