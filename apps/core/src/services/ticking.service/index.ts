import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import { SpaceshipService } from '../spaceship.service';
import { advanceBodyPosition } from './advance-body-position';
import { cloneSpaceship } from './clone-spaceship';
import {
  SPACESHIP_LAUNCH_CLEARANCE_METERS,
  SPACESHIP_RADIUS_METERS,
  TICK_INTERVAL_MS,
} from './constants';
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

  static async createSpaceship() {
    const spaceship = SpaceshipService.createSpaceship();
    const simulatedAt = spaceship.simulatedAt ?? new Date();
    const world = await TickingService.loadWorldSnapshot();
    const absoluteUpdate = TickingService.getAbsoluteSpaceshipUpdate(
      spaceship,
      simulatedAt,
      world,
    );
    return {
      ...spaceship,
      ...absoluteUpdate,
      position: absoluteUpdate?.position ?? spaceship.position,
      velocity: absoluteUpdate?.velocity ?? spaceship.velocity,
    };
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
    const absoluteUpdate = TickingService.getAbsoluteSpaceshipUpdate(
      spaceship,
      simulatedAt,
      world,
    );
    const currentSpaceship = absoluteUpdate
      ? {
          ...spaceship,
          ...absoluteUpdate,
          position: absoluteUpdate.position ?? spaceship.position,
          velocity: absoluteUpdate.velocity ?? spaceship.velocity,
        }
      : spaceship;
    const update = getSpaceshipUpdate(currentSpaceship, simulatedAt, world);
    if (!absoluteUpdate && !update) return spaceship;
    return RepositoryService.updatePropagatedSpaceship(spaceship, {
      ...absoluteUpdate,
      ...update,
      position: update?.position ?? absoluteUpdate?.position,
      velocity: update?.velocity ?? absoluteUpdate?.velocity,
      stats: update?.stats ?? absoluteUpdate?.stats,
    });
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
    const currentReferenceName = currentSpaceship.position.relativeTo;
    const currentReferenceBody = currentReferenceName
      ? world.bodiesByName.get(currentReferenceName)
      : undefined;
    const currentReferencePosition = currentReferenceName
      ? getBodyPositions(world, simulatedAt).get(currentReferenceName)
      : undefined;
    const currentReferenceVelocity =
      currentReferenceName && currentReferenceBody
        ? getBodyVelocity(world, currentReferenceName, simulatedAt)
        : undefined;
    const relativePosition = {
      x: Number(currentSpaceship.position.x),
      y: Number(currentSpaceship.position.y),
    };
    const absolutePosition = currentReferencePosition
      ? {
          x: currentReferencePosition.x + relativePosition.x,
          y: currentReferencePosition.y + relativePosition.y,
        }
      : relativePosition;
    const relativeVelocity =
      SpaceshipService.getSpaceshipVelocity(currentSpaceship);
    const worldVelocity = currentReferenceVelocity
      ? {
          x: currentReferenceVelocity.x + relativeVelocity.x,
          y: currentReferenceVelocity.y + relativeVelocity.y,
        }
      : relativeVelocity;
    const planningSpaceship = {
      ...currentSpaceship,
      position: {
        x: Math.round(absolutePosition.x).toString(),
        y: Math.round(absolutePosition.y).toString(),
      },
      velocity: worldVelocity,
    };
    const launchReference =
      currentReferenceBody && currentReferencePosition && currentReferenceVelocity
        ? {
            body: currentReferenceBody,
            position: currentReferencePosition,
            velocity: currentReferenceVelocity,
            surfaceDistance: 0,
          }
        : TickingService.findClosestReference(
            planningSpaceship,
            simulatedAt,
            world,
          );
    const activeFeature = createTargetSpeedFeature(
      planningSpaceship,
      simulatedAt,
      world,
      params.targetSpeedMetersPerSecond,
      params.maximumThrustPercent,
      params.targetDirection,
      launchReference?.body.name,
    );
    if (!activeFeature) return undefined;

    const referencePosition = launchReference?.position;
    const referenceBody = launchReference?.body;
    const referenceVelocity = launchReference?.velocity;
    const launchRelativePosition = referencePosition
      ? {
          x: absolutePosition.x - referencePosition.x,
          y: absolutePosition.y - referencePosition.y,
        }
      : absolutePosition;
    const relativeRadius = Math.hypot(
      launchRelativePosition.x,
      launchRelativePosition.y,
    );
    const launchRadius =
      referenceBody && relativeRadius > 0
        ? Number(referenceBody.radius) +
          SPACESHIP_RADIUS_METERS +
          SPACESHIP_LAUNCH_CLEARANCE_METERS
        : relativeRadius;
    const launchPosition =
      referencePosition && relativeRadius > 0
        ? {
            x: (launchRelativePosition.x / relativeRadius) * launchRadius,
            y: (launchRelativePosition.y / relativeRadius) * launchRadius,
          }
        : launchRelativePosition;
    const worldPosition = referencePosition
      ? {
          x: Math.round(referencePosition.x + launchPosition.x).toString(),
          y: Math.round(referencePosition.y + launchPosition.y).toString(),
        }
      : currentSpaceship.position;
    const launchWorldVelocity = referenceVelocity
      ? {
          x: referenceVelocity.x + relativeVelocity.x,
          y: referenceVelocity.y + relativeVelocity.y,
        }
      : currentSpaceship.velocity;

    return RepositoryService.updatePropagatedSpaceship(currentSpaceship, {
      activeFeature,
      motionState: 'flying',
      position: worldPosition,
      velocity: launchWorldVelocity,
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
        const absoluteUpdate = TickingService.getAbsoluteSpaceshipUpdate(
          spaceship,
          invocationTime,
          world,
        );
        const currentSpaceship = absoluteUpdate
          ? {
              ...spaceship,
              ...absoluteUpdate,
              position: absoluteUpdate.position ?? spaceship.position,
              velocity: absoluteUpdate.velocity ?? spaceship.velocity,
            }
          : spaceship;
        const update = getSpaceshipUpdate(
          currentSpaceship,
          invocationTime,
          world,
        );
        if (!absoluteUpdate && !update) continue;

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
            ...absoluteUpdate,
            ...update,
            position:
              update?.position ?? absoluteUpdate?.position ?? current.position,
            velocity:
              update?.velocity ?? absoluteUpdate?.velocity ?? current.velocity,
            stats: update?.stats ?? current.stats,
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

  private static getAbsoluteSpaceshipUpdate(
    spaceship: SpaceshipDocument,
    simulatedAt: Date,
    world: WorldSnapshot,
  ): Partial<SpaceshipDocument> | undefined {
    if (spaceship.motionState !== 'flying') return undefined;

    const referenceName = spaceship.position.relativeTo;
    if (!referenceName) return undefined;

    const referencePosition = getBodyPositions(world, simulatedAt).get(
      referenceName,
    );
    if (!referencePosition) return undefined;

    const relativePosition = {
      x: Number(spaceship.position.x),
      y: Number(spaceship.position.y),
    };
    const referenceVelocity = getBodyVelocity(world, referenceName, simulatedAt);
    const relativeVelocity = SpaceshipService.getSpaceshipVelocity(spaceship);

    return {
      position: {
        x: Math.round(referencePosition.x + relativePosition.x).toString(),
        y: Math.round(referencePosition.y + relativePosition.y).toString(),
      },
      velocity: {
        x: referenceVelocity.x + relativeVelocity.x,
        y: referenceVelocity.y + relativeVelocity.y,
      },
    };
  }

  private static findClosestReference(
    spaceship: SpaceshipDocument,
    simulatedAt: Date,
    world: WorldSnapshot,
  ) {
    const spaceshipPosition = {
      x: Number(spaceship.position.x),
      y: Number(spaceship.position.y),
    };
    const positions = getBodyPositions(world, simulatedAt);
    let closest:
      | {
          body: WorldSnapshot['bodies'][number];
          position: { x: number; y: number };
          velocity: { x: number; y: number };
          surfaceDistance: number;
        }
      | undefined;

    for (const body of world.bodies) {
      const position = positions.get(body.name);
      if (!position) continue;

      const centerDistance = Math.hypot(
        spaceshipPosition.x - position.x,
        spaceshipPosition.y - position.y,
      );
      const surfaceDistance = Math.max(
        0,
        centerDistance - Number(body.radius) - SPACESHIP_RADIUS_METERS,
      );
      if (closest && surfaceDistance >= closest.surfaceDistance) continue;

      closest = {
        body,
        position,
        velocity: getBodyVelocity(world, body.name, simulatedAt),
        surfaceDistance,
      };
    }

    return closest;
  }
}
