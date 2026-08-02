import type { Inventory } from './types';

export const DEFAULT_MODULE_UPGRADE_MULTIPLIER = 0.05;

export function multiplyInventoryCost(
  multipliers: Partial<Inventory>,
  level: number,
  levelMultiplier = 1,
): Partial<Inventory> {
  const multiplier = 1 + Math.max(0, level - 1) * levelMultiplier;

  return Object.fromEntries(
    Object.entries(multipliers).map(([material, amount]) => [
      material,
      Math.ceil((amount ?? 0) * multiplier),
    ]),
  ) as Partial<Inventory>;
}
