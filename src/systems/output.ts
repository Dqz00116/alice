import type { System, LLMStreamEvent } from '#src/core/types';

export const OutputSystem: System = (snapshot, event) => {
  const loop = snapshot.getComponent('agent', 'loop');
  const currentStep = loop?.step ?? 0;

  switch (event.type) {
    case 'llm.thinking_delta':
      return [{ type: 'effect.render', content: event.delta, stream: 'thinking' }];
    case 'llm.text_delta':
      return [{ type: 'effect.render', content: event.delta, stream: 'text' }];
    case 'llm.tool_call':
      return [{ type: 'effect.render', content: `[Tool: ${event.tool_call.function.name}]`, stream: 'text' }];
    case 'llm.stream_end':
      return [{ type: 'effect.emit', event: { type: 'system.step_end', step: currentStep } }];
    case 'llm.stream_error':
      return [{ type: 'effect.render', content: `Error: ${event.error}`, stream: 'text' }];
    default:
      return [];
  }
};
