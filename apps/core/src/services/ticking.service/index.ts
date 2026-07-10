import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import { SpaceshipService } from '../spaceship.service';
import { advanceBodyPosition } from './advance-body-position';
import { cloneSpaceship } from './clone-spaceship';
import { TICK_INTERVAL_MS } from './constants';
import {
  createTargetSpeedFeature,
  getBodyPositions,
  getBodyVelocity,
  getSpaceshipUpdate,
} from './get-spaceship-update';
import type { Timer, WorldSnapshot } from './types';

type TickResult = {
  bodies: number;
  spaceships: number;
};

export class TickingService {
  private static timer: Timer | undefined;
  private static startPromise: Promise<void> | undefined;
  private static tickPromise: Promise<TickResult> | undefined;

  static async start() {
    TickingService.startPromise ??= TickingService.startTicking();
    return TickingService.startPromise;
  }

  static stop() {
    if (TickingService.timer) {
      clearInterval(TickingService.timer);
      TickingService.timer = undefined;
    }

    TickingService.startPromise = undefined;
    TickingService.tickPromise = undefined;
  }

  static async getWorldData() {
    await TickingService.start();
    return RepositoryService.getWorldData();
  }

  static async getWorldSystemsBodies() {
    await TickingService.start();
    return RepositoryService.getWorldSystemsBodies();
  }

  static async updateWorld(time: string | Date = new Date()) {
    await TickingService.start();
    const invocationTime = TickingService.parseInvocationTime(time);
    const result = await TickingService.tick(invocationTime);
    return {
      selected: result.bodies + result.spaceships,
      updated: result.bodies + result.spaceships,
      bodies: result.bodies,
      spaceships: result.spaceships,
    };
  }

  static async updateWorldBodies(time: string | Date = new Date()) {
    await TickingService.start();
    const updated = await TickingService.advanceBodies(
      TickingService.parseInvocationTime(time),
    );
    return {
      selected: updated,
      updated,
    };
  }

  static async updateSpaceships(time: string | Date = new Date()) {
    await TickingService.start();
    const invocationTime = TickingService.parseInvocationTime(time);
    const processed = await TickingService.advanceSpaceships(
      invocationTime,
      await TickingService.loadWorldSnapshot(),
    );
    return {
      selected: processed,
      processed,
    };
  }

  static async updateSpaceship(
    spaceship: SpaceshipDocument,
    simulatedAt = new Date(),
    suppliedWorld?: WorldSnapshot,
  ) {
    const world = suppliedWorld ?? (await TickingService.loadWorldSnapshot());
    const update = getSpaceshipUpdate(spaceship, simulatedAt, world);
    if (!update) return spaceship;
    return RepositoryService.updatePropagatedSpaceship(spaceship, update);
  }

  static async startSpaceshipTargetSpeedFeature(
    spaceship: SpaceshipDocument,
    params: {
      targetSpeedMetersPerSecond: number;
      maximumThrustPercent: number;
      targetDirection?: number;
    },
  ) {
    const simulatedAt = new Date();
    const world = await TickingService.loadWorldSnapshot();
    const currentSpaceship = await TickingService.updateSpaceship(
      spaceship,
      simulatedAt,
      world,
    );
    const activeFeature = createTargetSpeedFeature(
      currentSpaceship,
      simulatedAt,
      world,
      params.targetSpeedMetersPerSecond,
      params.maximumThrustPercent,
      params.targetDirection,
    );
    if (!activeFeature) return undefined;

    const referencePosition = currentSpaceship.position.relativeTo
      ? getBodyPositions(world, simulatedAt).get(
          currentSpaceship.position.relativeTo,
        )
      : undefined;
    const referenceVelocity = currentSpaceship.position.relativeTo
      ? getBodyVelocity(world, currentSpaceship.position.relativeTo, simulatedAt)
      : undefined;
    const relativePosition = {
      x: Number(currentSpaceship.position.x),
      y: Number(currentSpaceship.position.y),
    };
    const relativeRadius = Math.hypot(relativePosition.x, relativePosition.y);
    const launchClearance =
      referencePosition && relativeRadius > 0
        ? {
            x: relativePosition.x / relativeRadius,
            y: relativePosition.y / relativeRadius,
          }
        : { x: 0, y: 0 };
    const worldPosition = referencePosition
      ? {
          x: Math.round(
            referencePosition.x + relativePosition.x + launchClearance.x,
          ).toString(),
          y: Math.round(
            referencePosition.y + relativePosition.y + launchClearance.y,
          ).toString(),
        }
      : currentSpaceship.position;
    const worldVelocity = referenceVelocity
      ? {
          x:
            referenceVelocity.x +
            SpaceshipService.getSpaceshipVelocity(currentSpaceship).x,
          y:
            referenceVelocity.y +
            SpaceshipService.getSpaceshipVelocity(currentSpaceship).y,
        }
      : currentSpaceship.velocity;

    return RepositoryService.updatePropagatedSpaceship(currentSpaceship, {
      activeFeature,
      motionState: 'flying',
      position: worldPosition,
      velocity: worldVelocity,
      simulatedAt,
      updatedAt: simulatedAt,
    });
  }

