import { SandboxObject, type SandboxObjectParams } from "./sandbox-object";

export type SandboxState = {
  objects: SandboxObjectParams[];
};

export class SandBox {
  private static readonly fps = 30;
  private static readonly tickIntervalMs = 1000 / SandBox.fps;

  private readonly objects = new Map<string, SandboxObject>();
  private tickCount = 0;
  private tickTimer?: ReturnType<typeof setTimeout>;

  constructor(state: Partial<SandboxState> = {}) {
    state.objects?.forEach((object) => this.addObject(object));
  }

  addObject(params: SandboxObject | SandboxObjectParams) {
    const object =
      params instanceof SandboxObject ? params : new SandboxObject(params);

    object.setSandbox(this);
    this.objects.set(object.id, object);
    return object;
  }

  removeObject(id: string) {
    const object = this.objects.get(id);
    this.objects.delete(id);
    object?.setSandbox(undefined);
    return object;
  }

  getObject(id: string) {
    return this.objects.get(id);
  }

  listObjects() {
    return Array.from(this.objects.values());
  }

  start() {
    if (this.tickTimer) {
      clearTimeout(this.tickTimer);
    }

    const tick = () => {
      const startedAt = Date.now();
      const objects = this.listObjects();

      objects.forEach((object) => object.tick(startedAt));

      const calcTime = Date.now() - startedAt;

      this.tickCount += 1;
      console.log(
        `tick passed, count: ${this.tickCount}, calcTime: ${calcTime}ms, fps: ${SandBox.fps}`,
      );

      this.tickTimer = setTimeout(
        tick,
        Math.max(SandBox.tickIntervalMs - calcTime, 0),
      );
    };

    tick();
  }
}
