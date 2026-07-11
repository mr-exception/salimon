import { INVENTORY_MATERIALS, type SpaceshipInventory } from '@repo/types';

export function normalizeSpaceshipInventory(
  inventory: Partial<SpaceshipInventory> | undefined,
): SpaceshipInventory {
  return Object.fromEntries(
    INVENTORY_MATERIALS.map((material) => [
      material,
      inventory?.[material] ?? 0,
    ]),
  ) as SpaceshipInventory;
}
