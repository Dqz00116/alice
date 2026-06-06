import type { Message, LLMStreamEvent } from '#src/core/types';

export function formatMessages(messages: Message[]): unknown {
  throw new Error('not implemented');
}

export async function* streamChat(body: unknown): AsyncGenerator<LLMStreamEvent> {
  throw new Error('not implemented');
}
