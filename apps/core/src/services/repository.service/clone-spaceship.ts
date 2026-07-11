import type { SpaceshipDocument } from '@models';
import { cloneDate } from './clone-date';

export function cloneSpaceship(
  spaceship: SpaceshipDocument,
): SpaceshipDocument {
  return {
    ...spaceship,
    position: { ...spaceship.position },
    velocity: spaceship.velocity ? { ...spaceship.velocity } : undefined,
    stats: spaceship.stats
      ? {
          ...spaceship.stats,
          thrusterDurability: [...spaceship.stats.thrusterDurability],
        }
      : undefined,
    inventory: spaceship.inventory ? { ...spaceship.inventory } : undefined,
    activeFeature: spaceship.activeFeature
      ? {
          ...spaceship.activeFeature,
          targetVelocity: { ...spaceship.activeFeature.targetVelocity },
        }
      : undefined,
    simulatedAt: cloneDate(spaceship.simulatedAt),
    createdAt: new Date(spaceship.createdAt),
    updatedAt: new Date(spaceship.updatedAt),
  };
}
