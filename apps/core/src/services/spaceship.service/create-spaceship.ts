import { randomUUID } from 'node:crypto';
import type { SpaceshipDocument } from '@models';
import { DEFAULT_SPACESHIP } from './constants';

export function createSpaceship(): SpaceshipDocument {
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

