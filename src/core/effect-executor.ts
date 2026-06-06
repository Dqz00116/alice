import type { Effect, World, ToolCall, ComponentDataMap } from '#src/core/types';
import type { EventBus } from '#src/core/event-bus';
import type { ToolScheduler } from '#src/core/tool-scheduler';
import type { AbortManager } from '#src/core/abort-manager';

export class EffectExecutor {
  constructor(
    private world: World,
    private eventBus: EventBus,
    private toolScheduler: ToolScheduler,
    private abortManager: AbortManager,
  ) {}

  async execute(effects: Effect[]): Promise<void> {
    for (const effect of effects) {
      await this.apply(effect);
    }
  }

  private async apply(effect: Effect): Promise<void> {
    switch (effect.type) {
      case 'effect.callLLM': {
        const provider = this.world.getComponent(effect.provider, 'provider');
        if (!provider) break;
        try {
          const body = provider.formatMessages(effect.messages);
          const stream = provider.streamChat(body);
          for await (const event of stream) {
            this.eventBus.emit(event);
          }
        } catch (err) {
          this.eventBus.emit({ type: 'llm.stream_error', error: String(err) });
        }
        break;
      }
      case 'effect.executeTool': {
        const toolCall: ToolCall = {
          id: `tool_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`,
          type: 'function',
          function: {
            name: effect.tool_name,
            arguments: JSON.stringify(effect.args),
          },
        };
        const results = await this.toolScheduler.schedule([toolCall]);
        const toolResult = results[0];
        if (!toolResult) break;
        if (toolResult.error) {
          this.eventBus.emit({ type: 'tool.error', tool_call_id: toolResult.tool_call_id, error: toolResult.error });
        } else {
          this.eventBus.emit({ type: 'tool.result', tool_call_id: toolResult.tool_call_id, result: toolResult.result! });
        }
        break;
      }
      case 'effect.appendMessage': {
        const comp = this.world.getComponent(effect.entity, 'messages');
        if (comp) {
          comp.messages.push(effect.message);
        }
        break;
      }
      case 'effect.updateComponent':
        this.world.setComponent(
          effect.entity,
          effect.component as keyof ComponentDataMap,
          effect.data as ComponentDataMap[keyof ComponentDataMap],
        );
        break;
      case 'effect.emit':
        this.eventBus.emit(effect.event);
        break;
      case 'effect.render':
        if (effect.stream === 'thinking') {
          process.stdout.write(`\x1b[2m${effect.content}\x1b[0m`);
        } else {
          process.stdout.write(effect.content);
        }
        break;
      case 'effect.abort':
        this.abortManager.abort(effect.reason);
        break;
      default: {
        const _exhaustive: never = effect;
        break;
      }
    }
  }
}
