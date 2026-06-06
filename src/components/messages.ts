import type { MessagesComponent, Message } from '#src/core/types';

export function createMessagesComponent(initialMessages: Message[] = []): MessagesComponent {
  return { messages: [...initialMessages] };
}
