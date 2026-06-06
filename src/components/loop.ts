import type { LoopComponent } from '#src/core/types';

export function createLoopComponent(): LoopComponent {
  return {
    step: 0,
    shouldContinue: true,
  };
}
