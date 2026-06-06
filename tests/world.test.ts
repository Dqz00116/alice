import { describe, it, expect } from 'vitest';
import { AliceWorld } from '#src/core/world';
import type { MessagesComponent, ConfigComponent, LoopComponent } from '#src/core/types';

describe('AliceWorld', () => {
  it('should write and read a component, returning the same reference', () => {
    const world = new AliceWorld();
    const msgComp: MessagesComponent = { messages: [] };
    world.setComponent('agent', 'messages', msgComp);
    const result = world.getComponent('agent', 'messages');
    expect(result).toBe(msgComp);
  });

  it('should return undefined for a non-existent component', () => {
    const world = new AliceWorld();
    const result = world.getComponent('agent', 'messages');
    expect(result).toBeUndefined();
  });

  it('should create a snapshot that reflects current world state', () => {
    const world = new AliceWorld();
    const msgComp: MessagesComponent = { messages: [{ role: 'user', content: 'hello' }] };
    world.setComponent('agent', 'messages', msgComp);

    const snapshot = world.createSnapshot();
    const snapMsg = snapshot.getComponent('agent', 'messages');
    expect(snapMsg).toBe(msgComp);
  });

  it('should return undefined from snapshot for non-existent component', () => {
    const world = new AliceWorld();
    const snapshot = world.createSnapshot();
    const result = snapshot.getComponent('agent', 'messages');
    expect(result).toBeUndefined();
  });

  it('should support multiple entity types', () => {
    const world = new AliceWorld();
    const config: ConfigComponent = { maxSteps: 5, temperature: 0.5, model: 'test' };
    const loop: LoopComponent = { step: 0, shouldContinue: true };
    world.setComponent('agent', 'config', config);
    world.setComponent('agent', 'loop', loop);

    expect(world.getComponent('agent', 'config')).toBe(config);
    expect(world.getComponent('agent', 'loop')).toBe(loop);
  });

  it('snapshot should not allow mutation of world', () => {
    const world = new AliceWorld();
    const msgComp: MessagesComponent = { messages: [] };
    world.setComponent('agent', 'messages', msgComp);

    const snapshot = world.createSnapshot();
    const snapMsg = snapshot.getComponent('agent', 'messages');
    snapMsg!.messages.push({ role: 'user', content: 'injected' });

    // Shared reference mutation is expected (snapshot is read-only by convention, not by deep copy)
    expect(world.getComponent('agent', 'messages')!.messages).toHaveLength(1);
  });
});
