import { BENCH_STORES } from './gearBench.ts'
import { FABRICATOR_STORES } from './scoutFabricator.ts'
import type { ResourceStoreCatalog } from './resourceLedger.ts'
import { SCOUT_CORE_FUEL_STORE } from './scoutEnergyCore.ts'
import { STORAGE_CAPACITY } from './scoutLayout.ts'

/** The domain identity is also the existing reachable ship-local fixture ID. */
export const SCOUT_RESOURCE_CONTAINER_ID = 'scout:storage'
export const SCOUT_RESOURCE_STORES = {
  [SCOUT_RESOURCE_CONTAINER_ID]: { shipId: 'ship:scout', capacityGrams: STORAGE_CAPACITY.massG },
  ...SCOUT_CORE_FUEL_STORE,
  ...FABRICATOR_STORES,
  ...BENCH_STORES,
} as const satisfies ResourceStoreCatalog
