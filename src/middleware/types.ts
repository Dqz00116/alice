import type { Event } from '#src/core/types';

export type NextFn = () => void;
export type Middleware = (event: Event, next: NextFn) => void;

export class MiddlewarePipeline {
  private middlewares: Middleware[] = [];

  use(middleware: Middleware): void {
    this.middlewares.push(middleware);
  }

  run(event: Event, finalHandler: () => void): void {
    let index = 0;
    const next: NextFn = () => {
      if (index < this.middlewares.length) {
        const mw = this.middlewares[index++]!;
        mw(event, next);
      } else {
        finalHandler();
      }
    };
    next();
  }
}
