import type { SpaceshipDocument } from '@models';
import type { SpaceshipDto } from '@repo/types';
import { getSpaceshipVelocity } from './get-spaceship-velocity';
import { normalizeSpaceshipStats } from './normalize-spaceship-stats';

export function toSpaceshipDto(spaceship: SpaceshipDocument): SpaceshipDto {
  const velocity = getSpaceshipVelocity(spaceship);
  return {
    securityCode: spaceship.securityCode,
    position: spaceship.position,
    positionCapturedAt: (
      spaceship.simulatedAt ?? spaceship.updatedAt
    ).toISOString(),
    direction: spaceship.direction,
    speed: spaceship.speed,
    velocity,
    motionState:
      spaceship.motionState ??
      (spaceship.speed === '0' && spaceship.position.relativeTo
        ? 'landed'
        : 'flying'),
    stats: normalizeSpaceshipStats(spaceship.stats),
    activeFeature: spaceship.activeFeature,
    simulatedAt: (spaceship.simulatedAt ?? spaceship.updatedAt).toISOString(),
  };
}

