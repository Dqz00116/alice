import type { System } from '#src/core/types';

export const ToolSystem: System = (snapshot, event) => {
  if (event.type !== 'llm.tool_call') return [];

  const args = JSON.parse(event.tool_call.function.arguments) as Record<string, unknown>;

  return [
    {
      type: 'effect.executeTool',
      tool_name: event.tool_call.function.name,
      args,
    },
  ];
};
