import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession, ITEM_CATALOG } from './resourceSession.ts'
import { createTopDownState } from './topDownRuntime.ts'
import { FABRICATOR_INPUT_ID, FABRICATOR_RECIPE_ID, MARKER_KIT_ID, FABRICATOR_TICKS, FABRICATOR_WATTS, freshFabricator, stepFabricator, validateFabricator } from './scoutFabricator.ts'

function setup() {
  const storage = { raw: null as string | null, fail: false, getItem: () => storage.raw, setItem: (_: string, raw: string) => { if (storage.fail) throw Error('quota'); storage.raw = raw } }
  const open = () => new ResourceSession(storage, createTopDownState('interior', true), undefined, { ...ITEM_CATALOG, ballast: { gramsPerItem: 1, stackSize: 1 } }, () => 0)
  const s = open()
  const gather = (grams: number) => s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams })
  const deposit = (grams: number) => s.depositFabricatorInput('mineral', grams, s.snapshot.revision)
  const start = () => s.startFabricator(FABRICATOR_RECIPE_ID, s.snapshot.revision)
  return { s, storage, open, gather, deposit, start }
}
const advance = (s: ResourceSession, ticks: number) => s.advance({ ...s.world, tick: s.world.tick + ticks })
const input = (s: ResourceSession) => s.snapshot.stores[FABRICATOR_INPUT_ID]!.quantities.mineral ?? 0

test('Fabricator exact 50 kg input cap includes reserved grams and only unreserved withdrawals', () => {
  const { s, gather, deposit, start } = setup()
  for (let i = 0; i < 5; i++) { assert.equal(gather(10000), true); assert.equal(deposit(10000), true) }
  assert.equal(input(s), 50000)
  assert.equal(gather(1), true)
  assert.equal(deposit(1), false)
  assert.equal(start(), true)
  assert.equal(input(s), 49000)
  assert.equal(deposit(1), false) // Reservation still occupies input buffer capacity.
  assert.equal(s.withdrawFabricatorInput('mineral', 49001, s.snapshot.revision), false)
  assert.equal(s.withdrawFabricatorInput('mineral', 9999, s.snapshot.revision), true)
  assert.equal(s.snapshot.fabricator.job!.reservedGrams, 1000)
  assert.equal(s.payload().shipGrams, 50001)
})

test('partial ingredients, unknown/multiple recipes, stale starts and generic output grants cannot mutate', () => {
  const { s, storage, gather, deposit, start } = setup()
  gather(999); deposit(999)
  const before = storage.raw
  assert.equal(start(), false)
  assert.equal(s.startFabricator('toString', s.snapshot.revision), false)
  assert.equal(s.changeItem('add', MARKER_KIT_ID, 1, s.snapshot.revision), false)
  // Generic item mint attempts are rejected without touching storage.
  assert.equal(storage.raw, before)
  const other = setup(); other.gather(2000); other.deposit(2000)
  assert.equal(other.s.startFabricator(FABRICATOR_RECIPE_ID, other.s.snapshot.revision - 1), false)
  assert.equal(other.start(), true)
  const active = other.storage.raw
  other.s.cancel('Interaction aborted; committed job remains resumable.')
  assert.equal(other.s.snapshot.fabricator.job!.remainingTicks, 1200)
  assert.equal(other.start(), false)
  assert.equal(other.s.startFabricator('unapproved:v2', other.s.snapshot.revision), false)
  assert.equal(other.storage.raw, active)
})

test('paid ticks complete once into persistent tray; exact mass/count and 100 kJ; deliberate claim only', () => {
  const { s, gather, deposit, start, open } = setup()
  gather(2000); deposit(2000); start()
  const mass = s.payload().shipGrams
  advance(s, 1199)
  assert.equal(s.snapshot.fabricator.job!.remainingTicks, 1)
  assert.equal(s.snapshot.fabricator.tray, null)
  assert.equal(s.snapshot.core.spentJ60, 1199 * FABRICATOR_WATTS)
  advance(s, 1)
  assert.equal(s.snapshot.fabricator.job, null)
  assert.equal(s.snapshot.fabricator.tray!.jobId, 1)
  assert.deepEqual(s.snapshot.inventory.items, {})
  assert.equal(s.snapshot.core.spentJ60, 100000 * 60)
  assert.equal(s.payload().shipGrams, mass)
  advance(s, 60)
  assert.equal(s.snapshot.fabricator.completed, 1)
  assert.equal(start(), false)
  const restored = open()
  assert.equal(restored.error, '')
  const rev = restored.snapshot.revision
  assert.equal(restored.claimFabricator(2, rev), false)
  assert.equal(restored.claimFabricator(1, rev - 1), false)
  assert.equal(restored.claimFabricator(1, rev), true)
  assert.equal(restored.claimFabricator(1, rev), false)
  assert.equal(restored.claimFabricator(1, restored.snapshot.revision), false)
  assert.equal(restored.snapshot.inventory.items[MARKER_KIT_ID], 1)
  assert.equal(restored.payload().shipGrams, mass)
  assert.equal(restored.startFabricator(FABRICATOR_RECIPE_ID, restored.snapshot.revision), true)
  advance(restored, 1200)
  assert.equal(restored.claimFabricator(2, restored.snapshot.revision), true)
  assert.equal(restored.snapshot.inventory.items[MARKER_KIT_ID], 2)
  assert.equal(restored.payload().shipGrams, mass)
})

