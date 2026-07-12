import type { SpaceshipInventory } from '@repo/types';
import { SPACESHIP_INVENTORY_MATERIALS } from './materials';

export function normalizeSpaceshipInventory(
  inventory: Partial<SpaceshipInventory> | undefined,
): SpaceshipInventory {
  return Object.fromEntries(
    SPACESHIP_INVENTORY_MATERIALS.map((material) => [
      material,
      inventory?.[material] ?? 0,
    ]),
  ) as SpaceshipInventory;
}
