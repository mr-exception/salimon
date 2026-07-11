import type { SpaceshipDocument } from '@models';
import { cloneSpaceship } from './clone-spaceship';
import { requireSpaceshipsBySecurityCode } from './state';
import { start } from './start';

export async function updatePropagatedSpaceship(
  spaceship: SpaceshipDocument,
  update: Partial<SpaceshipDocument>,
) {
  await start();
  const spaceships = requireSpaceshipsBySecurityCode();
  const current = spaceships.get(spaceship.securityCode);
  if (!current) return cloneSpaceship(spaceship);

  if (current.updatedAt.getTime() !== spaceship.updatedAt.getTime()) {
    return cloneSpaceship(current);
  }

  const updatedSpaceship = cloneSpaceship({
    ...current,
    ...update,
    position: update.position ?? current.position,
    velocity: update.velocity ?? current.velocity,
    stats: update.stats ?? current.stats,
  });
  spaceships.set(spaceship.securityCode, updatedSpaceship);
  return cloneSpaceship(updatedSpaceship);
}

