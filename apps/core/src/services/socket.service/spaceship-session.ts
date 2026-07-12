import { type SpaceshipDocument, SpaceshipService } from '../spaceship.service';
import { TickingService } from '../ticking.service';
import type { AsteroidDto } from '@repo/types';
import type { WorldBodyDocument } from '@models';
import { AsteroidService } from '../asteroid.service';
import {
  type WorldViewportRequest,
  WorldViewportService,
} from '../world-viewport.service';
import { RepositoryService } from '../repository.service';

type Coordinate = {
  x: bigint;
  y: bigint;
};

export class SpaceshipSession {
  private viewport?: WorldViewportRequest;
  private asteroids: AsteroidDto[] = [];

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

  async startManualForceFeature(params: {
    thrusters: { powerPercent: number; durationSeconds: number }[];
  }) {
    const spaceship = await TickingService.startSpaceshipManualForceFeature(
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

  async updateInventoryFromClient(body: unknown) {
    const inventory = SpaceshipService.parseSpaceshipInventory(body);
    const spaceship =
      await RepositoryService.updateSpaceshipInventoryBySecurityCode(
        this.securityCode,
        inventory,
      );
    if (!spaceship) return undefined;

    this.spaceship = spaceship;
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
    const world = await WorldViewportService.getWorldSystems(viewport, {
      requiredBodyNames: this.getRequiredWorldBodyNames(),
    });
    return {
      ...world,
      asteroids: await this.getViewportAsteroids(viewport),
    };
  }

  async getCurrentViewportWorldSystems() {
    if (!this.viewport) return undefined;

    const world = await WorldViewportService.getWorldSystems(this.viewport, {
      requiredBodyNames: this.getRequiredWorldBodyNames(),
    });
    return {
      ...world,
      asteroids: await this.getViewportAsteroids(this.viewport),
    };
  }

  async getAsteroids() {
    this.asteroids = await AsteroidService.updateSessionAsteroids({
      asteroids: this.asteroids,
      spaceshipPosition: this.spaceship.position,
    });
    return this.asteroids;
  }

  async getViewportAsteroids(viewport: WorldViewportRequest) {
    const asteroids = await this.getAsteroids();
    const center = this.getViewportCenter(viewport);
    const radius = this.parseViewportRadius(viewport);
    if (!center || radius === undefined) return asteroids;

    const worldData = await TickingService.getWorldData();
    const worldBodies = [
      ...worldData.planets,
      ...worldData.moons,
      ...worldData.stars,
    ];
    const bodyPositions = this.resolveWorldBodyPositions(worldBodies);
    return asteroids.filter((asteroid) => {
      const position = this.resolveAsteroidPosition(asteroid, bodyPositions);
      const distanceSquared =
        (position.x - center.x) ** 2n + (position.y - center.y) ** 2n;
      return distanceSquared <= radius ** 2n;
    });
  }

  private getViewportCenter(viewport: WorldViewportRequest) {
    if (viewport.x === undefined || viewport.y === undefined) return undefined;

    try {
      return {
        x: BigInt(viewport.x),
        y: BigInt(viewport.y),
      };
    } catch {
      return undefined;
    }
  }

  private parseViewportRadius(viewport: WorldViewportRequest) {
    if (viewport.radius === undefined) return undefined;

    try {
      const radius = BigInt(viewport.radius);
      return radius >= 0n ? radius : undefined;
    } catch {
      return undefined;
    }
  }

  private resolveAsteroidPosition(
    asteroid: AsteroidDto,
    bodyPositions: Map<string, Coordinate>,
  ): Coordinate {
    const position = {
      x: BigInt(asteroid.position.x),
      y: BigInt(asteroid.position.y),
    };
    if (!asteroid.position.relativeTo) return position;

    const reference = bodyPositions.get(asteroid.position.relativeTo);
    return reference
      ? {
          x: reference.x + position.x,
          y: reference.y + position.y,
        }
      : position;
  }

  private resolveWorldBodyPositions(bodies: WorldBodyDocument[]) {
    const bodyByName = new Map(bodies.map((body) => [body.name, body]));
    const positionByName = new Map<string, Coordinate>();

    const resolve = (body: WorldBodyDocument): Coordinate => {
      const cached = positionByName.get(body.name);
      if (cached) return cached;

      const position = {
        x: BigInt(body.position.x),
        y: BigInt(body.position.y),
      };
      const referenceName = body.position.relativeTo;
      const reference = referenceName
        ? bodyByName.get(referenceName)
        : undefined;
      if (reference) {
        const referencePosition = resolve(reference);
        position.x += referencePosition.x;
        position.y += referencePosition.y;
      }

      positionByName.set(body.name, position);
      return position;
    };

    bodies.forEach((body) => resolve(body));
    return positionByName;
  }

  private getRequiredWorldBodyNames() {
    return this.spaceship.position.relativeTo
      ? [this.spaceship.position.relativeTo]
      : [];
  }
}
