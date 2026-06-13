# Pattern Analysis: Tool Definition Discovery vs. Execution

## Expected Working Pattern

In an LLM agent architecture, the set of tools advertised to the model should match
the set of tools the runtime can actually execute.

```textnToolScheduler (execution)
    └─ definitions + handlers for all tools

ToolsComponent / Provider (discovery)
    └─ definitions advertised to LLM

Requirement: advertised_tools ⊆ executable_tools
Ideal:     advertised_tools == executable_tools
```

When the two sets differ:

- **Advertised > executable**: Model calls a tool that has no handler → runtime error.
- **Advertised < executable**: Model never learns about useful tools → under-calls them.

The current bug is the second case.

## Alice's Pattern

- `ToolScheduler` is the execution registry. It owns both `definitions` and `handlers`.
- `ToolsComponent` is the discovery registry used by the provider to format API requests.
- `main.rs` currently maintains two independent lists:
  1. Tools passed to `ToolScheduler::register`.
  2. Tools placed into `ToolsComponent::definitions`.

## Reference Fix Pattern

Keep the two registries in sync by populating `ToolsComponent.definitions` with the
same `ToolDef` instances that are registered in the scheduler. A minimal fix is to
update the `AllComponents` initialization in `main.rs`.

A more robust long-term pattern would be to initialize `ToolsComponent` from the
`ToolScheduler` definitions after registration (or to share a single registry). For
this focused fix, synchronizing the initialization list is sufficient.

## Conclusion

The execution side already supports all six tools. The discovery side is incomplete.
The fix should update `ToolsComponent.definitions` in `crates/alice/src/main.rs` to
include `bash`, `read_file`, `write_file`, `edit_file`, and `glob` definitions.
