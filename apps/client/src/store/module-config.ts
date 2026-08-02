import type { Inventory } from './world';
import {
  DEFAULT_MODULE_UPGRADE_MULTIPLIER,
  ENERGY_CORE_BASE_CAPACITY_KNS,
  ENERGY_CORE_BASE_DURABILITY,
  ENERGY_CORE_CAPACITY_LEVEL_MULTIPLIER,
  ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL,
  FABRICATOR_BASE_DURABILITY,
  FABRICATOR_DURABILITY_DRAIN_PER_CRAFT,
  FABRICATOR_LEVEL_MULTIPLIER,
  MINING_BASE_DURABILITY_KG,
  MINING_BASE_RANGE_METERS,
  MINING_BASE_RATE_KG_PER_SECOND,
  MINING_DURABILITY_PER_LEVEL_KG,
  MINING_RANGE_LEVEL_MULTIPLIER,
  MODULE_DURABILITY_CONFIG,
  MODULE_GRID_SIZE,
  MODULE_RESEARCH,
  THRUSTER_BASE_DURABILITY,
  THRUSTER_BASE_POWER_KNS,
  THRUSTER_DURABILITY_DRAIN_PER_SECOND,
  THRUSTER_LEVEL_MULTIPLIER,
  getModuleDefinition,
  getModuleUpgradeCost,
  multiplyInventoryCost,
  type ModuleAttribute,
  type ModuleAttributeDefinition,
  type ModuleType,
} from '../modules';

export {
  DEFAULT_MODULE_UPGRADE_MULTIPLIER,
  ENERGY_CORE_BASE_CAPACITY_KNS,
  ENERGY_CORE_BASE_DURABILITY,
  ENERGY_CORE_CAPACITY_LEVEL_MULTIPLIER,
  ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL,
  FABRICATOR_BASE_DURABILITY,
  FABRICATOR_DURABILITY_DRAIN_PER_CRAFT,
  FABRICATOR_LEVEL_MULTIPLIER,
  MINING_BASE_DURABILITY_KG,
  MINING_BASE_RANGE_METERS,
  MINING_BASE_RATE_KG_PER_SECOND,
  MINING_DURABILITY_PER_LEVEL_KG,
  MINING_RANGE_LEVEL_MULTIPLIER,
  MODULE_DURABILITY_CONFIG,
  MODULE_GRID_SIZE,
  MODULE_RESEARCH,
  THRUSTER_BASE_DURABILITY,
  THRUSTER_BASE_POWER_KNS,
  THRUSTER_DURABILITY_DRAIN_PER_SECOND,
  THRUSTER_LEVEL_MULTIPLIER,
  getModuleUpgradeCost,
  multiplyInventoryCost,
};

export type ConfiguredModuleType = ModuleType | 'hull';
export type ModuleAttributeConfig = Omit<
  ModuleAttributeDefinition,
  'attribute' | 'label'
>;
export type ConfiguredModule = {
  attributes: Partial<Record<ModuleAttribute, ModuleAttributeConfig>>;
};
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
export const MODULE_CONFIGS: Record<ConfiguredModuleType, ConfiguredModule> = {
  hull: {
    attributes: {
      durability: {
        defaultValue: 200,
        materials: { iron: 40, silicates: 18, carbon: 8 },
        upgradeMultiplier: 0.25,
      },
    },
  },
  mining: toConfiguredModule('mining'),
  thruster: toConfiguredModule('thruster'),
  fabricator: toConfiguredModule('fabricator'),
  'energy-core': toConfiguredModule('energy-core'),
};

export function getConfiguredModuleAttribute(
  type: ConfiguredModuleType,
  attribute: ModuleAttribute,
) {
  return MODULE_CONFIGS[type].attributes[attribute];
}

export function getConfiguredModuleAttributeValue(
  type: ConfiguredModuleType,
  attribute: ModuleAttribute,
  level: number,
) {
  const config = getConfiguredModuleAttribute(type, attribute);
  if (!config) return 0;

  return (
    config.defaultValue *
    (1 +
      Math.max(0, level - 1) *
        (config.upgradeMultiplier ?? DEFAULT_MODULE_UPGRADE_MULTIPLIER))
  );
}

export function getConfiguredModuleUpgradeMultiplier(
  type: ConfiguredModuleType,
  attribute: ModuleAttribute,
) {
  return (
    getConfiguredModuleAttribute(type, attribute)?.upgradeMultiplier ??
    DEFAULT_MODULE_UPGRADE_MULTIPLIER
  );
}

export function getHullUpgradeCost(nextLevel: number): Partial<Inventory> {
  const config = getConfiguredModuleAttribute('hull', 'durability');
  if (!config) return {};

  return multiplyInventoryCost(
    config.materials,
    nextLevel,
    config.upgradeMultiplier ?? DEFAULT_MODULE_UPGRADE_MULTIPLIER,
  );
}

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

function toConfiguredModule(type: ModuleType): ConfiguredModule {
  return {
    attributes: Object.fromEntries(
      getModuleDefinition(type).attributes.map((definition) => [
        definition.attribute,
        {
          defaultValue: definition.defaultValue,
          materials: definition.materials,
          upgradeMultiplier:
            'upgradeMultiplier' in definition
              ? definition.upgradeMultiplier
              : undefined,
        },
      ]),
    ) as ConfiguredModule['attributes'],
  };
}
