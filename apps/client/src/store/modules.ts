import { atom, getDefaultStore, useAtomValue, useSetAtom } from 'jotai';
import {
  ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL,
  FABRICATOR_DURABILITY_DRAIN_PER_CRAFT,
  MINING_MODULE,
  THRUSTER_MODULE,
  createModule,
  getModuleDefinition,
  getModuleDurabilityDrainRateFromDefinition,
  getModuleMaxDurabilityFromDefinition,
  MODULE_GRID_SIZE,
  upgradeModuleFromDefinition,
  type ModuleAttribute,
  type ModuleGridCell,
  type ModuleType,
  type ShipModule,
} from '../modules';

const store = getDefaultStore();

export * from '../modules';
export * from './module-config';

function createStarterModules() {
  return [];
}

const modulesAtom = atom<ShipModule[]>(createStarterModules());

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
  return getModuleMaxDurabilityFromDefinition(module);
}

export function getModuleDurabilityDrainRate(module: ShipModule) {
  return getModuleDurabilityDrainRateFromDefinition(module);
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
    createModule(type, firstOpenCell),
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

      return upgradeModuleFromDefinition(module, attribute);
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

  return {
    id: module.id,
    active: module.active && module.durability > 0,
    rateKgPerSecond: MINING_MODULE.getRateKgPerSecond(module.levels),
    rangeMeters: MINING_MODULE.getRangeMeters(module.levels),
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

export function getThrusterModulePowerKns() {
  const module = store
    .get(modulesAtom)
    .find((candidate) => candidate.type === 'thruster' && candidate.unlocked);
  return THRUSTER_MODULE.getPowerKns(module?.levels ?? {});
}

function canUpgradeAttribute(type: ModuleType, attribute: ModuleAttribute) {
  return getModuleDefinition(type).canUpgrade(attribute);
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