test('power interruption persists reservation and paused work; fuel resumes on following paid tick', () => {
  const { s, gather, deposit, start, open } = setup()
  gather(1001); deposit(1000)
  s.advance({ ...s.world, tick: 1 }, { watts: s.snapshot.core.reserveJ60 })
  assert.equal(s.snapshot.core.reserveJ60, 0)
  start(); advance(s, 3)
  assert.equal(s.snapshot.fabricator.job!.remainingTicks, 1200)
  assert.equal(s.snapshot.fabricator.job!.powerPaused, true)
  const restored = open()
  assert.deepEqual(restored.snapshot.fabricator, s.snapshot.fabricator)
  assert.equal(restored.depositCoreFuel('mineral', 1, restored.snapshot.revision), true)
  advance(restored, 3)
  assert.equal(restored.snapshot.fabricator.job!.remainingTicks, 1200)
  advance(restored, 1)
  assert.equal(restored.snapshot.fabricator.job!.remainingTicks, 1199)
  assert.equal(restored.snapshot.fabricator.job!.powerPaused, false)
  assert.equal(restored.snapshot.core.reserveJ60, 500 * 60 - 5000)
})

test('claim with full mass or slots leaves tray, receipts and saved bytes intact', () => {
  for (const mode of ['mass', 'slots']) {
    const { s, storage, gather, deposit, start } = setup()
    gather(1000); deposit(1000); start(); advance(s, 1200)
    if (mode === 'mass') gather(10000)
    else assert.equal(s.changeItem('add', 'ballast', 12, s.snapshot.revision), true)
    const before = storage.raw, snapshot = s.snapshot
    assert.equal(s.claimFabricator(1, s.snapshot.revision), false)
    assert.equal(s.snapshot, snapshot); assert.equal(storage.raw, before)
    if (mode === 'mass') deposit(1000)
    else s.changeItem('remove', 'ballast', 1, s.snapshot.revision)
    assert.equal(s.claimFabricator(1, s.snapshot.revision), true)
  }
})

test('failed/stale persistence aborts start, progress and claim atomically without duplication on reopen', () => {
  for (const stage of ['start', 'progress', 'claim']) for (const failure of ['quota', 'stale']) {
    const { s, storage, open, gather, deposit, start } = setup()
    gather(1000); deposit(1000)
    if (stage !== 'start') start()
    if (stage === 'claim') advance(s, 1200)
    const snapshot = s.snapshot, world = s.world
    if (failure === 'quota') storage.fail = true
    else storage.raw += ' '
    const before = storage.raw
    if (stage === 'start') assert.equal(start(), false)
    if (stage === 'progress') advance(s, 1)
    if (stage === 'claim') assert.equal(s.claimFabricator(1, s.snapshot.revision), false)
    assert.equal(s.snapshot, snapshot); assert.equal(storage.raw, before)
    if (stage === 'progress') assert.equal(s.world, world)
    storage.fail = false
    assert.equal(open().error, '')
    assert.deepEqual(open().snapshot.fabricator, snapshot.fabricator)
  }
})

