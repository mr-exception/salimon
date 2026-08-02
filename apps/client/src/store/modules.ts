import { atom, getDefaultStore, useAtomValue, useSetAtom } from 'jotai';
import type { Inventory } from './world';

export type ModuleType = 'mining' | 'thruster' | 'fabricator' | 'energy-core';
export type ModuleAttribute = 'efficiency' | 'durability' | 'range' | 'power';
export type ModuleGridCell = {
  x: number;
  y: number;
};
export type ShipModule = {
  id: string;
  type: ModuleType;
  name: string;
  position: ModuleGridCell;
  active: boolean;
  unlocked: boolean;
  levels: Partial<Record<ModuleAttribute, number>>;
  durability: number;
};
export type ResearchDefinition = {
  module: ModuleType;
  name: string;
  cost: Partial<Inventory>;
};

const store = getDefaultStore();

export const MODULE_GRID_SIZE = 8;
export const MINING_BASE_RATE_KG_PER_SECOND = 2;
export const MINING_BASE_DURABILITY_KG = 2_000;
export const MINING_DURABILITY_PER_LEVEL_KG = 100;
export const MINING_BASE_RANGE_METERS = 50_000;
export const MINING_RANGE_LEVEL_MULTIPLIER = 0.05;
export const THRUSTER_BASE_POWER_PERCENT = 100;
export const THRUSTER_BASE_DURABILITY = 100;
export const THRUSTER_DURABILITY_DRAIN_PER_SECOND = 0.02;
export const THRUSTER_LEVEL_MULTIPLIER = 0.05;
export const FABRICATOR_BASE_DURABILITY = 500;
export const FABRICATOR_DURABILITY_DRAIN_PER_CRAFT = 1;
export const FABRICATOR_LEVEL_MULTIPLIER = 0.05;
export const ENERGY_CORE_BASE_DURABILITY = 800;
export const ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL = 1;
export const ENERGY_CORE_LEVEL_MULTIPLIER = 0.05;

export const MODULE_DURABILITY_CONFIG: Record<
  ModuleType,
  { baseDurability: number; usageDrainRatePerSecond: number }
> = {
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
};

export const MODULE_RESEARCH: ResearchDefinition[] = [
  {
    module: 'mining',
    name: 'Mining module',
    cost: { iron: 0, silicates: 0, ice: 0 },
  },
  {
    module: 'thruster',
    name: 'Thruster module',
    cost: { iron: 120, silicates: 80, ice: 40 },
  },
  {
    module: 'fabricator',
    name: 'Fabricator module',
    cost: { iron: 80, silicates: 40, carbon: 20 },
  },
  {
    module: 'energy-core',
    name: 'Energy Core module',
    cost: { iron: 140, silicates: 100, carbon: 50 },
  },
];

const modulesAtom = atom<ShipModule[]>([
  {
    id: 'mining-module-1',
    type: 'mining',
    name: 'Mining module',
    position: { x: 3, y: 3 },
    active: false,
    unlocked: true,
    levels: { efficiency: 1, durability: 1, range: 1 },
    durability: MINING_BASE_DURABILITY_KG,
  },
  {
    id: 'fabricator-module-1',
    type: 'fabricator',
    name: 'Fabricator module',
    position: { x: 4, y: 3 },
    active: false,
    unlocked: true,
    levels: { durability: 1 },
    durability: FABRICATOR_BASE_DURABILITY,
  },
  {
    id: 'energy-core-module-1',
    type: 'energy-core',
    name: 'Energy Core module',
    position: { x: 4, y: 4 },
    active: false,
    unlocked: true,
    levels: { durability: 1 },
    durability: ENERGY_CORE_BASE_DURABILITY,
  },
]);

export function useModules() {
  return useAtomValue(modulesAtom);
}

export function useSetModules() {
  return useSetAtom(modulesAtom);
}

export function getModules() {
  return store.get(modulesAtom);
}

export function getModuleMaxDurability(module: ShipModule) {
  const durabilityLevel = module.levels.durability ?? 1;
  if (module.type === 'mining') {
    return (
      MINING_BASE_DURABILITY_KG +
      Math.max(0, durabilityLevel - 1) * MINING_DURABILITY_PER_LEVEL_KG
    );
  }

  if (module.type === 'fabricator') {
    return (
      FABRICATOR_BASE_DURABILITY *
      (1 + Math.max(0, durabilityLevel - 1) * FABRICATOR_LEVEL_MULTIPLIER)
    );
  }

  if (module.type === 'energy-core') {
    return (
      ENERGY_CORE_BASE_DURABILITY *
      (1 + Math.max(0, durabilityLevel - 1) * ENERGY_CORE_LEVEL_MULTIPLIER)
    );
  }

  return (
    THRUSTER_BASE_DURABILITY *
    (1 + Math.max(0, durabilityLevel - 1) * THRUSTER_LEVEL_MULTIPLIER)
  );
}

