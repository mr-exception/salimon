import test from 'node:test'
import assert from 'node:assert/strict'
import { freshHazards, hazardRates, stepHazards, validateHazards, type HazardSource } from './hazards.ts'
import { advanceSurvival, freshSurvival, healthValue, setHealth, validateSurvival } from './survival.ts'
import { ResourceSession, RESOURCE_SAVE_KEY } from './resourceSession.ts'
import { createTopDownState } from './topDownRuntime.ts'
import * as q from './exact.ts'
const source = (type: 'radiation' | 'poison', intensity: number, id: string = type): HazardSource => ({ id, type, intensity })
const module = (type: 'temperature' | 'radiation' | 'poison', protection: number, remaining = '1000/1', id: string = type) => ({ id, type, protection, remaining })
function run(sources: HazardSource[], ticks: number, hazards = freshHazards(), survival = freshSurvival()) {
  const rates = hazardRates(sources)
  survival = advanceSurvival(survival, ticks, s => { hazards = stepHazards(hazards, rates, s).state })
  return { hazards, survival }
}
test('HAZ-1 exact decimal same-type addition, once-only protection, permutation and tick partition invariance', () => {
  const hazards = { ...freshHazards(), modules: [module('radiation', .2)] }
  const sources = [source('radiation', .1, 'a'), source('radiation', .2, 'b'), source('poison', .125)]
  const result = run(sources, 60, hazards)
  assert.equal(result.hazards.exposure.radiation, '1/10'); assert.equal(result.hazards.exposure.poison, '1/8')
  assert.equal(result.hazards.modules[0]!.remaining, '4999/5')
  assert.equal(result.survival.health, 200)
  assert.deepEqual(run([...sources].reverse(), 60, hazards), result)
  const first = run(sources, 17, hazards)
  assert.deepEqual(run(sources, 43, first.hazards, first.survival), result)
  const active = { ...freshHazards(), exposure: { radiation: '100/1', poison: '100/1' } }
  const damaged = run(sources, 1, active)
  assert.equal(damaged.survival.healthExact, '479983/2400')
  assert.equal(damaged.survival.health, 200 - .425 / 60)
})
test('HAZ-2 temperature ideal and inclusive 4–44 safe range; exact fractional direct HP', () => {
  for (const temperatureC of [4, 24, 44]) {
    const result = run([{ id: 'temp', type: 'temperature', temperatureC }], 60, { ...freshHazards(), modules: [module('temperature', 2)] })
    assert.equal(result.survival.health, 200)
    assert.equal(result.hazards.modules[0]!.remaining, temperatureC === 24 ? '1000/1' : '998/1')
  }
  const result = run([{ id: 'temp', type: 'temperature', temperatureC: 44.1 }], 60, { ...freshHazards(), modules: [module('temperature', 2)] })
  assert.equal(result.survival.healthExact, '19999/100')
  assert.equal(run([{ id: 'temp', type: 'temperature', temperatureC: 23.9 }], 1).survival.healthExact, '1199999/6000')
})
test('HAZ-2 threshold crosses mid-tick; exit stops damage, independent recovery and residual reentry', () => {
  const initial = { ...freshHazards(), exposure: { radiation: '11999/120', poison: '80/1' }, modules: [module('radiation', 2), module('poison', 1)] }
  const crossed = run([source('radiation', 3)], 1, initial)
  assert.equal(crossed.hazards.exposure.radiation, '100/1')
  assert.equal(crossed.survival.healthExact, '23999/120')
  assert.equal(crossed.hazards.exposure.poison, '799/10')
  const exited = run([], 60, crossed.hazards, crossed.survival)
  assert.equal(exited.hazards.exposure.radiation, '93/1')
  assert.equal(exited.hazards.exposure.poison, '739/10')
  assert.equal(exited.survival.healthExact, crossed.survival.healthExact)
  assert.equal(exited.hazards.modules[0]!.remaining, crossed.hazards.modules[0]!.remaining)
  assert.equal(run([source('radiation', 3)], 60, exited.hazards, exited.survival).hazards.exposure.radiation, '94/1')
  assert.equal(run([], 1200, exited.hazards, exited.survival).hazards.exposure.poison, '0/1')
})
test('HAZ-3 exact full 1000 budget, no double debit at full exposure; mid-tick destruction has no later recovery bonus', () => {
  const initial = { ...freshHazards(), exposure: { radiation: '100/1', poison: '0/1' }, modules: [module('radiation', 1000)] }
  const result = run([source('radiation', 1000)], 60, initial)
  assert.equal(result.hazards.modules[0]!.remaining, '0/1'); assert.equal(result.survival.health, 200)
  const next = run([source('radiation', 6)], 1, result.hazards, result.survival)
  assert.equal(next.survival.healthExact, '1999/10')
  assert.equal(run([], 60, next.hazards, next.survival).hazards.exposure.radiation, '95/1')
  const state = { ...freshHazards(), modules: [module('temperature', 2, '1/60')] }
  const hp = freshSurvival(), step = stepHazards(state, hazardRates([{ id: 'temp', type: 'temperature', temperatureC: 54 }]), hp)
  assert.equal(hp.healthExact, '5999/30'); assert.equal(step.events[0]!.blocked, '1/60')
  assert.deepEqual(step.events[0]!.destroyed, ['temperature'])
  const multi = { ...freshHazards(), modules: [module('poison', 2, '1/120', 'b'), module('poison', 1, '1/100', 'a')] }
  const a = run([source('poison', 4)], 1, multi)
  const b = run([source('poison', 4)], 1, { ...multi, modules: [...multi.modules].reverse() })
  assert.deepEqual(a.hazards.exposure, b.hazards.exposure)
  assert.deepEqual(a.hazards.modules.map(m => m.remaining), b.hazards.modules.map(m => m.remaining).reverse())
})
test('HAZ-4 exact health survives regen/starvation and lethal damage cannot regenerate or advance extra ticks', () => {
  const hp = freshSurvival(); setHealth(hp, q.decode('399/2'))
  assert.equal(advanceSurvival(hp, 3600).health, 200)
  const starving = { ...hp, hunger: 0, recoveryTicks: 0, starvationTicks: 599 }
  assert.equal(advanceSurvival(starving, 1).healthExact, '397/2')
  const result = run([{ id: 'temp', type: 'temperature', temperatureC: 120024 }], 100, freshHazards(), { ...freshSurvival(), recoveryTicks: 3599 })
  assert.equal(result.survival.health, 0); assert.equal(result.survival.activeTicks, 1)
  assert.equal(result.survival.healthExact, undefined)
})
test('HAZ-4 production adapter atomic advance/reload, repeated tick/pause, stale/failed writes preserve exact whole revision', () => {
  let raw: string | null = null, fail = false
  const storage = { getItem: () => raw, setItem: (key: string, value: string) => { assert.equal(key, RESOURCE_SAVE_KEY); if (fail) throw Error('quota'); raw = value } }
  const world = createTopDownState('exterior', true), now = () => 1000
  const session = new ResourceSession(storage, world, undefined, undefined, now)
  assert.equal(session.addProtection({ id: 'fixture', type: 'radiation', protection: .2 }), true)
  const sources = [source('radiation', .3), { id: 'temp', type: 'temperature' as const, temperatureC: 24.1 }]
  session.setHazardSources(sources)
  session.advance({ ...session.world, tick: session.world.tick + 17 })
  assert.equal(session.hazards.exposure.radiation, '17/600')
  assert.equal(session.survival.healthExact, '1199983/6000')
  const first = JSON.parse(raw!)
  session.advance(session.world); session.checkpoint(); assert.deepEqual(JSON.parse(raw!), first)
  const reload = new ResourceSession(storage, world, undefined, undefined, now)
  assert.deepEqual(reload.hazards, session.hazards); assert.deepEqual(reload.survival, session.survival)
  reload.setHazardSources(sources)
  reload.advance({ ...reload.world, tick: reload.world.tick + 43 })
  session.advance({ ...session.world, tick: session.world.tick + 43 }) // stale writer must freeze
  assert.match(session.error, /Saving failed/)
  assert.deepEqual(session.hazards, first.hazards)
  assert.equal(reload.hazards.exposure.radiation, '1/10'); assert.equal(reload.survival.healthExact, '19999/100')
  const committed = raw, before = structuredClone(reload.snapshot), position = reload.world
  fail = true
  reload.advance({ ...reload.world, tick: reload.world.tick + 1 })
  assert.match(reload.error, /Saving failed/); assert.equal(raw, committed)
  assert.deepEqual(reload.hazards, before.hazards); assert.deepEqual(reload.survival, before.survival); assert.equal(reload.world, position)
  assert.deepEqual(reload.hazardEvents, [])
  reload.checkpoint(); assert.equal(raw, committed)
  fail = false
  const resumed = new ResourceSession(storage, world, undefined, undefined, now)
  assert.deepEqual(resumed.hazards, before.hazards)
  // No module is reminted after reload; duplicate ID (including destroyed) is rejected.
  assert.equal(resumed.addProtection({ id: 'fixture', type: 'radiation', protection: 2 }), false)
})
test('HAZ-4 v6 migration preserves existing health/ledger and malformed v7 saves fail closed', () => {
  let raw: string | null = null
  const storage = { getItem: () => raw, setItem: (_: string, v: string) => { raw = v } }, world = createTopDownState('exterior', true)
  const session = new ResourceSession(storage, world, undefined, undefined, () => 1000)
  session.upgrade(10000, 12)
  const old = JSON.parse(raw!); old.version = 6; delete old.miningWork; delete old.hazards; old.survival.health = 167
  raw = JSON.stringify(old)
  const migrated = new ResourceSession(storage, world, undefined, undefined, () => 1000)
  assert.equal(migrated.survival.health, 167); assert.deepEqual(migrated.hazards, freshHazards()); assert.deepEqual(migrated.snapshot.inventory, old.inventory)
  migrated.advance({ ...migrated.world, tick: 1 }); migrated.checkpoint()
  const valid = JSON.parse(raw!)
  for (const mutate of [
    (s: any) => { delete s.hazards }, (s: any) => { s.hazards.exposure.poison = '101/1' },
    (s: any) => { s.hazards.exposure.radiation = '2/2' }, (s: any) => { s.hazards.modules = [module('poison', 2, '1001/1')] },
    (s: any) => { s.survival.health = 166.5 }, (s: any) => { s.survival.healthExact = '333/2' },
  ]) {
    const data = structuredClone(valid); mutate(data); raw = JSON.stringify(data); const bytes = raw
    const broken = new ResourceSession(storage, world, undefined, undefined, () => 1000)
    assert.ok(broken.error); broken.advance({ ...broken.world, tick: 10 }); broken.checkpoint(); assert.equal(raw, bytes)
  }
})
test('HAZ-5 only declared supported sources/modules; reject malformed, duplicate, unsupported and nonfinite data', () => {
  for (const sources of [[source('radiation', -1)], [source('poison', Infinity)], [source('poison', 1), source('poison', 2)], [{ id: 'x', type: 'fire', intensity: 1 }]]) assert.throws(() => hazardRates(sources as HazardSource[]))
  for (const modules of [[module('poison', NaN)], [module('poison', 1), module('poison', 2)], [module('radiation', 1, '-1/1')]]) assert.throws(() => validateHazards({ ...freshHazards(), modules }))
  assert.throws(() => validateSurvival({ ...freshSurvival(), health: 199.5, healthExact: '3990/20' }))
  assert.deepEqual(freshHazards().modules, [])
  const hp = freshSurvival(); setHealth(hp, q.sub(healthValue(hp), q.decimal(.0000001))); validateSurvival(hp)
})

