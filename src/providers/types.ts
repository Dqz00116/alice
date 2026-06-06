import type { Message, LLMStreamEvent } from '#src/core/types';

export interface StreamingProvider {
  formatMessages(messages: Message[]): unknown;
  streamChat(body: unknown): AsyncGenerator<LLMStreamEvent>;
}
