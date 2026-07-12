import type { InventoryMaterial } from '@repo/types';

export const SPACESHIP_INVENTORY_MATERIALS = [
  'iron',
  'silicates',
  'ice',
  'silver',
  'carbon',
  'gold',
  'hydrogen',
  'nitrogen',
] as const satisfies readonly InventoryMaterial[];
