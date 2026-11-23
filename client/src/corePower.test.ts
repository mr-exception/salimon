import { test } from 'node:test'
import assert from 'node:assert/strict'
import { CORE_CAPACITY_J, CORE_INITIAL_RESERVE_J, freshCoreEnergy, stepCoreEnergy, validateCoreEnergy, validateCoreConservation } from './scoutEnergyCore.ts'
import { ResourceSession, RESOURCE_SAVE_KEY, type ResourceStorage } from './resourceSession.ts'
import { createTopDownState } from './topDownRuntime.ts'

const run = (ticks: number, state = freshCoreEnergy(), fuel: Record<string, number> = { biomass: 100 }, watts = 0) => {
  let generated = 0, drawn = 0
  for (let i = 0; i < ticks; i++) {
    const next = stepCoreEnergy(state, fuel, { watts })
    state = next.core; fuel = next.quantities
    generated += next.result.generatedJ; drawn += next.result.drawnJ60
  }
  return { state, fuel, generated, drawn }
}
test('Core exact 1/60 clock, watts, whole grams, depletion and bounded accumulation', () => {
  const first = run(1)
  assert.equal(first.state.conversionBudgetJ60, 10000)
  assert.equal(first.generated, 0)
  const before = run(11)
  assert.equal(before.generated, 0)
  assert.equal(run(12).generated, 2000)
  assert.equal(run(60).generated, 10000)
  assert.equal(run(3, freshCoreEnergy(), { mineral: 2 }).generated, 500)
  const depleted = run(60, freshCoreEnergy(), { biomass: 1 })
  assert.deepEqual(depleted.fuel, {})
  assert.equal(depleted.state.conversionBudgetJ60, 0)
  assert.equal(depleted.generated, 2000)
  const empty = run(60, { ...freshCoreEnergy(), reserveJ60: 0 }, {})
  assert.equal(empty.generated, 0)
  assert.equal(stepCoreEnergy(empty.state, {}).result.status, 'empty')
  assert.deepEqual(run(60, freshCoreEnergy(), {}).state, { ...freshCoreEnergy(), tick: 60 })
})
test('fuel ranking, budget/headroom eligibility, map order and untouched input', () => {
  const state = { ...freshCoreEnergy(), conversionBudgetJ60: 120000 }
  const a = stepCoreEnergy(state, { mineral: 4, biomass: 1 })
  const b = stepCoreEnergy(state, { biomass: 1, mineral: 4 })
  assert.deepEqual(a, b)
  assert.deepEqual(a.result.convertedGrams, { biomass: 1 })
  assert.deepEqual(a.quantities, { mineral: 4 })
  const edge = stepCoreEnergy({ ...state, reserveJ60: (CORE_CAPACITY_J - 500) * 60 }, { biomass: 1, mineral: 4 })
  assert.deepEqual(edge.result.convertedGrams, { mineral: 1 })
  const fuel = Object.freeze({ mineral: 1 })
  stepCoreEnergy(state, fuel)
  assert.deepEqual(fuel, { mineral: 1 })
})
test('full reserve, fractional headroom, no backlog burst, draw before generation and pause/resume', () => {
  const full = run(600, { ...freshCoreEnergy(), reserveJ60: CORE_CAPACITY_J * 60 })
  assert.equal(full.generated, 0)
  assert.equal(full.state.conversionBudgetJ60, 120000)
  assert.equal(stepCoreEnergy(full.state, full.fuel).result.status, 'full')
  const paused = stepCoreEnergy(full.state, full.fuel, { watts: 120000, paused: true })
  assert.equal(paused.result.consumer, 'paused')
  assert.equal(paused.result.drawnJ60, 0)
  const resumed = stepCoreEnergy(paused.core, full.fuel, { watts: 120000 })
  assert.equal(resumed.result.consumer, 'powered')
  assert.equal(resumed.result.generatedJ, 2000)
  assert.equal(resumed.core.reserveJ60, CORE_CAPACITY_J * 60)
  const fractional = stepCoreEnergy({ ...freshCoreEnergy(), reserveJ60: CORE_CAPACITY_J * 60 - 29999, conversionBudgetJ60: 30000 }, { mineral: 1 })
  assert.equal(fractional.result.generatedJ, 0)
  assert.equal(fractional.quantities.mineral, 1)
  const insufficient = stepCoreEnergy({ ...freshCoreEnergy(), reserveJ60: 1 }, {}, { watts: 5000 })
  assert.equal(insufficient.result.consumer, 'insufficient-power')
  assert.equal(insufficient.core.reserveJ60, 1)
  assert.equal(insufficient.core.spentJ60, 0)
  assert.equal(run(60, freshCoreEnergy(), {}, 1).state.reserveJ60, CORE_INITIAL_RESERVE_J * 60 - 60)
  const waiting = stepCoreEnergy({ ...freshCoreEnergy(), reserveJ60: 0, conversionBudgetJ60: 20000 }, { mineral: 1 }, { watts: 5000 })
  assert.equal(waiting.result.consumer, 'insufficient-power')
  assert.equal(waiting.result.generatedJ, 500)
  assert.equal(stepCoreEnergy(waiting.core, waiting.quantities, { watts: 5000 }).result.consumer, 'powered')
})
test('malformed state, power, overflow and conservation reject without free energy', () => {
  for (const watts of [-1, 0.5, NaN, Infinity, Number.MAX_SAFE_INTEGER]) assert.throws(() => stepCoreEnergy(freshCoreEnergy(), {}, { watts }))
  for (const bad of [{ reserveJ60: -1 }, { conversionBudgetJ60: 120001 }, { spentJ60: 0.5 }, { tick: -1 }]) assert.throws(() => validateCoreEnergy({ ...freshCoreEnergy(), ...bad }))
  assert.throws(() => stepCoreEnergy({ ...freshCoreEnergy(), spentJ60: Number.MAX_SAFE_INTEGER }, {}, { watts: 1 }))
  assert.throws(() => validateCoreConservation({ ...freshCoreEnergy(), reserveJ60: 30000001 }, {}))
  assert.throws(() => validateCoreConservation(freshCoreEnergy(), { unknown: 1 }))
})
class Storage implements ResourceStorage {
  raw: string | null = null
  fail = false
  getItem(_key?: string) { return this.raw }
  setItem(_key: string, value: string) { if (this.fail) throw Error('crash'); this.raw = value }
}
const open = (storage: Storage) => new ResourceSession(storage, createTopDownState('interior', true), undefined, undefined, () => 0)
const fuelSession = () => {
  const storage = new Storage(), s = open(storage)
  assert(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 10 }))
  assert(s.depositCoreFuel('mineral', 10, s.snapshot.revision))
  return { storage, s }
}
const advance = (s: ResourceSession, ticks = 1, watts = 0) => s.advance({ ...s.world, tick: s.world.tick + ticks }, { watts })
test('session split/batch, reload partial conversion, repeated ticks, withdrawals and exact mass/energy', () => {
  const a = fuelSession(), b = fuelSession()
  const mass = a.s.payload().shipGrams
  advance(a.s, 2)
  assert.equal(a.s.snapshot.core.conversionBudgetJ60, 20000)
  const reopened = open(a.storage)
  assert.equal(reopened.error, '')
  assert.equal(reopened.world.tick, 2)
  advance(reopened)
  assert.equal(reopened.snapshot.convertedGrams.mineral, 1)
  assert.equal(reopened.snapshot.core.reserveJ60, (CORE_INITIAL_RESERVE_J + 500) * 60)
  assert.equal(reopened.payload().shipGrams, mass - 1)
  advance(reopened, 1, 5000)
  assert.equal(reopened.corePower.drawnJ60, 5000)
  const bytes = a.storage.raw, snapshot = reopened.snapshot
  reopened.advance(reopened.world, { watts: 5000 })
  assert.equal(reopened.corePower.drawnJ60, 0)
  reopened.advance({ ...reopened.world, tick: 2 })
  assert.equal(reopened.corePower.drawnJ60, 0)
  assert.equal(reopened.snapshot, snapshot)
  assert.equal(a.storage.raw, bytes)
  advance(b.s, 3)
  advance(b.s, 1, 5000)
  assert.deepEqual(b.s.snapshot.core, reopened.snapshot.core)
  assert.deepEqual(b.s.snapshot.stores, reopened.snapshot.stores)
  assert(reopened.withdrawCoreFuel('mineral', 9, reopened.snapshot.revision))
  assert(!reopened.withdrawCoreFuel('mineral', 1, reopened.snapshot.revision))
  advance(reopened, 60, 5000)
  assert.equal(reopened.snapshot.core.spentJ60, 305000)
  validateCoreConservation(reopened.snapshot.core, reopened.snapshot.convertedGrams)
  assert.deepEqual(open(a.storage).snapshot.core, reopened.snapshot.core)
})
test('atomic failed write/crash/retry, stale writer and invalid saved consumed/energy totals', () => {
  const { storage, s } = fuelSession()
  advance(s, 2)
  const current = s.snapshot, bytes = storage.raw, tick = s.world.tick
  storage.fail = true
  advance(s, 1, 5000)
  assert.equal(s.snapshot, current); assert.equal(storage.raw, bytes); assert.equal(s.world.tick, tick)
  assert.equal(s.corePower.drawnJ60, 0)
  storage.fail = false
  const retry = open(storage), stale = open(storage)
  advance(retry, 1, 5000)
  assert.equal(retry.snapshot.convertedGrams.mineral, 1)
  const committed = storage.raw
  advance(stale, 1, 5000)
  assert.match(stale.error, /Saving failed/)
  assert.equal(storage.raw, committed)
  assert.equal(stale.snapshot.convertedGrams.mineral, undefined)
  assert.equal(open(storage).snapshot.convertedGrams.mineral, 1)
  for (const mutate of [(v: any) => v.convertedGrams.mineral++, (v: any) => v.core.reserveJ60++, (v: any) => v.core.spentJ60++, (v: any) => v.convertedGrams.unknown = 1]) {
    const v = JSON.parse(committed!); mutate(v); storage.raw = JSON.stringify(v)
    const before = storage.raw, invalid = open(storage)
    assert.match(invalid.error, /could not be loaded/); invalid.checkpoint(); assert.equal(storage.raw, before)
  }
})
test('v14 migration preserves original bytes, initial reserve and raw mass', () => {
  const { storage, s } = fuelSession()
  const legacy = JSON.parse(storage.raw!)
  legacy.version = 14; delete legacy.convertedGrams
  legacy.core = { capacityJ: CORE_CAPACITY_J, reserveJ: CORE_INITIAL_RESERVE_J, conversionBudgetMicroJ: 0 }
  storage.raw = JSON.stringify(legacy)
  const bytes = storage.raw, migrated = open(storage)
  assert.equal(migrated.error, '')
  assert.equal(storage.raw, bytes)
  assert.deepEqual(migrated.snapshot.core, freshCoreEnergy())
  assert.equal(migrated.payload().shipGrams, s.payload().shipGrams)
  advance(migrated, 3)
  assert.equal(JSON.parse(storage.getItem(RESOURCE_SAVE_KEY)!).version, 15)
  assert.equal(migrated.snapshot.convertedGrams.mineral, 1)
})

test('consumer debit remains durable in a trusted composition without a fuel account', () => {
  const storage = new Storage()
  const s = new ResourceSession(storage, createTopDownState('interior', true), {}, undefined, () => 0)
  advance(s, 1, 1)
  assert.equal(s.error, '')
  assert.equal(s.corePower.drawnJ60, 1)
  assert.equal(s.snapshot.core.reserveJ60, CORE_INITIAL_RESERVE_J * 60 - 1)
  assert.equal(s.snapshot.core.spentJ60, 1)
  const restored = new ResourceSession(storage, createTopDownState('interior', true), {}, undefined, () => 0)
  assert.deepEqual(restored.snapshot.core, s.snapshot.core)
})
