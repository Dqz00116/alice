# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Alice is a data-driven, stateless, streaming-concurrent, highly customizable CLI AI Agent engine written in TypeScript. It provides a framework for orchestrating LLM interactions and tool calls via a pure-function System + Effect architecture. No runtime dependencies — the engine is self-contained with only dev tooling.

## Commands

```bash
npm run dev          # Run the engine entry point (tsx src/index.ts)
npm test             # Run all tests (vitest run)
npm run typecheck    # Type-check without emitting (tsc --noEmit)
```

To run a single test file: `npx vitest run tests/<file>.test.ts`
To run a single test by name: `npx vitest run -t "<test name>"`

## Architecture

### Five-layer model (top-down)

| Layer | Role | Key files |
|---|---|---|
| **Host** (assembly) | Initialize World, register Systems, configure Provider, install Middleware | `src/index.ts` |
| **Core** (infrastructure) | EventBus, EffectExecutor, SystemRegistry, ToolScheduler, AbortManager, World/Snapshot | `src/core/*.ts` |
| **Business** (built-in Systems) | InputSystem, ProviderSystem, ToolSystem, OutputSystem, HookSystem | `src/systems/*.ts` |
| **Data** (pure data components) | Messages, Provider, Tools, Config, Loop — data containers with no behavior | `src/components/*.ts` |
| **Infrastructure** (protocol adapters) | StreamingProvider interface, ToolAccesses, Middleware pipeline | `src/providers/types.ts`, `src/middleware/types.ts` |

### Core abstractions

- **World** (`src/core/world.ts`): ECS-style data store — the single source of truth. Components are stored by `(entity, componentType)` key pairs. `createSnapshot()` produces an immutable read-only view for Systems.

- **System** (`src/core/types.ts:98`): Pure function `(WorldSnapshot, Event) => Effect[]`. Stateless — same inputs always produce same outputs. Systems are registered against event types they handle.

- **Event** (`src/core/types.ts:21-44`): Four categories — `InputEvent` (user input), `LLMStreamEvent` (thinking/text deltas, tool calls, stream end/error), `ToolEvent` (tool requests/results/errors), `SystemEvent` (step start/end, hook triggers).

- **Effect** (`src/core/types.ts:47-95`): Seven variants describing side effects — `callLLM`, `executeTool`, `appendMessage`, `updateComponent`, `emit`, `render`, `abort`. Effects are descriptions, not actions; `EffectExecutor` interprets and applies them.

- **EventBus** (`src/core/event-bus.ts`): Publish-subscribe event router. Systems subscribe by event type string.

- **MiddlewarePipeline** (`src/middleware/types.ts`): Chain-of-responsibility intercepting events before they reach Systems. Suitable for logging, auth, rate limiting.

### Data flow (the main loop)

```
User input → InputSystem (appendMessage + emit step_start)
  → ProviderSystem (callLLM stream)
    → OutputSystem (render tokens as they arrive)
    → ToolSystem (executeTool on tool_call)
      → ToolScheduler → tool.result event
        → ProviderSystem (next LLM call with tool result)
          → ... loop until shouldContinue=false
```

HookSystem drives step lifecycle: `beforeStep` (increment step) → Systems run → `afterStep` (emit `shouldContinue` check) → `shouldContinue` (update loop component if step < maxSteps).

### Extension points (4 levels, increasing flexibility)

1. **Middleware** — intercept/modify/drop events in the pipeline
2. **Hook** — lifecycle callbacks (`beforeStep`, `afterStep`, `beforeToolCall`, `afterToolCall`, `shouldContinue`)
3. **Custom System** — register additional Systems for new event types or override built-in behavior
4. **Custom Provider** — implement `StreamingProvider` interface (`formatMessages` + `streamChat`) to connect any LLM

### Provider implementation

Providers implement `StreamingProvider` (`src/providers/types.ts`): `formatMessages(messages) → body` and `async *streamChat(body) → AsyncGenerator<LLMStreamEvent>`. The template is at `src/providers/_template.ts`. Current provider files (`anthropic.ts`, `deepseek.ts`) are stubs re-exporting the template.

### Tool implementation

Tools follow the pattern in `src/tools/_template.ts`: a `ToolDef` (name, description, input_schema) paired with a `ToolHandler` (`async (args) => string`). Register via `ToolScheduler.register(def, handler)` and add to the ToolsComponent definitions/handlers maps.

### Import aliases

The project uses `#src/*` import aliases defined in `package.json` → `imports`. These resolve to `./src/*.ts` or `./src/*/index.ts`. TypeScript is configured with `moduleResolution: "bundler"` and `resolvePackageJsonImports: true` to support this.

## Key design decisions

- **Why pure-function Systems?** Determinism and testability. Same (snapshot, event) → same Effects. No hidden state, no side-effect interference.
- **Why EventBus + Effect instead of a state machine?** LLM interaction control flow is highly non-deterministic. Event-driven dispatch + declarative effects decouples routing from execution.
- **Why single-threaded async?** Node.js event loop eliminates lock/race issues for I/O-bound workloads. Read-write conflict detection for tools is a simple task queue, no complex locking needed.
- **Snapshots are shared-reference, not deep-copied.** Systems must treat snapshot data as read-only by convention. The test at `tests/world.test.ts:48-59` explicitly documents this tradeoff.
