import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import test from 'node:test'
import assert from 'node:assert/strict'
import { advanceSurvival, freshSurvival, METER, recoverOffline, survivalPolicy, validateSurvival } from './survival.ts'
import { ResourceSession, RESOURCE_SAVE_KEY } from './resourceSession.ts'
import { createTopDownState, stepTopDown } from './topDownRuntime.ts'

test('exact 30 minute hunger, starvation boundary, terminal truncation and partition invariance', () => {
  const initial = freshSurvival()
  const empty = advanceSurvival(initial, 108000)
  assert.equal(empty.hunger, 0); assert.equal(empty.health, 200); assert.equal(empty.starvationTicks, 0)
  assert.equal(advanceSurvival(empty, 599).health, 200)
  assert.equal(advanceSurvival(empty, 600).health, 199)
  assert.deepEqual(advanceSurvival(initial, 108600), advanceSurvival(advanceSurvival(initial, 108001), 599))
  const terminal = advanceSurvival(initial, 1_000_000)
  assert.equal(terminal.health, 0); assert.equal(terminal.activeTicks, 228000)
  assert.deepEqual(advanceSurvival(terminal, 10000), terminal)
  assert.throws(() => advanceSurvival(initial, Infinity))
  assert.throws(() => advanceSurvival({ ...initial, activeTicks: Number.MAX_SAFE_INTEGER }, 1))
})
test('18 hour fatigue and 6 hour offline cap, rollback and no offline hunger/damage', () => {
  // Hunger survival duration is shorter; exercise fatigue from a nearly empty boundary.
  const initial = { ...freshSurvival(1000), fatigue: 60 }
  assert.equal(advanceSurvival(initial, 60).fatigue, 0)
  const drained = { ...freshSurvival(1000), hunger: 0, fatigue: 0, health: 1 }
  const recovered = recoverOffline(drained, 21_601_000)
  assert.equal(recovered.fatigue, METER); assert.equal(recovered.hunger, 0); assert.equal(recovered.health, 1)
  assert.deepEqual(recoverOffline(recovered, 0), recovered)
  assert.deepEqual(recoverOffline(recovered, recovered.offlineAtMs), recovered)
  assert.equal(recoverOffline({ ...drained, health: 0 }, 21_601_000).fatigue, 0)
  assert.equal(3_888_000 / 60, 64_800)
})
test('regen requires a continuous eligible interval and clamps at 200', () => {
  const initial = { ...freshSurvival(), health: 190 }
  assert.equal(advanceSurvival(initial, 3599).health, 190)
  assert.equal(advanceSurvival(initial, 3600).health, 191)
  assert.equal(advanceSurvival(freshSurvival(), 3600).health, 200)
  const boundary = { ...initial, hunger: METER * .7 + 36, recoveryTicks: 3599 }
  assert.equal(advanceSurvival(boundary, 1).health, 191)
  const broken = advanceSurvival({ ...boundary, hunger: METER * .7 }, 1)
  assert.equal(broken.health, 190); assert.equal(broken.recoveryTicks, 0)
  assert.throws(() => validateSurvival({ ...initial, health: 201 }))
  assert.throws(() => validateSurvival({ ...initial, fatigue: -1 }))
  assert.throws(() => validateSurvival({ ...initial, extra: 1 }))
})
test('movement thresholds compose independently; exhaustion and terminal ban work', () => {
  const policy = (h: number, f: number) => survivalPolicy({ ...freshSurvival(), hunger: h * METER / 100, fatigue: f * METER / 100 })
  assert.equal(policy(50, 51).movement, 1)
  assert.equal(policy(49, 50).movement, .9 * .85)
  assert.equal(policy(20, 20).movement, .9 * .65)
  assert.equal(policy(19, 0).movement, .75 * .3)
  assert.equal(policy(19, 0).canWork, false)
  assert.equal(survivalPolicy({ ...freshSurvival(), health: 0 }).movement, 0)
})
test('production session clocks radar once; launch credit durable, failed writes freeze, reset retains ledger', () => {
  let raw: string | null = null, now = 1000, fail = false
  const storage = { getItem: (_key: string) => raw, setItem: (key: string, v: string) => { assert.equal(key, RESOURCE_SAVE_KEY); if (fail) throw Error('quota'); raw = v } }
  const world = createTopDownState('exterior', true)
  const session = new ResourceSession(storage, world, undefined, undefined, () => now)
  for (let i = 0; i < 60; i++) session.advance(stepTopDown(session.world, [0, 0]))
  assert.equal(session.snapshot.survival.activeTicks, 60)
  const saved = raw!
  const radar = { ...session.world, cockpit: { ...session.world.cockpit, view: 'radar' as const }, tick: session.world.tick + 60 }
  session.advance(radar); assert.equal(session.survival.activeTicks, 120)
  session.advance(session.world); assert.equal(session.survival.activeTicks, 120)
  now += 1000
  const reloaded = new ResourceSession(storage, world, undefined, undefined, () => now)
  assert.equal(reloaded.survival.activeTicks, 120)
  assert.equal(reloaded.survival.fatigue, METER)
  const credited = raw
  new ResourceSession(storage, world, undefined, undefined, () => now)
  assert.equal(raw, credited)
  fail = true
  reloaded.advance({ ...reloaded.world, tick: reloaded.world.tick + 60 })
  assert.ok(reloaded.error); assert.equal(raw, credited)
  assert.equal(reloaded.changeItem('add', 'unknown', 1, reloaded.snapshot.revision), false)
  assert.equal(reloaded.advance(stepTopDown(reloaded.world, [1, 0])), reloaded.world)
  fail = false
  const terminalSave = JSON.parse(saved); terminalSave.survival = { ...freshSurvival(now), health: 0 }
  raw = JSON.stringify(terminalSave)
  const terminal = new ResourceSession(storage, world, undefined, undefined, () => now)
  terminal.toggleTool(); terminal.start('source:landing:mineral:01'); assert.equal(terminal.activeSourceId, null)
  const ledger = { sources: terminal.snapshot.sources, inventory: terminal.snapshot.inventory, stores: terminal.snapshot.stores }
  assert.equal(terminal.freshRun(), true)
  assert.equal(terminal.survival.health, 200)
  assert.deepEqual({ sources: terminal.snapshot.sources, inventory: terminal.snapshot.inventory, stores: terminal.snapshot.stores }, ledger)
  assert.equal(terminal.freshRun(), false)
})

