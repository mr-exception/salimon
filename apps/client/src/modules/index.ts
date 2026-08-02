import { EnergyCoreModule } from './energy-core-module';
import { FabricatorModule } from './fabricator-module';
import { MiningModule } from './mining-module';
import { ThrusterModule } from './thruster-module';
import {
  DEFAULT_MODULE_UPGRADE_MULTIPLIER,
  multiplyInventoryCost,
} from './upgrade';
import type {
  ModuleAttribute,
  ModuleGridCell,
  ModuleType,
  Inventory,
  ShipModule,
} from './types';
import type { BaseModule } from './base-module';

export * from './base-module';
export * from './energy-core-module';
export * from './fabricator-module';
export * from './mining-module';
export * from './thruster-module';
export type {
  ModuleAttribute,
  ModuleAttributeDefinition,
  ModuleDurabilityDrainRate,
  ModuleDurabilityDrainUnit,
  ModuleGridCell,
  ModuleLevels,
  ModuleType,
  ResearchDefinition,
  ShipModule,
} from './types';
export { DEFAULT_MODULE_UPGRADE_MULTIPLIER, multiplyInventoryCost };

export const MODULE_GRID_SIZE = 8;

export const MODULE_DEFINITIONS = {
  mining: new MiningModule(),
  thruster: new ThrusterModule(),
  fabricator: new FabricatorModule(),
  'energy-core': new EnergyCoreModule(),
} as const satisfies Record<ModuleType, BaseModule>;

export const MINING_MODULE = MODULE_DEFINITIONS.mining;
export const THRUSTER_MODULE = MODULE_DEFINITIONS.thruster;
export const FABRICATOR_MODULE = MODULE_DEFINITIONS.fabricator;
export const ENERGY_CORE_MODULE = MODULE_DEFINITIONS['energy-core'];

export const MINING_BASE_RATE_KG_PER_SECOND =
  MINING_MODULE.getAttributeValue({}, 'efficiency');
export const MINING_BASE_DURABILITY_KG = MINING_MODULE.baseDurability;
export const MINING_DURABILITY_PER_LEVEL_KG =
  MINING_MODULE.getAttributeValue({ durability: 2 }, 'durability') -
  MINING_BASE_DURABILITY_KG;
export const MINING_BASE_RANGE_METERS =
  MINING_MODULE.getAttributeValue({}, 'range');
export const MINING_RANGE_LEVEL_MULTIPLIER =
  MINING_MODULE.getAttribute('range')?.upgradeMultiplier ??
  DEFAULT_MODULE_UPGRADE_MULTIPLIER;

export const THRUSTER_BASE_POWER_KNS =
  THRUSTER_MODULE.getAttributeValue({}, 'power');
export const THRUSTER_BASE_DURABILITY = THRUSTER_MODULE.baseDurability;
export const THRUSTER_DURABILITY_DRAIN_PER_SECOND =
  THRUSTER_MODULE.durabilityDrainPerSecond;
export const THRUSTER_LEVEL_MULTIPLIER =
  THRUSTER_MODULE.getAttribute('power')?.upgradeMultiplier ??
  DEFAULT_MODULE_UPGRADE_MULTIPLIER;

export const FABRICATOR_BASE_DURABILITY = FABRICATOR_MODULE.baseDurability;
export const FABRICATOR_DURABILITY_DRAIN_PER_CRAFT =
  FABRICATOR_MODULE.durabilityDrainPerCraft;
export const FABRICATOR_LEVEL_MULTIPLIER =
  FABRICATOR_MODULE.getAttribute('durability')?.upgradeMultiplier ??
  DEFAULT_MODULE_UPGRADE_MULTIPLIER;

export const ENERGY_CORE_BASE_DURABILITY = ENERGY_CORE_MODULE.baseDurability;
export const ENERGY_CORE_BASE_CAPACITY_KNS =
  ENERGY_CORE_MODULE.getCapacityKns({});
export const ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL =
  ENERGY_CORE_MODULE.durabilityDrainPerRefuel;
export const ENERGY_CORE_CAPACITY_LEVEL_MULTIPLIER =
  ENERGY_CORE_MODULE.getAttribute('capacity')?.upgradeMultiplier ??
  DEFAULT_MODULE_UPGRADE_MULTIPLIER;

export const MODULE_DURABILITY_CONFIG = {
  mining: {
    baseDurability: MINING_BASE_DURABILITY_KG,
    usageDrainRatePerSecond: MINING_BASE_RATE_KG_PER_SECOND,
  },
  thruster: {
    baseDurability: THRUSTER_BASE_DURABILITY,
    usageDrainRatePerSecond: THRUSTER_DURABILITY_DRAIN_PER_SECOND,
  },
  fabricator: {
    baseDurability: FABRICATOR_BASE_DURABILITY,
    usageDrainRatePerSecond: 0,
  },
  'energy-core': {
    baseDurability: ENERGY_CORE_BASE_DURABILITY,
    usageDrainRatePerSecond: 0,
  },
} as const;

export const MODULE_RESEARCH = Object.values(MODULE_DEFINITIONS).map(
  (definition) => definition.research,
);

export function getModuleDefinition(type: ModuleType) {
  return MODULE_DEFINITIONS[type];
}

export function createModule(type: ModuleType, position: ModuleGridCell) {
  return getModuleDefinition(type).create(`${type}-module-1`, position);
}

export function getModuleMaxDurabilityFromDefinition(module: ShipModule) {
  return getModuleDefinition(module.type).getMaxDurability(module.levels);
}

export function getModuleDurabilityDrainRateFromDefinition(
  module: ShipModule,
) {
  return getModuleDefinition(module.type).getDurabilityDrainRate(module);
}

export function upgradeModuleFromDefinition(
  module: ShipModule,
  attribute: ModuleAttribute,
) {
  return getModuleDefinition(module.type).upgrade(module, attribute);
}

export function getModuleUpgradeCost(
  type: ModuleType,
  attribute: ModuleAttribute,
  nextLevel: number,
): Partial<Inventory> {
  return getModuleDefinition(type).getUpgradeCost(attribute, nextLevel);
}
