import type { System } from '#src/core/types';

export const ProviderSystem: System = (snapshot, event) => {
  if (event.type !== 'system.step_start' && event.type !== 'tool.result') return [];

  const messages = snapshot.getComponent('agent', 'messages');
  if (!messages) return [];

  const loop = snapshot.getComponent('agent', 'loop');
  if (loop && !loop.shouldContinue) return [];

  return [
    {
      type: 'effect.callLLM',
      provider: 'agent',
      messages: messages.messages,
    },
  ];
};
