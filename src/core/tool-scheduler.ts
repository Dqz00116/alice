import type { ToolCall, ToolDef, ToolHandler } from '#src/core/types';

export interface ToolResult {
  tool_call_id: string;
  result?: string;
  error?: string;
}

export class ToolScheduler {
  private definitions: ToolDef[] = [];
  private handlers: Record<string, ToolHandler> = {};

  register(def: ToolDef, handler: ToolHandler): void {
    this.definitions.push(def);
    this.handlers[def.name] = handler;
  }

  async schedule(toolCalls: ToolCall[]): Promise<ToolResult[]> {
    // TODO: classify tools as read-only vs write
    // TODO: execute read-only tools concurrently, write tools sequentially
    const results: ToolResult[] = [];
    for (const tc of toolCalls) {
      try {
        const handler = this.handlers[tc.function.name];
        if (!handler) {
          results.push({ tool_call_id: tc.id, error: `unknown tool: ${tc.function.name}` });
          continue;
        }
        const args = JSON.parse(tc.function.arguments) as Record<string, unknown>;
        const result = await handler(args);
        results.push({ tool_call_id: tc.id, result });
      } catch (err) {
        results.push({ tool_call_id: tc.id, error: String(err) });
      }
    }
    return results;
  }
}
