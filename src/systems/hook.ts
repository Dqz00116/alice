import type { System } from '#src/core/types';

export const HookSystem: System = (snapshot, event) => {
  if (event.type !== 'system.hook_trigger') return [];

  const loop = snapshot.getComponent('agent', 'loop');
  if (!loop) return [];

  switch (event.hook) {
    case 'shouldContinue':
      return [
        {
          type: 'effect.updateComponent',
          entity: 'agent',
          component: 'loop',
          data: { ...loop, shouldContinue: loop.step < (snapshot.getComponent('agent', 'config')?.maxSteps ?? 10) },
        },
      ];
    case 'beforeStep':
      return [
        {
          type: 'effect.updateComponent',
          entity: 'agent',
          component: 'loop',
          data: { ...loop, step: loop.step + 1 },
        },
      ];
    case 'afterStep':
      return [
        {
          type: 'effect.emit',
          event: { type: 'system.hook_trigger', hook: 'shouldContinue' },
        },
      ];
    default:
      return [];
  }
};
