import { SpaceshipModel } from '@models';
import type { SpaceshipInventory } from '@repo/types';
import { cloneSpaceship } from './clone-spaceship';
import { updateSpaceshipBySecurityCode } from './update-spaceship-by-security-code';

export async function updateSpaceshipInventoryBySecurityCode(
  securityCode: string,
  inventory: SpaceshipInventory,
) {
  const updatedAt = new Date();
  const spaceship = await updateSpaceshipBySecurityCode(securityCode, {
    inventory,
    updatedAt,
  });
  if (!spaceship) return undefined;

  await SpaceshipModel.updateBySecurityCode(securityCode, {
    inventory,
    updatedAt,
  });
  return cloneSpaceship(spaceship);
}