test('save rejects unpaid work, forged reservations, outputs, claims, recipes, receipt IDs and unsupported fields without replacing bytes', () => {
  const { s, storage, gather, deposit, start, open } = setup()
  gather(2000); deposit(2000); start(); advance(s, 10)
  const raw = storage.raw!
  for (const corrupt of [
    (v: any) => { delete v.fabricator },
    (v: any) => { v.fabricator.job.recipeId = 'untrusted' },
    (v: any) => { v.fabricator.job.reservedGrams = 999 },
    (v: any) => { v.fabricator.job.remainingTicks = 1 },
    (v: any) => { v.fabricator.job.remainingTicks = 1201 },
    (v: any) => { v.fabricator.job.powerPaused = 'yes' },
    (v: any) => { v.fabricator.started = 2 },
    (v: any) => { v.fabricator.job.id = 2 },
    (v: any) => { v.fabricator.claimed = 1 },
    (v: any) => { v.fabricator.tray = { jobId: 1, itemId: MARKER_KIT_ID, count: 1 } },
    (v: any) => { v.inventory.items[MARKER_KIT_ID] = 1 },
    (v: any) => { v.fabricator.extra = true },
    (v: any) => { v.version = 17 },
  ]) {
    const v = JSON.parse(raw); corrupt(v); storage.raw = JSON.stringify(v)
    const before = storage.raw, invalid = open()
    assert.match(invalid.error, /could not be loaded/)
    invalid.checkpoint(); assert.equal(storage.raw, before)
  }
  storage.raw = raw
  assert.equal(open().snapshot.fabricator.job!.remainingTicks, 1190)
})

test('v15 migrates empty Fabricator and trusted kit metadata without modifying original bytes', () => {
  const { s, storage, open, gather } = setup(); gather(100)
  const old = JSON.parse(storage.raw!); old.version = 15; delete old.fabricator; delete old.stores[FABRICATOR_INPUT_ID]; delete old.itemDefinitions[MARKER_KIT_ID]
  storage.raw = JSON.stringify(old)
  const before = storage.raw, restored = open()
  assert.equal(restored.error, '')
  assert.equal(storage.raw, before)
  assert.deepEqual(restored.snapshot.fabricator, freshFabricator())
  assert.deepEqual(restored.snapshot.inventory, s.snapshot.inventory)
  restored.checkpoint()
  assert.equal(storage.raw, before)
  assert.equal(restored.depositFabricatorInput('mineral', 100, restored.snapshot.revision), true)
  assert.equal(JSON.parse(storage.getItem()!).version, 16)
})

test('pure Fabricator rejects partial power receipts, validates bounded state, and split ticks match uninterrupted work', () => {
  const f = validateFabricator({ ...freshFabricator(), started: 1, job: { id: 1, recipeId: FABRICATOR_RECIPE_ID, reservedGrams: 1000, remainingTicks: FABRICATOR_TICKS, powerPaused: false } })
  assert.throws(() => stepFabricator(f, 4999))
  assert.throws(() => validateFabricator({ ...f, started: Number.MAX_SAFE_INTEGER }))
  const initial = structuredClone(f)
  let state = validateFabricator(f)
  for (let i = 0; i < 1200; i++) { state = stepFabricator(state, 5000); if (i === 37) state = validateFabricator(JSON.parse(JSON.stringify(state))) }
  let uninterrupted = validateFabricator(initial)
  for (let i = 0; i < 1200; i++) uninterrupted = stepFabricator(uninterrupted, 5000)
  assert.deepEqual(state, uninterrupted)
  assert.equal(state.completed, 1)
})


test('non-Fabricator power receipts cannot fund forged fabrication progress, including v15 migration', () => {
  const { s, storage, open, gather, deposit } = setup()
  gather(1000); deposit(1000)
  s.advance({ ...s.world, tick: 1 }, { watts: 1000000 })
  assert.equal(s.snapshot.fabricator.otherSpentJ60, 1000000)
  const old = JSON.parse(storage.raw!); old.version = 15; delete old.fabricator; delete old.stores[FABRICATOR_INPUT_ID]; delete old.itemDefinitions[MARKER_KIT_ID]
  // This legacy fixture must preserve raw conservation: return its input to player.
  old.inventory.quantities.mineral = 1000
  storage.raw = JSON.stringify(old)
  const migrated = open()
  assert.equal(migrated.error, '')
  assert.equal(migrated.snapshot.fabricator.otherSpentJ60, 1000000)
  // Use a separate live session to preserve its compare-and-replace token.
  const next = setup(); next.gather(1000); next.deposit(1000)
  next.s.advance({ ...next.s.world, tick: 1 }, { watts: 1000000 }); next.start()
  const corrupt = JSON.parse(next.storage.raw!); corrupt.fabricator.job.remainingTicks--
  next.storage.raw = JSON.stringify(corrupt)
  assert.match(next.open().error, /Unpaid Fabricator work/)
})
