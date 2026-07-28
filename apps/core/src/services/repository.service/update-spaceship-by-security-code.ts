import { SpaceshipModel, type SpaceshipDocument } from '@models';
import { cloneSpaceship } from './clone-spaceship';
import { repositoryState } from './state';

export async function updateSpaceshipBySecurityCode(
  securityCode: string,
  update: Partial<SpaceshipDocument>,
) {
  repositoryState.spaceshipsBySecurityCode ??= new Map();
  const spaceships = repositoryState.spaceshipsBySecurityCode;
  const spaceship =
    spaceships.get(securityCode) ??
    (await SpaceshipModel.findBySecurityCode(securityCode));
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
