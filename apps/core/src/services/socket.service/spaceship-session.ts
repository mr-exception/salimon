import { type SpaceshipDocument, SpaceshipService } from '../spaceship.service';
import { TickingService } from '../ticking.service';
import {
  type WorldViewportRequest,
  WorldViewportService,
} from '../world-viewport.service';

export class SpaceshipSession {
  private viewport?: WorldViewportRequest;

  private constructor(
    private readonly securityCode: string,
    private spaceship: SpaceshipDocument,
  ) {}

  static async create(securityCode: string) {
    const spaceship = await SpaceshipService.loadSpaceship(securityCode);
    return spaceship
      ? new SpaceshipSession(securityCode, spaceship)
      : undefined;
  }

  getSpaceship() {
    return this.spaceship;
  }

  getSpaceshipDto() {
    return SpaceshipService.toSpaceshipDto(this.spaceship);
  }

  async refreshSpaceship() {
    this.spaceship = await TickingService.updateSpaceship(this.spaceship);
    return this.spaceship;
  }

  async startTargetSpeedFeature(params: {
    targetSpeedMetersPerSecond: number;
    maximumThrustPercent: number;
    targetDirection?: number;
  }) {
    const spaceship = await TickingService.startSpaceshipTargetSpeedFeature(
      this.spaceship,
      params,
    );
    if (!spaceship) return undefined;

    this.spaceship = spaceship;
    return this.spaceship;
  }

  async stopActiveFeature() {
    this.spaceship = await TickingService.stopSpaceshipActiveFeature(
      this.spaceship,
    );
    return this.spaceship;
  }

  updateSpaceshipFromClient(body: unknown) {
    const update = SpaceshipService.parseSpaceshipUpdate(body);
    const now = new Date();
    this.spaceship = {
      ...this.spaceship,
      ...update,
      securityCode: this.securityCode,
      simulatedAt: now,
      updatedAt: now,
    };
    return this.spaceship;
  }

  setViewport(viewport: WorldViewportRequest) {
    this.viewport = viewport;
  }

  async getViewportWorldSystems(viewport = this.viewport) {
    if (!viewport) throw new Error('Viewport is not set');

    this.viewport = viewport;
    return WorldViewportService.getWorldSystems(viewport, {
      requiredBodyNames: this.getRequiredWorldBodyNames(),
    });
  }

  async getCurrentViewportWorldSystems() {
    return this.viewport
      ? WorldViewportService.getWorldSystems(this.viewport, {
          requiredBodyNames: this.getRequiredWorldBodyNames(),
        })
      : undefined;
  }

  private getRequiredWorldBodyNames() {
    return this.spaceship.position.relativeTo
      ? [this.spaceship.position.relativeTo]
      : [];
  }
}