export function setModuleActive(moduleId: string, active: boolean) {
  store.set(
    modulesAtom,
    store
      .get(modulesAtom)
      .map((module) =>
        module.id === moduleId &&
        module.unlocked &&
        module.durability > 0 &&
        module.position.x >= 0 &&
        module.position.y >= 0
          ? { ...module, active }
          : module,
      ),
  );
}

export function placeModule(moduleId: string, position: ModuleGridCell) {
  if (!isInsideModuleGrid(position) || isModuleGridCellOccupied(position)) {
    return false;
  }

  store.set(
    modulesAtom,
    store
      .get(modulesAtom)
      .map((module) =>
        module.id === moduleId ? { ...module, position } : module,
      ),
  );
  return true;
}

export function unlockModule(type: ModuleType) {
  const currentModules = store.get(modulesAtom);
  if (currentModules.some((module) => module.type === type)) return false;

  const firstOpenCell = getFirstOpenModuleGridCell(currentModules);
  if (!firstOpenCell) return false;

  store.set(modulesAtom, [
    ...currentModules,
    {
      id: `${type}-module-1`,
      type,
      name:
        type === 'thruster'
          ? 'Thruster module'
          : type === 'fabricator'
            ? 'Fabricator module'
            : type === 'energy-core'
              ? 'Energy Core module'
              : 'Mining module',
      position: firstOpenCell,
      active: false,
      unlocked: true,
      levels:
        type === 'thruster'
          ? { power: 1, durability: 1 }
          : type === 'fabricator'
            ? { durability: 1 }
            : type === 'energy-core'
              ? { durability: 1 }
              : { efficiency: 1, durability: 1, range: 1 },
      durability:
        type === 'thruster'
          ? THRUSTER_BASE_DURABILITY
          : type === 'fabricator'
            ? FABRICATOR_BASE_DURABILITY
            : type === 'energy-core'
              ? ENERGY_CORE_BASE_DURABILITY
              : MINING_BASE_DURABILITY_KG,
    },
  ]);
  return true;
}

export function upgradeModuleAttribute(
  moduleId: string,
  attribute: ModuleAttribute,
) {
  store.set(
    modulesAtom,
    store.get(modulesAtom).map((module) => {
      if (
        module.id !== moduleId ||
        !canUpgradeAttribute(module.type, attribute)
      ) {
        return module;
      }

      const nextLevel = (module.levels[attribute] ?? 0) + 1;
      const nextModule = {
        ...module,
        levels: {
          ...module.levels,
          [attribute]: nextLevel,
        },
      };

      if (module.type === 'mining' && attribute === 'durability') {
        return {
          ...nextModule,
          durability: module.durability + MINING_DURABILITY_PER_LEVEL_KG,
        };
      }

      if (module.type === 'thruster' && attribute === 'durability') {
        return {
          ...nextModule,
          durability: module.durability * (1 + THRUSTER_LEVEL_MULTIPLIER),
        };
      }

      if (module.type === 'fabricator' && attribute === 'durability') {
        return {
          ...nextModule,
          durability: module.durability * (1 + FABRICATOR_LEVEL_MULTIPLIER),
        };
      }

      if (module.type === 'energy-core' && attribute === 'durability') {
        return {
          ...nextModule,
          durability: module.durability * (1 + ENERGY_CORE_LEVEL_MULTIPLIER),
        };
      }

      return nextModule;
    }),
  );
}

export function repairModule(moduleId: string) {
  store.set(
    modulesAtom,
    store
      .get(modulesAtom)
      .map((module) =>
        module.id === moduleId
          ? { ...module, durability: getModuleMaxDurability(module) }
          : module,
      ),
  );
}

export function repairModuleByAmount(moduleId: string, amount: number) {
  if (!Number.isFinite(amount) || amount <= 0) return false;

  let repaired = false;
  store.set(
    modulesAtom,
    store.get(modulesAtom).map((module) => {
      if (module.id !== moduleId) return module;

      const maxDurability = getModuleMaxDurability(module);
      const nextDurability = Math.min(
        maxDurability,
        module.durability + amount,
      );
      repaired = nextDurability > module.durability;
      return { ...module, durability: nextDurability };
    }),
  );
  return repaired;
}

