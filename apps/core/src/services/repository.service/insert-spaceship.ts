import type { SpaceshipDocument } from '@models';
import { cloneSpaceship } from './clone-spaceship';
import { requireSpaceshipsBySecurityCode } from './state';
import { start } from './start';

export async function insertSpaceship(spaceship: SpaceshipDocument) {
  await start();
  requireSpaceshipsBySecurityCode().set(
    spaceship.securityCode,
    cloneSpaceship(spaceship),
  );
  return cloneSpaceship(spaceship);
}

