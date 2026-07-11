import type { SpaceshipDocument } from '@models';
import { cloneSpaceship } from './clone-spaceship';
import { requireSpaceshipsBySecurityCode } from './state';
import { start } from './start';

export async function updateSpaceshipBySecurityCode(
  securityCode: string,
  update: Partial<SpaceshipDocument>,
) {
  await start();
  const spaceships = requireSpaceshipsBySecurityCode();
  const spaceship = spaceships.get(securityCode);
  if (!spaceship) return undefined;

  const updatedSpaceship = cloneSpaceship({
    ...spaceship,
    ...update,
    position: update.position ?? spaceship.position,
    velocity: update.velocity ?? spaceship.velocity,
    stats: update.stats ?? spaceship.stats,
    inventory: update.inventory ?? spaceship.inventory,
  });
  spaceships.set(securityCode, updatedSpaceship);
  return cloneSpaceship(updatedSpaceship);
}
