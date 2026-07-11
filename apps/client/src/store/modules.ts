import { atom, getDefaultStore, useAtomValue, useSetAtom } from 'jotai';
import type { Inventory } from './world';

export type ModuleType = 'mining' | 'thruster';
export type ModuleAttribute = 'efficiency' | 'durability' | 'power';
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
export const MINING_BASE_EFFICIENCY_KNS = 1;
export const MINING_BASE_DURABILITY_KN = 2_000;
export const MINING_DURABILITY_PER_LEVEL_KN = 100;
export const THRUSTER_BASE_POWER_PERCENT = 100;
export const THRUSTER_BASE_DURABILITY = 100;
export const THRUSTER_LEVEL_MULTIPLIER = 0.05;

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
];

const modulesAtom = atom<ShipModule[]>([
  {
    id: 'mining-module-1',
    type: 'mining',
    name: 'Mining module',
    position: { x: 3, y: 3 },
    active: false,
    unlocked: true,
    levels: { efficiency: 1, durability: 1 },
    durability: MINING_BASE_DURABILITY_KN,
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
      name: type === 'thruster' ? 'Thruster module' : 'Mining module',
      position: firstOpenCell,
      active: false,
      unlocked: true,
      levels:
        type === 'thruster'
          ? { power: 1, durability: 1 }
          : { efficiency: 1, durability: 1 },
      durability:
        type === 'thruster'
          ? THRUSTER_BASE_DURABILITY
          : MINING_BASE_DURABILITY_KN,
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
          durability: module.durability + MINING_DURABILITY_PER_LEVEL_KN,
        };
      }

      if (module.type === 'thruster' && attribute === 'durability') {
        return {
          ...nextModule,
          durability: module.durability * (1 + THRUSTER_LEVEL_MULTIPLIER),
        };
      }

      return nextModule;
    }),
  );
}

export function getMiningModuleStats() {
  const module = store
    .get(modulesAtom)
    .find((candidate) => candidate.type === 'mining' && candidate.unlocked);
  if (!module) return undefined;

  const efficiencyLevel = module.levels.efficiency ?? 1;
  const durabilityLevel = module.levels.durability ?? 1;
  return {
    id: module.id,
    active: module.active && module.durability > 0,
    efficiencyKns: MINING_BASE_EFFICIENCY_KNS * efficiencyLevel,
    durability: module.durability,
    maxDurability:
      MINING_BASE_DURABILITY_KN +
      Math.max(0, durabilityLevel - 1) * MINING_DURABILITY_PER_LEVEL_KN,
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
      (attribute === 'efficiency' || attribute === 'durability')) ||
    (type === 'thruster' &&
      (attribute === 'power' || attribute === 'durability'))
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
