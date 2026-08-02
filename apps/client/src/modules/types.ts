import type { SpaceshipInventory } from '@repo/types';

export type Inventory = SpaceshipInventory;

export type ModuleType = 'mining' | 'thruster' | 'fabricator' | 'energy-core';
export type ModuleAttribute =
  | 'efficiency'
  | 'durability'
  | 'range'
  | 'power'
  | 'capacity';
export type ModuleGridCell = {
  x: number;
  y: number;
};
export type ModuleLevels = Partial<Record<ModuleAttribute, number>>;
export type ShipModule = {
  id: string;
  type: ModuleType;
  name: string;
  position: ModuleGridCell;
  active: boolean;
  unlocked: boolean;
  levels: ModuleLevels;
  durability: number;
};
export type ModuleDurabilityDrainUnit = 'second' | 'craft' | 'refuel';
export type ModuleDurabilityDrainRate = {
  amount: number;
  unit: ModuleDurabilityDrainUnit;
};
export type ModuleAttributeDefinition = {
  attribute: ModuleAttribute;
  label: string;
  defaultValue: number;
  materials: Partial<Inventory>;
  upgradeMultiplier?: number;
};
export type ResearchDefinition = {
  module: ModuleType;
  name: string;
  cost: Partial<Inventory>;
};
