import { cloneSpaceship } from './clone-spaceship';
import { requireSpaceshipsBySecurityCode } from './state';
import { start } from './start';

export async function findSpaceshipBySecurityCode(securityCode: string) {
  await start();
  const spaceship = requireSpaceshipsBySecurityCode().get(securityCode);
  return spaceship ? cloneSpaceship(spaceship) : undefined;
}

