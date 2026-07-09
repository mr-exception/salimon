import { randomUUID } from 'node:crypto';
import {
  SpaceshipModel,
  type SpaceshipDocument,
  type SpaceshipMotionState,
  type SpaceshipStats,
} from '@models';
import type { SpaceshipDto } from '@repo/types';

export const SECURITY_CODE_HEADER = 'x-spaceship-security-code';

export type {
  SpaceshipDocument,
  SpaceshipMotionState,
  SpaceshipStats,
  SpaceshipVelocity,
} from '@models';

export type { SpaceshipDto };

const INITIAL_SPACESHIP_FUEL_KNS = 1_000_000;
export const MAX_HULL_DURABILITY = 200;
export const MAX_THRUSTER_DURABILITY = 100;
export const SPACESHIP_THRUSTER_COUNT = 4;
const DEFAULT_SPACESHIP = {
  position: {
    x: '6371200',
    y: '0',
    relativeTo: 'Earth',
  },
  direction: 0,
  speed: '0',
  velocity: { x: 0, y: 0 },
  motionState: 'landed',
  stats: {
    fuelKns: INITIAL_SPACESHIP_FUEL_KNS,
    hullDurability: MAX_HULL_DURABILITY,
    thrusterDurability: Array(SPACESHIP_THRUSTER_COUNT).fill(
      MAX_THRUSTER_DURABILITY,
    ),
  },
} satisfies Omit<SpaceshipDto, 'securityCode' | 'simulatedAt'>;
const UUID_V4_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const INTEGER_PATTERN = /^-?\d+$/;

export class SpaceshipService {
  static createSpaceship(): SpaceshipDocument {
    const now = new Date();
    return {
      ...DEFAULT_SPACESHIP,
      position: { ...DEFAULT_SPACESHIP.position },
      stats: { ...DEFAULT_SPACESHIP.stats },
      securityCode: randomUUID(),
      simulatedAt: now,
      createdAt: now,
      updatedAt: now,
    };
  }

  static toSpaceshipDto(spaceship: SpaceshipDocument): SpaceshipDto {
    const velocity = SpaceshipService.getSpaceshipVelocity(spaceship);
    return {
      securityCode: spaceship.securityCode,
      position: spaceship.position,
      direction: spaceship.direction,
      speed: spaceship.speed,
      velocity,
      motionState:
        spaceship.motionState ??
        (spaceship.speed === '0' && spaceship.position.relativeTo
          ? 'landed'
          : 'flying'),
      stats: SpaceshipService.normalizeSpaceshipStats(spaceship.stats),
      simulatedAt: (spaceship.simulatedAt ?? spaceship.updatedAt).toISOString(),
    };
  }

  static async loadSpaceship(securityCode: string) {
    const storedSpaceship = await SpaceshipModel.findBySecurityCode(
      securityCode,
    );
    if (!storedSpaceship) return undefined;

    const { OfflineSpaceshipService } = await import(
      './offline-spaceship.service.js'
    );
    return OfflineSpaceshipService.propagateOfflineSpaceship(storedSpaceship);
  }

  static async updateSpaceship(
    securityCode: string,
    update: ReturnType<typeof SpaceshipService.parseSpaceshipUpdate>,
  ): Promise<SpaceshipDocument | undefined> {
    const now = new Date();
    return SpaceshipModel.updateBySecurityCode(securityCode, {
      ...update,
      simulatedAt: now,
      updatedAt: now,
    });
  }

  static getSpaceshipVelocity(
    spaceship: Pick<SpaceshipDocument, 'direction' | 'speed' | 'velocity'>,
  ) {
    if (spaceship.velocity) return spaceship.velocity;

    const speed = Number(spaceship.speed);
    const headingRadians = (spaceship.direction * Math.PI) / 180;
    return {
      x: Math.sin(headingRadians) * speed,
      y: -Math.cos(headingRadians) * speed,
    };
  }

  static getSecurityCode(headers: Record<string, unknown>) {
    const rawSecurityCode = headers[SECURITY_CODE_HEADER];
    const securityCode = Array.isArray(rawSecurityCode)
      ? rawSecurityCode[0]
      : rawSecurityCode;
    return securityCode && UUID_V4_PATTERN.test(securityCode)
      ? securityCode
      : undefined;
  }

