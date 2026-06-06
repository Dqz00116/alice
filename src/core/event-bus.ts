import type { Event } from '#src/core/types';

export type EventHandler = (event: Event) => void;

export class EventBus {
  private subscribers = new Map<string, Set<EventHandler>>();

  subscribe(eventType: string, handler: EventHandler): void {
    let handlers = this.subscribers.get(eventType);
    if (!handlers) {
      handlers = new Set();
      this.subscribers.set(eventType, handlers);
    }
    handlers.add(handler);
  }

  unsubscribe(eventType: string, handler: EventHandler): void {
    this.subscribers.get(eventType)?.delete(handler);
  }

  emit(event: Event): void {
    const handlers = this.subscribers.get(event.type);
    if (handlers) {
      for (const handler of handlers) {
        handler(event);
      }
    }
  }
}
