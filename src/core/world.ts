import type { ComponentDataMap, ComponentType, EntityId, World, WorldSnapshot } from '#src/core/types';

class Snapshot implements WorldSnapshot {
  constructor(private components: Map<EntityId, Map<ComponentType, unknown>>) {}

  getComponent<T extends keyof ComponentDataMap>(
    entity: EntityId,
    type: T,
  ): ComponentDataMap[T] | undefined {
    const entityComponents = this.components.get(entity);
    if (!entityComponents) return undefined;
    return entityComponents.get(type) as ComponentDataMap[T] | undefined;
  }
}

export class AliceWorld implements World {
  private components = new Map<EntityId, Map<ComponentType, unknown>>();

  setComponent<T extends keyof ComponentDataMap>(
    entity: EntityId,
    type: T,
    data: ComponentDataMap[T],
  ): void {
    let entityComponents = this.components.get(entity);
    if (!entityComponents) {
      entityComponents = new Map();
      this.components.set(entity, entityComponents);
    }
    entityComponents.set(type, data);
  }

  getComponent<T extends keyof ComponentDataMap>(
    entity: EntityId,
    type: T,
  ): ComponentDataMap[T] | undefined {
    const entityComponents = this.components.get(entity);
    if (!entityComponents) return undefined;
    return entityComponents.get(type) as ComponentDataMap[T] | undefined;
  }

  createSnapshot(): WorldSnapshot {
    return new Snapshot(this.components);
  }
}
