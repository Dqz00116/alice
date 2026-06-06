import type { ConfigComponent } from '#src/core/types';

export function createConfigComponent(overrides: Partial<ConfigComponent> = {}): ConfigComponent {
  return {
    maxSteps: overrides.maxSteps ?? 10,
    temperature: overrides.temperature ?? 0.7,
    model: overrides.model ?? 'default',
  };
}