  static async stopSpaceshipActiveFeature(spaceship: SpaceshipDocument) {
    const simulatedAt = new Date();
    const currentSpaceship = await TickingService.updateSpaceship(
      spaceship,
      simulatedAt,
    );
    return RepositoryService.updatePropagatedSpaceship(currentSpaceship, {
      activeFeature: undefined,
      simulatedAt,
      updatedAt: simulatedAt,
    });
  }

  static async flushToDatabase() {
    return RepositoryService.flushToDatabase();
  }

  private static async startTicking() {
    await RepositoryService.start();
    await TickingService.tick(new Date());

    TickingService.timer ??= setInterval(() => {
      void TickingService.tick(new Date()).catch((error: unknown) => {
        console.error('Failed to tick world data', error);
      });
    }, TICK_INTERVAL_MS);
  }

  private static async tick(invocationTime: Date) {
    if (TickingService.tickPromise) return TickingService.tickPromise;

    const startedAt = Date.now();
    TickingService.tickPromise = (async () => {
      const world = await TickingService.loadWorldSnapshot();
      const bodies = await TickingService.advanceBodies(invocationTime);
      const spaceships = await TickingService.advanceSpaceships(
        invocationTime,
        world,
      );
      return { bodies, spaceships };
    })().finally(() => {
      console.log(`tick passed: ${Date.now() - startedAt}ms`);
      TickingService.tickPromise = undefined;
    });

    return TickingService.tickPromise;
  }

  private static async advanceBodies(invocationTime: Date) {
    let updated = 0;

    await RepositoryService.updateWorldBodies((worldData) => {
      for (const body of [
        ...[...worldData.stars].sort((left, right) =>
          left.name.localeCompare(right.name),
        ),
        ...[...worldData.planets].sort((left, right) =>
          left.name.localeCompare(right.name),
        ),
        ...[...worldData.moons].sort((left, right) =>
          left.name.localeCompare(right.name),
        ),
      ]) {
        const elapsedSeconds = Math.max(
          0,
          (invocationTime.getTime() - body.updatedAt.getTime()) / 1_000,
        );
        body.position = advanceBodyPosition(body, elapsedSeconds);
        body.updatedAt = invocationTime;
        updated += 1;
      }

      return updated;
    });

    return updated;
  }

  private static async advanceSpaceships(
    invocationTime: Date,
    world: WorldSnapshot,
  ) {
    let processed = 0;

    await RepositoryService.updateSpaceships((spaceshipsBySecurityCode) => {
      const spaceships = [...spaceshipsBySecurityCode.values()]
        .sort((left, right) =>
          left.securityCode.localeCompare(right.securityCode),
        )
        .map(cloneSpaceship);

      for (const spaceship of spaceships) {
        const update = getSpaceshipUpdate(spaceship, invocationTime, world);
        if (!update) continue;

        const current = spaceshipsBySecurityCode.get(spaceship.securityCode);
        if (
          !current ||
          current.updatedAt.getTime() !== spaceship.updatedAt.getTime()
        ) {
          continue;
        }

        spaceshipsBySecurityCode.set(
          spaceship.securityCode,
          cloneSpaceship({
            ...current,
            ...update,
            position: update.position ?? current.position,
            velocity: update.velocity ?? current.velocity,
            stats: update.stats ?? current.stats,
          }),
        );
        processed += 1;
      }

      return processed;
    });

    return processed;
  }

  private static async loadWorldSnapshot(): Promise<WorldSnapshot> {
    const { planets, moons, stars } = await RepositoryService.getWorldData();
    const bodies = [...planets, ...moons, ...stars];
    return {
      bodies,
      bodiesByName: new Map(bodies.map((body) => [body.name, body])),
    };
  }

  private static parseInvocationTime(time: string | Date) {
    const invocationTime = new Date(time);
    if (Number.isNaN(invocationTime.getTime())) {
      throw new Error('Invocation time is invalid');
    }
    return invocationTime;
  }
}
