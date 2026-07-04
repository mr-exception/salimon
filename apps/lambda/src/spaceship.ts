import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import { randomUUID } from 'node:crypto';
import { MongoClient } from 'mongodb';

export const SECURITY_CODE_HEADER = 'x-spaceship-security-code';

type SpaceshipPosition = {
  x: string;
  y: string;
  relativeTo?: string;
};

export type SpaceshipVelocity = {
  x: number;
  y: number;
};

export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';

export type SpaceshipDocument = {
  securityCode: string;
  position: SpaceshipPosition;
  direction: number;
  speed: string;
  velocity?: SpaceshipVelocity;
  motionState?: SpaceshipMotionState;
  simulatedAt?: Date;
  createdAt: Date;
  updatedAt: Date;
};

export type SpaceshipDto = Pick<
  SpaceshipDocument,
  | 'securityCode'
  | 'position'
  | 'direction'
  | 'speed'
  | 'velocity'
  | 'motionState'
> & { simulatedAt: string };

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
} satisfies Omit<SpaceshipDto, 'securityCode' | 'simulatedAt'>;
const UUID_V4_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const INTEGER_PATTERN = /^-?\d+$/;
let clientPromise: Promise<MongoClient> | undefined;

export function jsonResponse(
  statusCode: number,
  body: Record<string, unknown>,
): APIGatewayProxyResultV2 {
  return {
    statusCode,
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  };
}

export async function getSpaceshipsCollection() {
  return (await getDatabase()).collection<SpaceshipDocument>('spaceships');
}

export async function getDatabase() {
  const uri = process.env.MONGODB_URI;
  if (!uri) throw new Error('MONGODB_URI is not configured');

  clientPromise ??= new MongoClient(uri).connect().catch((error: unknown) => {
    clientPromise = undefined;
    throw error;
  });

  return (await clientPromise).db();
}

export function createSpaceship(): SpaceshipDocument {
  const now = new Date();
  return {
    ...DEFAULT_SPACESHIP,
    position: { ...DEFAULT_SPACESHIP.position },
    securityCode: randomUUID(),
    simulatedAt: now,
    createdAt: now,
    updatedAt: now,
  };
}

export function toSpaceshipDto(spaceship: SpaceshipDocument): SpaceshipDto {
  const velocity = getSpaceshipVelocity(spaceship);
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
    simulatedAt: (spaceship.simulatedAt ?? spaceship.updatedAt).toISOString(),
  };
}

export function getSpaceshipVelocity(
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

export function getSecurityCode(event: APIGatewayProxyEventV2) {
  const securityCode = event.headers[SECURITY_CODE_HEADER];
  return securityCode && UUID_V4_PATTERN.test(securityCode)
    ? securityCode
    : undefined;
}

export function parseSpaceshipUpdate(event: APIGatewayProxyEventV2) {
  if (!event.body) throw new Error('Request body is required');

  let body: unknown;
  try {
    body = JSON.parse(event.body);
  } catch {
    throw new Error('Request body must be valid JSON');
  }

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

  const { direction, speed, velocity, motionState } = candidate;
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
  };
}