  static parseSpaceshipUpdate(body: unknown) {
    if (!body || typeof body !== 'object') {
      throw new Error('Request body must be an object');
    }

    const candidate = body as Record<string, unknown>;
    const position = candidate.position;
    if (!position || typeof position !== 'object') {
      throw new Error('position is required');
    }

    const positionCandidate = position as Record<string, unknown>;
    const { x, y, relativeTo } = positionCandidate;
    if (
      typeof x !== 'string' ||
      !INTEGER_PATTERN.test(x) ||
      typeof y !== 'string' ||
      !INTEGER_PATTERN.test(y)
    ) {
      throw new Error('position.x and position.y must be integer strings');
    }
    if (
      relativeTo !== undefined &&
      (typeof relativeTo !== 'string' || relativeTo.length === 0)
    ) {
      throw new Error('position.relativeTo must be a non-empty string');
    }

    const { direction, speed, velocity, motionState, stats } = candidate;
    if (
      typeof direction !== 'number' ||
      !Number.isFinite(direction) ||
      direction < 0 ||
      direction >= 360
    ) {
      throw new Error('direction must be a number from 0 up to 360');
    }
    if (typeof speed !== 'string' || !/^\d+$/.test(speed)) {
      throw new Error('speed must be a non-negative integer string');
    }
    if (!velocity || typeof velocity !== 'object') {
      throw new Error('velocity is required');
    }
    const velocityCandidate = velocity as Record<string, unknown>;
    if (
      typeof velocityCandidate.x !== 'number' ||
      !Number.isFinite(velocityCandidate.x) ||
      typeof velocityCandidate.y !== 'number' ||
      !Number.isFinite(velocityCandidate.y)
    ) {
      throw new Error('velocity.x and velocity.y must be finite numbers');
    }
    if (
      motionState !== 'flying' &&
      motionState !== 'landed' &&
      motionState !== 'crashed'
    ) {
      throw new Error('motionState must be flying, landed, or crashed');
    }
    const parsedMotionState: SpaceshipMotionState = motionState;
    if (!stats || typeof stats !== 'object') {
      throw new Error('stats is required');
    }
    const statsCandidate = stats as Record<string, unknown>;
    if (
      typeof statsCandidate.fuelKns !== 'number' ||
      !Number.isFinite(statsCandidate.fuelKns) ||
      statsCandidate.fuelKns < 0
    ) {
      throw new Error('stats.fuelKns must be a non-negative finite number');
    }
    const hullDurability = statsCandidate.hullDurability;
    const thrusterDurability = statsCandidate.thrusterDurability;
    if (
      typeof hullDurability !== 'number' ||
      !Number.isFinite(hullDurability) ||
      hullDurability < 0 ||
      hullDurability > MAX_HULL_DURABILITY
    ) {
      throw new Error('stats.hullDurability must be between 0 and 200');
    }
    if (
      !Array.isArray(thrusterDurability) ||
      thrusterDurability.length !== SPACESHIP_THRUSTER_COUNT ||
      thrusterDurability.some(
        (value) =>
          typeof value !== 'number' ||
          !Number.isFinite(value) ||
          value < 0 ||
          value > MAX_THRUSTER_DURABILITY,
      )
    ) {
      throw new Error(
        'stats.thrusterDurability must contain four values from 0 to 100',
      );
    }

    return {
      position: {
        x,
        y,
        ...(relativeTo === undefined ? {} : { relativeTo }),
      },
      direction,
      speed,
      velocity: { x: velocityCandidate.x, y: velocityCandidate.y },
      motionState: parsedMotionState,
      stats: {
        fuelKns: statsCandidate.fuelKns,
        hullDurability,
        thrusterDurability,
      },
    };
  }

  static normalizeSpaceshipStats(
    stats: Partial<SpaceshipStats> | undefined,
  ): SpaceshipStats {
    return {
      fuelKns: stats?.fuelKns ?? INITIAL_SPACESHIP_FUEL_KNS,
      hullDurability: stats?.hullDurability ?? MAX_HULL_DURABILITY,
      thrusterDurability: Array.from(
        { length: SPACESHIP_THRUSTER_COUNT },
        (_, index) =>
          stats?.thrusterDurability?.[index] ?? MAX_THRUSTER_DURABILITY,
      ),
    };
  }
}
