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
    activeFeature: cloneActiveFeature(spaceship.activeFeature),
    simulatedAt: cloneDate(spaceship.simulatedAt),
    createdAt: new Date(spaceship.createdAt),
    updatedAt: new Date(spaceship.updatedAt),
  };
}

function cloneActiveFeature(
  activeFeature: SpaceshipDocument['activeFeature'],
): SpaceshipDocument['activeFeature'] {
  if (!activeFeature) return undefined;
  const activeFeatureType = (activeFeature as { type?: unknown }).type;
  if (
    typeof activeFeature === 'object' &&
    'type' in activeFeature &&
    activeFeatureType === 'lock-on'
  ) {
    return undefined;
  }

  if (activeFeature.type === 'target-speed') {
    return {
      ...activeFeature,
      targetVelocity: { ...activeFeature.targetVelocity },
    };
  }

  return {
    ...activeFeature,
    thrusters: activeFeature.thrusters.map((thruster) => ({ ...thruster })),
  };
}
