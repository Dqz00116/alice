# Completion Summary — REQ-3bc87e1a

## Feature

Fix Loop Step Tracking and Tool Result Message Write-Back

## Status

Complete

## Summary

Fixed two core correctness issues in the Rust engine's main loop:

1. `LoopComponent.step` is now incremented after each completed LLM stream (`StreamEnd`), enabling `max_steps` termination and correct `StepStart`/`StepEnd` reporting.
2. `EffectExecutor::ExecuteTool` now appends tool results (and errors) to `MessagesComponent` as `Message::Tool`, allowing the LLM to observe tool output in subsequent turns.

## Key Changes

- `crates/alice-core/src/effect_executor.rs`
  - Increment `LoopComponent.step` on `StreamEnd`.
  - Append `Message::Tool` on tool success and error.
- `crates/alice-core/tests/effect_executor_test.rs`
  - New tests for step increment and tool result write-back.
- `crates/alice/tests/cli_loop_test.rs`
  - Assert step is incremented after a full loop turn.

## Evidence

- `docs/evidence/EVIDENCE-3bc87e1a.md`

## Verification

- `cargo test --all-targets` — PASS (11 tests)
- `cargo clippy --all-targets` — PASS (zero warnings)
- `cargo build --release` — PASS
