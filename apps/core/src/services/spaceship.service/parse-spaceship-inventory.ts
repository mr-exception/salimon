import { INVENTORY_MATERIALS, type SpaceshipInventory } from '@repo/types';

export function parseSpaceshipInventory(body: unknown): SpaceshipInventory {
  if (!body || typeof body !== 'object') {
    throw new Error('inventory must be an object');
  }

  const candidate = body as Record<string, unknown>;
  return Object.fromEntries(
    INVENTORY_MATERIALS.map((material) => {
      const amount = candidate[material];
      if (
        typeof amount !== 'number' ||
        !Number.isFinite(amount) ||
        amount < 0
      ) {
        throw new Error(`inventory.${material} must be a non-negative number`);
      }

      return [material, Math.floor(amount)];
    }),
  ) as SpaceshipInventory;
}
