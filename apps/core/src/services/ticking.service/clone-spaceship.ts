import type { SpaceshipDocument } from '@models';

function cloneDate(value: Date | undefined) {
  return value ? new Date(value) : undefined;
}

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
    simulatedAt: cloneDate(spaceship.simulatedAt),
    createdAt: new Date(spaceship.createdAt),
    updatedAt: new Date(spaceship.updatedAt),
  };
}