test('launch recovery failure and malformed survival preserve original bytes and forbid fresh-run grants', () => {
  let raw: string | null = null, fail = false
  const storage = { getItem: () => raw, setItem: (_: string, value: string) => { if (fail) throw Error('quota'); raw = value } }
  const world = createTopDownState('exterior', true)
  const initial = new ResourceSession(storage, world, undefined, undefined, () => 1000)
  initial.upgrade(10000, 12)
  const valid = JSON.parse(raw!); valid.survival = { ...freshSurvival(1000), fatigue: METER / 2 }
  raw = JSON.stringify(valid); const original = raw
  fail = true
  const blocked = new ResourceSession(storage, world, undefined, undefined, () => 21_601_000)
  assert.match(blocked.error, /Saving failed/)
  assert.equal(blocked.survival.fatigue, METER / 2)
  assert.equal(raw, original); assert.equal(blocked.freshRun(), false)
  fail = false
  for (const condition of [{ ...valid.survival, health: 201 }, { ...valid.survival, hunger: -1 }, { ...valid.survival, fatigue: .5 }, { ...valid.survival, starvationTicks: 1 }]) {
    raw = JSON.stringify({ ...valid, survival: condition }); const bytes = raw
    const broken = new ResourceSession(storage, world, undefined, undefined, () => 1000)
    assert.match(broken.error, /could not be loaded/)
    broken.checkpoint(); assert.equal(broken.freshRun(), false); assert.equal(raw, bytes)
  }
})

test('lethal production tick commits immediately and reload cannot resurrect or gather', () => {
  let raw: string | null = null
  const storage = { getItem: () => raw, setItem: (_: string, value: string) => { raw = value } }
  const session = new ResourceSession(storage, createTopDownState('exterior', true), undefined, undefined, () => 1000)
  session.survival = { ...freshSurvival(1000), hunger: 0, health: 1, starvationTicks: 599 }
  new NavigationRuntime(SAMPLE_BODIES, session.world.scoutLocation).advance(1 / 60)
  session.advance(stepTopDown(session.world, [0, 0]))
  assert.equal(session.snapshot.survival.health, 0)
  const reload = new ResourceSession(storage, session.world, undefined, undefined, () => 1000)
  assert.equal(reload.survival.health, 0)
  assert.equal(reload.transfer({ revision: reload.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 }), false)
  assert.equal(reload.world.tick, session.world.tick)
})

test('v5 migration preserves the entire ledger and initializes no historic active/offline progress', () => {
  let raw: string | null = null
  const storage = { getItem: () => raw, setItem: (_: string, value: string) => { raw = value } }
  const world = createTopDownState('exterior', true)
  const old = new ResourceSession(storage, world, undefined, undefined, () => 1000)
  old.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 333 })
  const v5 = JSON.parse(raw!); v5.version = 5; delete v5.survival; delete v5.miningWork; delete v5.scout; delete v5.hazards
  raw = JSON.stringify(v5); const bytes = raw
  const migrated = new ResourceSession(storage, world, undefined, undefined, () => 21_601_000)
  assert.equal(migrated.error, ''); assert.equal(raw, bytes)
  assert.deepEqual(migrated.survival, freshSurvival(21_601_000))
  assert.deepEqual(migrated.snapshot.inventory, v5.inventory)
  assert.deepEqual(migrated.snapshot.sources, v5.sources)
  assert.deepEqual(migrated.snapshot.stores, v5.stores)
  migrated.advance(stepTopDown(migrated.world, [0, 0])); migrated.checkpoint()
  assert.equal(JSON.parse(raw!).version, 9)
  assert.equal(new ResourceSession(storage, world, undefined, undefined, () => 21_601_000).survival.activeTicks, 1)
})

test('approved mining multipliers use exact integer thresholds and compose; movement floor preserves death ban', () => {
  const policy = (hunger: number, fatigue: number) => survivalPolicy({ ...freshSurvival(), hunger, fatigue })
  const boundary = METER / 5
  assert.equal(policy(boundary, boundary + 1).miningUnits, 16)
  assert.equal(policy(boundary - 1, boundary + 1).miningUnits, 12)
  assert.equal(policy(boundary, boundary).miningUnits, 12)
  assert.equal(policy(boundary - 1, boundary).miningUnits, 9)
  assert.equal(policy(0, 0).miningUnits, 0)
  assert.ok(policy(0, 0).movement >= .1)
  assert.equal(survivalPolicy({ ...freshSurvival(), health: 0 }).miningUnits, 0)
})
