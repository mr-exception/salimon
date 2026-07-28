import { SpaceshipModel } from '@models';
import { cloneSpaceship } from './clone-spaceship';
import { repositoryState } from './state';

export async function findSpaceshipBySecurityCode(securityCode: string) {
  const cachedSpaceship =
    repositoryState.spaceshipsBySecurityCode?.get(securityCode);
  if (cachedSpaceship) return cloneSpaceship(cachedSpaceship);

  const spaceship = await SpaceshipModel.findBySecurityCode(securityCode);
  if (!spaceship) return undefined;

  repositoryState.spaceshipsBySecurityCode ??= new Map();
  repositoryState.spaceshipsBySecurityCode.set(
    spaceship.securityCode,
    cloneSpaceship(spaceship),
  );

  return cloneSpaceship(spaceship);
}
