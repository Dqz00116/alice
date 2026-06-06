import type { ToolsComponent, ToolDef, ToolHandler } from '#src/core/types';

export function createToolsComponent(
  definitions: ToolDef[] = [],
  handlers: Record<string, ToolHandler> = {},
): ToolsComponent {
  return { definitions, handlers };
}
