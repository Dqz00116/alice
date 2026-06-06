import type { ProviderComponent, Message, LLMStreamEvent } from '#src/core/types';

export function createProviderComponent(config: {
  formatMessages: (messages: Message[]) => unknown;
  streamChat: (body: unknown) => AsyncGenerator<LLMStreamEvent>;
}): ProviderComponent {
  return {
    formatMessages: config.formatMessages,
    streamChat: config.streamChat,
  };
}