export function consumeFabricatorDurability(moduleId: string, craftCount = 1) {
  if (!Number.isFinite(craftCount) || craftCount <= 0) return false;

  const requestedDurability =
    FABRICATOR_DURABILITY_DRAIN_PER_CRAFT * craftCount;
  let consumed = false;
  store.set(
    modulesAtom,
    store.get(modulesAtom).map((module) => {
      if (
        module.id !== moduleId ||
        module.type !== 'fabricator' ||
        module.durability < requestedDurability
      ) {
        return module;
      }

      consumed = true;
      return {
        ...module,
        durability: Math.max(0, module.durability - requestedDurability),
      };
    }),
  );
  return consumed;
}

export function consumeEnergyCoreDurability(moduleId: string, refuelCount = 1) {
  if (!Number.isFinite(refuelCount) || refuelCount <= 0) return false;

  const requestedDurability =
    ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL * refuelCount;
  let consumed = false;
  store.set(
    modulesAtom,
    store.get(modulesAtom).map((module) => {
      if (
        module.id !== moduleId ||
        module.type !== 'energy-core' ||
        module.durability < requestedDurability
      ) {
        return module;
      }

      consumed = true;
      return {
        ...module,
        durability: Math.max(0, module.durability - requestedDurability),
      };
    }),
  );
  return consumed;
}

export function getMiningModuleStats() {
  const module = store
    .get(modulesAtom)
    .find((candidate) => candidate.type === 'mining' && candidate.unlocked);
  if (!module) return undefined;

  const efficiencyLevel = module.levels.efficiency ?? 1;
  const rangeLevel = module.levels.range ?? 1;
  return {
    id: module.id,
    active: module.active && module.durability > 0,
    rateKgPerSecond: MINING_BASE_RATE_KG_PER_SECOND * efficiencyLevel,
    rangeMeters:
      MINING_BASE_RANGE_METERS *
      (1 + Math.max(0, rangeLevel - 1) * MINING_RANGE_LEVEL_MULTIPLIER),
    durability: module.durability,
    maxDurability: getModuleMaxDurability(module),
  };
}

export function consumeMiningDurability(requestedKnSeconds: number) {
  if (!Number.isFinite(requestedKnSeconds) || requestedKnSeconds <= 0) return 0;

  let consumedKnSeconds = 0;
  store.set(
    modulesAtom,
    store.get(modulesAtom).map((module) => {
      if (
        module.type !== 'mining' ||
        !module.active ||
        module.durability <= 0
      ) {
        return module;
      }

      consumedKnSeconds = Math.min(module.durability, requestedKnSeconds);
      return {
        ...module,
        active: module.durability - consumedKnSeconds > 0,
        durability: Math.max(0, module.durability - consumedKnSeconds),
      };
    }),
  );
  return consumedKnSeconds;
}

export function getThrusterModulePowerPercent() {
  const module = store
    .get(modulesAtom)
    .find((candidate) => candidate.type === 'thruster' && candidate.unlocked);
  const powerLevel = module?.levels.power ?? 1;

  return THRUSTER_BASE_POWER_PERCENT * (1 + (powerLevel - 1) * 0.05);
}

function canUpgradeAttribute(type: ModuleType, attribute: ModuleAttribute) {
  return (
    (type === 'mining' &&
      (attribute === 'efficiency' ||
        attribute === 'durability' ||
        attribute === 'range')) ||
    (type === 'thruster' &&
      (attribute === 'power' || attribute === 'durability')) ||
    (type === 'fabricator' && attribute === 'durability') ||
    (type === 'energy-core' && attribute === 'durability')
  );
}

function isInsideModuleGrid(position: ModuleGridCell) {
  return (
    Number.isInteger(position.x) &&
    Number.isInteger(position.y) &&
    position.x >= 0 &&
    position.y >= 0 &&
    position.x < MODULE_GRID_SIZE &&
    position.y < MODULE_GRID_SIZE
  );
}

function isModuleGridCellOccupied(position: ModuleGridCell) {
  return store
    .get(modulesAtom)
    .some(
      (module) =>
        module.position.x === position.x && module.position.y === position.y,
    );
}

function getFirstOpenModuleGridCell(modules: ShipModule[]) {
  for (let y = 0; y < MODULE_GRID_SIZE; y += 1) {
    for (let x = 0; x < MODULE_GRID_SIZE; x += 1) {
      if (
        !modules.some(
          (module) => module.position.x === x && module.position.y === y,
        )
      ) {
        return { x, y };
      }
    }
  }

  return undefined;
}
