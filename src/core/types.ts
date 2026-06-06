// ── Entity & Component ──
export type EntityId = string;
export type ComponentType = string;

// ── Message ──
export type Message =
  | { role: 'user'; content: string }
  | { role: 'assistant'; content: string; tool_calls?: ToolCall[] }
  | { role: 'tool'; content: string; tool_call_id: string };

export interface ToolCall {
  id: string;
  type: 'function';
  function: {
    name: string;
    arguments: string;
  };
}

// ── Event (4 categories) ──
export type Event = InputEvent | LLMStreamEvent | ToolEvent | SystemEvent;

export interface InputEvent {
  type: 'input.user';
  source: string;
  content: string;
}

export type LLMStreamEvent =
  | { type: 'llm.thinking_delta'; delta: string }
  | { type: 'llm.text_delta'; delta: string }
  | { type: 'llm.tool_call'; tool_call: ToolCall }
  | { type: 'llm.stream_end'; stop_reason: string }
  | { type: 'llm.stream_error'; error: string };

export type ToolEvent =
  | { type: 'tool.call_request'; tool_calls: ToolCall[] }
  | { type: 'tool.result'; tool_call_id: string; result: string }
  | { type: 'tool.error'; tool_call_id: string; error: string };

export type SystemEvent =
  | { type: 'system.step_start'; step: number }
  | { type: 'system.step_end'; step: number }
  | { type: 'system.hook_trigger'; hook: string };

// ── Effect (7 variants) ──
export type Effect =
  | CallLLMEffect
  | ExecuteToolEffect
  | AppendMessageEffect
  | UpdateComponentEffect
  | EmitEffect
  | RenderEffect
  | AbortEffect;

export interface CallLLMEffect {
  type: 'effect.callLLM';
  provider: string;
  messages: Message[];
}

export interface ExecuteToolEffect {
  type: 'effect.executeTool';
  tool_name: string;
  args: Record<string, unknown>;
}

export interface AppendMessageEffect {
  type: 'effect.appendMessage';
  entity: EntityId;
  message: Message;
}

export interface UpdateComponentEffect {
  type: 'effect.updateComponent';
  entity: EntityId;
  component: ComponentType;
  data: unknown;
}

export interface EmitEffect {
  type: 'effect.emit';
  event: Event;
}

export interface RenderEffect {
  type: 'effect.render';
  content: string;
  stream: 'thinking' | 'text';
}

export interface AbortEffect {
  type: 'effect.abort';
  reason: string;
}

// ── System (pure function) ──
export type System = (snapshot: WorldSnapshot, event: Event) => Effect[];

// ── World ──
export interface World {
  setComponent<T extends keyof ComponentDataMap>(
    entity: EntityId,
    type: T,
    data: ComponentDataMap[T],
  ): void;
  getComponent<T extends keyof ComponentDataMap>(
    entity: EntityId,
    type: T,
  ): ComponentDataMap[T] | undefined;
  createSnapshot(): WorldSnapshot;
}

export interface WorldSnapshot {
  getComponent<T extends keyof ComponentDataMap>(
    entity: EntityId,
    type: T,
  ): ComponentDataMap[T] | undefined;
}

// ── Component Data Map (extensible) ──
export interface MessagesComponent {
  messages: Message[];
}

export interface ProviderComponent {
  formatMessages(messages: Message[]): unknown;
  streamChat(body: unknown): AsyncGenerator<LLMStreamEvent>;
}

export interface ToolsComponent {
  definitions: ToolDef[];
  handlers: Record<string, ToolHandler>;
}

export interface ToolDef {
  name: string;
  description: string;
  input_schema: unknown;
}

export type ToolHandler = (args: Record<string, unknown>) => Promise<string>;

export interface ConfigComponent {
  maxSteps: number;
  temperature: number;
  model: string;
}

export interface LoopComponent {
  step: number;
  shouldContinue: boolean;
}

export interface ComponentDataMap {
  messages: MessagesComponent;
  provider: ProviderComponent;
  tools: ToolsComponent;
  config: ConfigComponent;
  loop: LoopComponent;
}
