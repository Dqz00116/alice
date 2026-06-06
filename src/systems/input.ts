import type { System } from '#src/core/types';

export const InputSystem: System = (snapshot, event) => {
  if (event.type !== 'input.user') return [];

  const loop = snapshot.getComponent('agent', 'loop');
  const currentStep = loop?.step ?? 0;

  return [
    {
      type: 'effect.appendMessage',
      entity: 'agent',
      message: { role: 'user', content: event.content },
    },
    {
      type: 'effect.emit',
      event: { type: 'system.step_start', step: currentStep },
    },
  ];
};