test('HAZ-4 destroyed module tombstones survive save/reload and fresh character; old ticks cannot replay exposure', () => {
  let raw: string | null = null
  const storage = { getItem: () => raw, setItem: (_: string, value: string) => { raw = value } }
  const world = createTopDownState('exterior', true), now = () => 1000
  const session = new ResourceSession(storage, world, undefined, undefined, now)
  session.addProtection({ id: 'fixture', type: 'radiation', protection: 1000 })
  session.setHazardSources([source('radiation', 1000)])
  session.advance({ ...session.world, tick: session.world.tick + 60 })
  const saved = raw
  assert.equal(session.hazards.modules[0]!.remaining, '0/1')
  session.advance({ ...session.world, tick: session.world.tick - 1 })
  assert.equal(raw, saved)
  const reload = new ResourceSession(storage, world, undefined, undefined, now)
  assert.equal(reload.hazards.modules[0]!.remaining, '0/1')
  reload.setHazardSources([{ id: 'lethal', type: 'temperature', temperatureC: 120024 }])
  reload.advance({ ...reload.world, tick: reload.world.tick + 1 })
  assert.equal(reload.survival.health, 0)
  assert.equal(reload.freshRun(), true)
  assert.equal(reload.hazards.modules[0]!.remaining, '0/1')
  assert.equal(reload.addProtection({ id: 'fixture', type: 'radiation', protection: 1000 }), false)
})


test('exact HP below numeric range stays alive until exact lethal damage', () => {
  const hp = freshSurvival()
  setHealth(hp, q.div(q.decimal(Number.MIN_VALUE), q.decimal(60)))
  assert.equal(hp.health, Number.MIN_VALUE)
  validateSurvival(hp)
  const alive = advanceSurvival(hp, 1)
  assert.equal(alive.activeTicks, 1)
  assert.equal(alive.healthExact, hp.healthExact)
  const lethal = run([{ id: 'tiny', type: 'temperature', temperatureC: 24.1 }], 60, freshHazards(), hp)
  assert.equal(lethal.survival.health, 0)
  assert.equal(lethal.survival.activeTicks, 1)
})
