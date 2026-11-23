import type { ResourceStoreCatalog } from './resourceLedger.ts'

export const FABRICATOR_INPUT_ID = 'scout:fabricator:input'
export const MARKER_KIT_ID = 'field-marker-kit:v1'
export const FABRICATOR_RECIPE_ID = MARKER_KIT_ID
export const FABRICATOR_GRAMS = 1000
export const FABRICATOR_TICKS = 1200
export const FABRICATOR_WATTS = 5000
export const FABRICATOR_ITEMS = { [MARKER_KIT_ID]: { name: 'Field marker kit', gramsPerItem: 1000, stackSize: 1 } } as const
export const FABRICATOR_STORES = { [FABRICATOR_INPUT_ID]: { shipId: 'ship:scout', capacityGrams: 50_000 } } as const satisfies ResourceStoreCatalog
export interface FabricatorJob {
  readonly id: number
  readonly recipeId: typeof FABRICATOR_RECIPE_ID
  readonly reservedGrams: number
  readonly remainingTicks: number
  readonly powerPaused: boolean
}
/** Monotonic transaction receipts bind each start, completion and claim to a job ID. */
export interface FabricatorState {
  /** Historical/non-Fabricator Core expenditure, excluded from paid job ticks. */
  readonly otherSpentJ60: number
  readonly started: number
  readonly completed: number
  readonly claimed: number
  readonly job: FabricatorJob | null
  readonly tray: { readonly jobId: number; readonly itemId: typeof MARKER_KIT_ID; readonly count: 1 } | null
}
export const freshFabricator = (otherSpentJ60 = 0): FabricatorState => ({ otherSpentJ60, started: 0, completed: 0, claimed: 0, job: null, tray: null })
function record(v: unknown, keys: string): Record<string, unknown> {
  if (!v || typeof v !== 'object' || Array.isArray(v) || Object.keys(v).sort().join() !== keys) throw Error('Invalid Fabricator fields')
  return v as Record<string, unknown>
}
function integer(v: unknown): asserts v is number {
  if (typeof v !== 'number' || !Number.isSafeInteger(v) || v < 0) throw Error('Invalid Fabricator integer')
}
export function validateFabricator(value: unknown): FabricatorState {
  const s = record(value, 'claimed,completed,job,otherSpentJ60,started,tray')
  for (const v of [s.started, s.completed, s.claimed, s.otherSpentJ60]) integer(v)
  const { started, completed, claimed } = s as unknown as FabricatorState
  if (!Number.isSafeInteger(started * FABRICATOR_TICKS * FABRICATOR_WATTS) || completed > started || claimed > completed || started - completed > 1 || completed - claimed > 1) throw Error('Invalid Fabricator receipts')
  if ((s.job !== null) !== (started > completed) || (s.tray !== null) !== (completed > claimed) || (s.job && s.tray)) throw Error('Invalid Fabricator ownership')
  if (s.job !== null) {
    const j = record(s.job, 'id,powerPaused,recipeId,remainingTicks,reservedGrams')
    integer(j.remainingTicks)
    if (j.id !== started || j.recipeId !== FABRICATOR_RECIPE_ID || j.reservedGrams !== FABRICATOR_GRAMS || !j.remainingTicks || j.remainingTicks > FABRICATOR_TICKS || typeof j.powerPaused !== 'boolean') throw Error('Invalid Fabricator job')
  }
  if (s.tray !== null) {
    const t = record(s.tray, 'count,itemId,jobId')
    if (t.jobId !== completed || t.itemId !== MARKER_KIT_ID || t.count !== 1) throw Error('Invalid Fabricator tray')
  }
  return structuredClone(value) as FabricatorState
}
export function fabricatorSpentJ60(s: FabricatorState): number {
  return (s.completed * FABRICATOR_TICKS + (s.job ? FABRICATOR_TICKS - s.job.remainingTicks : 0)) * FABRICATOR_WATTS
}
export function stepFabricator(s: FabricatorState, drawnJ60: number): FabricatorState {
  validateFabricator(s)
  integer(drawnJ60)
  if (!s.job) {
    if (!drawnJ60) return s
    const otherSpentJ60 = s.otherSpentJ60 + drawnJ60
    integer(otherSpentJ60)
    return { ...s, otherSpentJ60 }
  }
  if (drawnJ60 !== FABRICATOR_WATTS && drawnJ60 !== 0) throw Error('Invalid Fabricator power receipt')
  if (!drawnJ60) return { ...s, job: { ...s.job, powerPaused: true } }
  if (s.job.remainingTicks > 1) return { ...s, job: { ...s.job, remainingTicks: s.job.remainingTicks - 1, powerPaused: false } }
  return { ...s, completed: s.completed + 1, job: null, tray: { jobId: s.started, itemId: MARKER_KIT_ID, count: 1 } }
}
