import test from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession, decodeResourceSave } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID, stepTopDown } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { freshSurvival, METER, completeDiscreteAction, advanceSurvival } from './survival.ts'
const mineral = 'source:landing:mineral:01', biology = 'source:landing:biology:01'
function fixture() {
  let raw: string | null = null, fail = false
  const storage = { getItem: () => raw, setItem: (_: string, value: string) => { if (fail) throw Error('quota'); raw = value } }
  const initial = createTopDownState('exterior', true)
  const world = { ...initial, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'planet:fixture' as const, point: point2(-7.1, 0) } } : e) }
  const make = () => new ResourceSession(storage, world, undefined, undefined, () => 0)
  return { make, session: make(), raw: () => raw, set: (v: string) => { raw = v }, fail: () => { fail = true } }
}
function ticks(s: ResourceSession, count: number) { for (let i = 0; i < count; i++) s.advance(stepTopDown(s.world, [0, 0])) }
function begin(s: ResourceSession) { s.toggleTool(); s.start(mineral) }
test('75% and composed rates, partial work survives cancel/reload; no gram/source duplication', () => {
  for (const [hunger, fatigue, units] of [[METER / 10, METER, 12], [METER, METER / 5, 12], [METER / 10, METER / 5, 9]]) {
    const f = fixture(), s = f.session
    s.survival = { ...freshSurvival(), hunger, fatigue }; begin(s); ticks(s, 37); s.cancel()
    const saved = JSON.parse(f.raw()!)
    assert.equal(saved.miningWork[mineral], 37 * units)
    assert.deepEqual(saved.inventory.quantities, {})
    const reload = f.make(); begin(reload)
    const needed = Math.ceil((960 - 37 * units) / units)
    ticks(reload, needed)
    assert.equal(reload.snapshot.inventory.quantities.mineral, 1000)
    assert.equal(reload.snapshot.sources[mineral]!.remainingGrams, 59000)
    reload.cancel()
    assert.equal(JSON.parse(f.raw()!).miningWork[mineral] ?? 0, (37 + needed) * units - 960)
    const again = f.make(); assert.equal(again.snapshot.inventory.quantities.mineral, 1000)
  }
})
test('threshold crossing uses each tick starting conditions and batch matches individual steps', () => {
  const a = fixture().session, b = fixture().session
  for (const s of [a, b]) { s.survival = { ...freshSurvival(), hunger: METER / 5 + 36, fatigue: METER / 5 + 1 }; begin(s) }
  ticks(a, 61); b.advance({ ...b.world, tick: b.world.tick + 61 })
  a.cancel(); b.cancel()
  assert.deepEqual(a.snapshot, b.snapshot)
  assert.equal(a.snapshot.miningWork[mineral], 16 + 12 + 59 * 9)
})
test('partial work stays with its source and capacity commits exact grams', () => {
  const f = fixture(), s = f.session; begin(s); ticks(s, 59); s.cancel()
  s.world = { ...s.world, entities: s.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'planet:fixture', point: point2(-5, 4.5) } } : e) }
  s.start(biology); ticks(s, 1); s.cancel()
  assert.deepEqual(s.snapshot.miningWork, { [mineral]: 944, [biology]: 16 })
  assert.deepEqual(s.snapshot.inventory.quantities, {})
  const limited = fixture().session
  limited.transfer({ revision: 0, from: { kind: 'source', id: mineral }, to: { kind: 'player' }, resourceId: 'mineral', grams: 9999 })
  begin(limited); ticks(limited, 60)
  assert.equal(limited.snapshot.inventory.quantities.mineral, 10000)
  assert.equal(limited.snapshot.sources[mineral]!.remainingGrams, 50000)
  assert.equal(limited.activeSourceId, null)
})
test('failed extraction retains last whole work/ledger revision; malformed work fails closed; v7 migration has no grants', () => {
  const f = fixture(), s = f.session; begin(s); ticks(s, 30); s.cancel(); const saved = f.raw()!
  s.start(mineral); f.fail(); ticks(s, 30)
  assert.ok(s.error); assert.equal(f.raw(), saved)
  assert.deepEqual(s.snapshot.miningWork, { [mineral]: 480 })
  assert.deepEqual(s.snapshot.inventory.quantities, {})
  for (const work of [{ [mineral]: 960 }, { [mineral]: -1 }, { [mineral]: .5 }, { unknown: 10 }, null]) {
    assert.throws(() => decodeResourceSave(JSON.stringify({ ...JSON.parse(saved), miningWork: work }), s.world))
  }
  const old = { ...JSON.parse(saved), version: 7 }; delete old.scout; delete old.miningWork
  const migrated = decodeResourceSave(JSON.stringify(old), s.world)
  assert.deepEqual(migrated.miningWork, {}); assert.deepEqual(migrated.inventory, JSON.parse(saved).inventory)
})
test('15 active ticks shared machine recovery survives save/reload; emergency always bypasses; rejected actions consume nothing', () => {
  const f = fixture(), s = f.session
  s.survival = completeDiscreteAction(s.survival, 'core-intake')!; s.checkpoint()
  const reload = f.make()
  assert.equal(completeDiscreteAction(reload.survival, 'tray-collect'), null)
  const at14 = advanceSurvival(reload.survival, 14)
  assert.equal(completeDiscreteAction(at14, 'fabricator-submit'), null)
  assert.equal(completeDiscreteAction(advanceSurvival(at14, 1), 'tray-collect')!.actionReadyTick, 30)
  for (const condition of [at14, { ...at14, fatigue: 0, recoveryTicks: 0 }, { ...at14, health: 0 }]) assert.deepEqual(completeDiscreteAction(condition, 'emergency'), condition)
  assert.equal(completeDiscreteAction({ ...freshSurvival(), fatigue: 0 }, 'core-intake'), null)
  assert.throws(() => completeDiscreteAction(freshSurvival(), 'unknown' as never))
})
test('zero elapsed interaction never creates invalid zero work or mints time', () => {
  const f = fixture(), s = f.session; begin(s)
  for (let i = 0; i < 20; i++) s.advance(s.world)
  s.cancel()
  assert.deepEqual(s.snapshot.miningWork, {})
  assert.equal(f.make().error, '')
})
