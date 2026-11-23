import type { ResourceStoreCatalog } from './resourceLedger.ts'

/** Core fuel is unconverted raw material, owned by the Scout and never nested in cargo. */
export const SCOUT_CORE_FUEL_ID = 'scout:core:fuel'
export const CORE_FUEL_CAPACITY_GRAMS = 50_000
export const CORE_CAPACITY_J = 2_000_000
export const CORE_INITIAL_RESERVE_J = 500_000

export interface CoreEnergyState {
  readonly capacityJ: number
  readonly reserveJ: number
  /** Integer microjoules accumulated for a future fuel conversion tick. */
  readonly conversionBudgetMicroJ: number
}

export function freshCoreEnergy(): CoreEnergyState {
  return { capacityJ: CORE_CAPACITY_J, reserveJ: CORE_INITIAL_RESERVE_J, conversionBudgetMicroJ: 0 }
}

export function validateCoreEnergy(value: unknown): CoreEnergyState {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new RangeError('Invalid Core state')
  const state = value as Record<string, unknown>
  if (Object.keys(state).sort().join() !== 'capacityJ,conversionBudgetMicroJ,reserveJ') throw new RangeError('Invalid Core state fields')
  if (state.capacityJ !== CORE_CAPACITY_J ||
      typeof state.reserveJ !== 'number' || !Number.isSafeInteger(state.reserveJ) || state.reserveJ < 0 || state.reserveJ > CORE_CAPACITY_J ||
      typeof state.conversionBudgetMicroJ !== 'number' || !Number.isSafeInteger(state.conversionBudgetMicroJ) || state.conversionBudgetMicroJ !== 0) {
    // E02 stores no conversion progress; future conversion will own a bounded accumulator.
    throw new RangeError('Invalid Core energy accounting')
  }
  return { capacityJ: CORE_CAPACITY_J, reserveJ: state.reserveJ, conversionBudgetMicroJ: 0 }
}

export const SCOUT_CORE_FUEL_STORE = {
  [SCOUT_CORE_FUEL_ID]: { shipId: 'ship:scout', capacityGrams: CORE_FUEL_CAPACITY_GRAMS },
} as const satisfies ResourceStoreCatalog
