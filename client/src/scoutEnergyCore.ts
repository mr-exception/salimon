import type { ResourceStoreCatalog } from './resourceLedger.ts'

export const SCOUT_CORE_FUEL_ID = 'scout:core:fuel'
export const CORE_FUEL_CAPACITY_GRAMS = 50_000
export const CORE_CAPACITY_J = 2_000_000
export const CORE_INITIAL_RESERVE_J = 500_000
export const CORE_MAX_WATTS = 10_000
export const CORE_YIELD_J: Readonly<Record<string, number>> = Object.freeze({ biomass: 2000, mineral: 500 })
// Integer watts over one 60 Hz tick are exactly integer sixtieths of a joule.
export interface CoreEnergyState {
  readonly tick: number
  readonly capacityJ: number
  readonly reserveJ60: number
  readonly conversionBudgetJ60: number
  readonly spentJ60: number
}
export interface CorePowerRequest { readonly watts: number; readonly paused?: boolean }
export type CoreStatus = 'powered' | 'idle' | 'empty' | 'full'
export interface CorePowerResult {
  readonly status: CoreStatus
  readonly consumer: 'idle' | 'powered' | 'paused' | 'insufficient-power'
  readonly drawnJ60: number
  readonly generatedJ: number
  readonly convertedGrams: Readonly<Record<string, number>>
}
const MAX_BUDGET = Math.max(...Object.values(CORE_YIELD_J)) * 60
const integer = (v: unknown, max = Number.MAX_SAFE_INTEGER): number => {
  if (typeof v !== 'number' || !Number.isSafeInteger(v) || v < 0 || v > max) throw new RangeError('Invalid Core energy accounting')
  return v
}
export function freshCoreEnergy(): CoreEnergyState {
  return { tick: 0, capacityJ: CORE_CAPACITY_J, reserveJ60: CORE_INITIAL_RESERVE_J * 60, conversionBudgetJ60: 0, spentJ60: 0 }
}
export function validateCoreEnergy(value: unknown): CoreEnergyState {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new RangeError('Invalid Core state')
  const s = value as Record<string, unknown>
  if (Object.keys(s).sort().join() !== 'capacityJ,conversionBudgetJ60,reserveJ60,spentJ60,tick' || s.capacityJ !== CORE_CAPACITY_J) throw new RangeError('Invalid Core state fields')
  return { tick: integer(s.tick), capacityJ: CORE_CAPACITY_J, reserveJ60: integer(s.reserveJ60, CORE_CAPACITY_J * 60), conversionBudgetJ60: integer(s.conversionBudgetJ60, MAX_BUDGET), spentJ60: integer(s.spentJ60) }
}
/** E02 allowed no production, expenditure or fractional energy. */
export function migrateCoreEnergy(value: unknown): CoreEnergyState {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new RangeError('Invalid legacy Core')
  const s = value as Record<string, unknown>
  if (Object.keys(s).sort().join() !== 'capacityJ,conversionBudgetMicroJ,reserveJ' || s.capacityJ !== CORE_CAPACITY_J || s.reserveJ !== CORE_INITIAL_RESERVE_J || s.conversionBudgetMicroJ !== 0) throw new RangeError('Invalid legacy Core energy')
  return freshCoreEnergy()
}
export function validateCoreConservation(state: CoreEnergyState, converted: Readonly<Record<string, number>>): void {
  validateCoreEnergy(state)
  let generated = 0n
  for (const [id, grams] of Object.entries(converted)) {
    if (!Object.hasOwn(CORE_YIELD_J, id)) throw new RangeError('Unknown Core fuel')
    integer(grams)
    generated += BigInt(grams) * BigInt(CORE_YIELD_J[id]! * 60)
  }
  if (BigInt(CORE_INITIAL_RESERVE_J * 60) + generated !== BigInt(state.reserveJ60) + BigInt(state.spentJ60)) throw new RangeError('Core energy conservation failed')
}
/** Pure one-tick transition. The session alone commits the result and releases power receipts. */
export function stepCoreEnergy(state: CoreEnergyState, fuel: Readonly<Record<string, number>>, request: CorePowerRequest = { watts: 0 }) {
  validateCoreEnergy(state)
  integer(request.watts, CORE_CAPACITY_J * 60)
  if (request.paused !== undefined && typeof request.paused !== 'boolean') throw new RangeError('Invalid Core pause')
  for (const grams of Object.values(fuel)) integer(grams)
  let reserve = state.reserveJ60
  const drawn = !request.paused && request.watts <= reserve ? request.watts : 0
  const consumer: CorePowerResult['consumer'] = request.paused ? 'paused' : request.watts === 0 ? 'idle' : drawn ? 'powered' : 'insufficient-power'
  reserve -= drawn // Approved order: all-or-nothing job draw, then conversion.
  const quantities = { ...fuel }, converted: Record<string, number> = {}
  const ordered = Object.keys(fuel).filter(id => Object.hasOwn(CORE_YIELD_J, id) && fuel[id]! > 0)
    .sort((a, b) => CORE_YIELD_J[b]! - CORE_YIELD_J[a]! || (a < b ? -1 : a > b ? 1 : 0))
  const bound = ordered.length ? CORE_YIELD_J[ordered[0]!]! * 60 : 0
  let budget = Math.min(bound, state.conversionBudgetJ60 + (ordered.length ? CORE_MAX_WATTS : 0))
  let generatedJ = 0
  // At most 4 grams at approved yields; no unbounded catch-up or hidden reservations.
  for (const id of ordered) {
    const yieldJ = CORE_YIELD_J[id]!, cost = yieldJ * 60
    const grams = Math.min(quantities[id]!, Math.floor(budget / cost), Math.floor((CORE_CAPACITY_J * 60 - reserve) / cost))
    if (!grams) continue
    quantities[id] -= grams
    if (!quantities[id]) delete quantities[id]
    converted[id] = grams
    budget -= grams * cost; reserve += grams * cost; generatedJ += grams * yieldJ
  }
  if (!ordered.some(id => quantities[id])) budget = 0
  const core = validateCoreEnergy({ ...state, tick: integer(state.tick + 1), reserveJ60: reserve, conversionBudgetJ60: budget, spentJ60: integer(state.spentJ60 + drawn) })
  const status: CoreStatus = drawn || generatedJ ? 'powered' : reserve === CORE_CAPACITY_J * 60 ? 'full' : ordered.length || reserve ? 'idle' : 'empty'
  return { core, quantities, result: { status, consumer, drawnJ60: drawn, generatedJ, convertedGrams: converted } satisfies CorePowerResult }
}
export const SCOUT_CORE_FUEL_STORE = {
  [SCOUT_CORE_FUEL_ID]: { shipId: 'ship:scout', capacityGrams: CORE_FUEL_CAPACITY_GRAMS },
} as const satisfies ResourceStoreCatalog
