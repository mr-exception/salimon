import type { Inventory } from '@store';

export type RepairKitTier = 't1';
export type FuelCellTier = 't1';

export type FabricatorBlueprint =
  | {
      id: 'fuel-cell-t1';
      name: string;
      summary: string;
      cost: Partial<Inventory>;
      output: {
        type: 'fuel-cell';
        tier: FuelCellTier;
        quantity: number;
        fuelKns: number;
      };
    }
  | {
      id: 'repair-kit-t1';
      name: string;
      summary: string;
      cost: Partial<Inventory>;
      output: {
        type: 'repair-kit';
        tier: RepairKitTier;
        quantity: number;
        repairAmount: number;
      };
    };

export const FABRICATOR_BLUEPRINTS: FabricatorBlueprint[] = [
  {
    id: 'fuel-cell-t1',
    name: 'Feul Cell T1',
    summary: 'Contains 100MN',
    cost: { carbon: 100 },
    output: {
      type: 'fuel-cell',
      tier: 't1',
      quantity: 1,
      fuelKns: 100_000,
    },
  },
  {
    id: 'repair-kit-t1',
    name: 'Repair Kit T1',
    summary: 'Repairs 100 durability',
    cost: { iron: 10 },
    output: {
      type: 'repair-kit',
      tier: 't1',
      quantity: 1,
      repairAmount: 100,
    },
  },
];

export const REPAIR_KIT_TIERS = FABRICATOR_BLUEPRINTS.flatMap((blueprint) =>
  blueprint.output.type === 'repair-kit'
    ? [
        {
          tier: blueprint.output.tier,
          label: blueprint.name,
          repairAmount: blueprint.output.repairAmount,
        },
      ]
    : [],
);

export const FUEL_CELL_TIERS = FABRICATOR_BLUEPRINTS.flatMap((blueprint) =>
  blueprint.output.type === 'fuel-cell'
    ? [
        {
          tier: blueprint.output.tier,
          label: blueprint.name,
          fuelKns: blueprint.output.fuelKns,
        },
      ]
    : [],
);
