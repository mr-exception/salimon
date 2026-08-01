import type { SpaceshipDocument } from '@models';
import type { SpaceshipDto } from '@repo/types';
import { getSpaceshipVelocity } from './get-spaceship-velocity';
import { normalizeSpaceshipInventory } from './normalize-spaceship-inventory';
import { normalizeSpaceshipStats } from './normalize-spaceship-stats';

function normalizeActiveFeature(
  activeFeature: SpaceshipDocument['activeFeature'] | unknown,
): SpaceshipDto['activeFeature'] {
  if (
    activeFeature &&
    typeof activeFeature === 'object' &&
    'type' in activeFeature &&
    activeFeature.type === 'lock-on'
  ) {
    return undefined;
  }

  return activeFeature as SpaceshipDto['activeFeature'];
}

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
    inventory: normalizeSpaceshipInventory(spaceship.inventory),
    activeFeature: normalizeActiveFeature(spaceship.activeFeature),
    simulatedAt: (spaceship.simulatedAt ?? spaceship.updatedAt).toISOString(),
  };
}
