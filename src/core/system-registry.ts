import type { System, Event } from '#src/core/types';

export class SystemRegistry {
  private systems: { system: System; eventTypes: string[] }[] = [];

  register(system: System, eventTypes: string[]): void {
    this.systems.push({ system, eventTypes });
  }

  getSystemsForEvent(event: Event): System[] {
    return this.systems
      .filter(({ eventTypes }) => eventTypes.includes(event.type))
      .map(({ system }) => system);
  }
}
