import { AliceWorld } from '#src/core/world';
import { EventBus } from '#src/core/event-bus';
import { EffectExecutor } from '#src/core/effect-executor';
import { SystemRegistry } from '#src/core/system-registry';
import { ToolScheduler } from '#src/core/tool-scheduler';
import { AbortManager } from '#src/core/abort-manager';
import { MiddlewarePipeline } from '#src/middleware/types';
import { InputSystem } from '#src/systems/input';
import { ProviderSystem } from '#src/systems/provider';
import { ToolSystem } from '#src/systems/tool';
import { OutputSystem } from '#src/systems/output';
import { HookSystem } from '#src/systems/hook';
import { createMessagesComponent } from '#src/components/messages';
import { createProviderComponent } from '#src/components/provider';
import { createToolsComponent } from '#src/components/tools';
import { createConfigComponent } from '#src/components/config';
import { createLoopComponent } from '#src/components/loop';
import type { Event } from '#src/core/types';

// ── Host: Assemble World ──
const world = new AliceWorld();
world.setComponent('agent', 'messages', createMessagesComponent());
world.setComponent('agent', 'provider', createProviderComponent({
  formatMessages: (msgs) => ({ messages: msgs }),
  streamChat: async function* () { yield { type: 'llm.stream_end', stop_reason: 'placeholder' }; },
}));
world.setComponent('agent', 'tools', createToolsComponent());
world.setComponent('agent', 'config', createConfigComponent());
world.setComponent('agent', 'loop', createLoopComponent());

// ── Host: Wire Core Infrastructure ──
const eventBus = new EventBus();
const toolScheduler = new ToolScheduler();
const abortManager = new AbortManager();
const effectExecutor = new EffectExecutor(world, eventBus, toolScheduler, abortManager);
const systemRegistry = new SystemRegistry();
const middleware = new MiddlewarePipeline();

// ── Host: Register Systems (system → event types) ──
systemRegistry.register(InputSystem, ['input.user']);
systemRegistry.register(ProviderSystem, ['system.step_start', 'tool.result']);
systemRegistry.register(ToolSystem, ['llm.tool_call']);
systemRegistry.register(OutputSystem, ['llm.thinking_delta', 'llm.text_delta', 'llm.tool_call', 'llm.stream_end', 'llm.stream_error']);
systemRegistry.register(HookSystem, ['system.hook_trigger']);

// ── Host: Event dispatch (EventBus → Middleware → SystemRegistry → EffectExecutor) ──
const dispatchEvent = (event: Event): void => {
  middleware.run(event, () => {
    const systems = systemRegistry.getSystemsForEvent(event);
    for (const system of systems) {
      const snapshot = world.createSnapshot();
      const effects = system(snapshot, event);
      // Fire-and-forget: let effects execute asynchronously
      effectExecutor.execute(effects);
    }
  });
};

// Subscribe all event types
const allEventTypes = [
  'input.user',
  'system.step_start',
  'tool.result',
  'llm.tool_call',
  'llm.thinking_delta',
  'llm.text_delta',
  'llm.stream_end',
  'llm.stream_error',
  'system.step_end',
  'system.hook_trigger',
];

for (const eventType of allEventTypes) {
  eventBus.subscribe(eventType, dispatchEvent);
}

// ── Host: Start loop ──
console.log('Alice Agent ready.');
eventBus.emit({ type: 'system.hook_trigger', hook: 'beforeStep' });
