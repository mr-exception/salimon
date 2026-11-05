import type { ResourceStoreCatalog } from './resourceLedger.ts'
import { STORAGE_CAPACITY } from './scoutLayout.ts'

/** The domain identity is also the existing reachable ship-local fixture ID. */
export const SCOUT_RESOURCE_CONTAINER_ID = 'scout:storage'
export const SCOUT_RESOURCE_STORES = {
  [SCOUT_RESOURCE_CONTAINER_ID]: { shipId: 'ship:scout', capacityGrams: STORAGE_CAPACITY.massG },
} as const satisfies ResourceStoreCatalog
