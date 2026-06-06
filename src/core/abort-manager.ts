export type AbortHandler = (reason: string) => void;

export class AbortManager {
  private controller: AbortController | null = null;
  private handlers: AbortHandler[] = [];

  get signal(): AbortSignal | null {
    return this.controller?.signal ?? null;
  }

  createScope(): AbortSignal {
    this.controller?.abort('scope superseded');
    this.controller = new AbortController();
    return this.controller.signal;
  }

  abort(reason: string): void {
    this.controller?.abort(reason);
    for (const handler of this.handlers) {
      handler(reason);
    }
  }

  onAbort(handler: AbortHandler): void {
    this.handlers.push(handler);
  }
}
