import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession as LiveResourceSession, RESOURCE_CATALOG, resourcePayload, withResourcePayload } from './resourceSession.ts'
class ResourceSession extends LiveResourceSession {
  constructor(...args: ConstructorParameters<typeof LiveResourceSession>) { args[4] = () => 0; super(...args) }
}
import { registerCountItems, resourceInventoryUsage } from './resourceInventory.ts'
import { createTopDownState, PLAYER_ID, stepTopDown } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { initialShip } from './simulation.ts'
import { TEST_ITEMS } from '../tests/itemFixtures.ts'
const world = () => {
  const w = createTopDownState('exterior', true)
  return { ...w, entities: w.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'planet:fixture' as const, point: point2(-7.1, 0) } } : e) }
}
function make() {
  const storage = { raw: null as string | null, fail: false, getItem: () => storage.raw, setItem: (_: string, raw: string) => { if (storage.fail) throw Error('quota'); storage.raw = raw } }
  const session = new ResourceSession(storage, world(), undefined, TEST_ITEMS)
  const change = (action: 'add' | 'remove', id: string, count: number) => session.changeItem(action, id, count, session.snapshot.revision)
  return { storage, session, change }
}
const usage = (s: ResourceSession) => resourceInventoryUsage(s.snapshot.inventory, RESOURCE_CATALOG, s.itemCatalog)
test('registered catalogs validate unused definitions and snapshot immutable authored metadata', () => {
  for (const grams of [0, -1, .5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1]) assert.throws(() => registerCountItems({ x: { gramsPerItem: grams, stackSize: 1 } }))
  for (const stack of [0, -1, .5, Infinity]) assert.throws(() => registerCountItems({ x: { gramsPerItem: 1, stackSize: stack } }))
  for (const value of [null, [], { '': { gramsPerItem: 1, stackSize: 1 } }, { x: null }, { x: { gramsPerItem: 1, stackSize: 1, volume: 1 } }]) assert.throws(() => registerCountItems(value as any))
  const input = { x: { gramsPerItem: 123, stackSize: 3 } }, catalog = registerCountItems(input)
  input.x.gramsPerItem = 0
  assert.equal(catalog.x!.gramsPerItem, 123); assert.equal(Object.isFrozen(catalog.x), true)
})
test('shared stack slots, exact mixed mass, removal, full reload and onboard mass', () => {
  const { session, storage, change } = make()
  assert.equal(change('add', 'fixture-single', 2), true)
  assert.equal(change('add', 'fixture-stack', 5), true)
  assert.deepEqual(usage(session), { grams: 1500, slots: 4 })
  assert.equal(session.transfer({ revision: 2, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 333 }), true)
  assert.deepEqual(usage(session), { grams: 1833, slots: 5 })
  const reload = new ResourceSession(storage, world(), undefined, TEST_ITEMS)
  assert.equal(reload.error, ''); assert.deepEqual(reload.snapshot, session.snapshot)
  assert.equal(reload.changeItem('remove', 'fixture-stack', 1, reload.snapshot.revision), true)
  assert.deepEqual(usage(reload), { grams: 1583, slots: 4 })
  assert.equal(reload.changeItem('remove', 'fixture-single', 2, reload.snapshot.revision), true)
  assert.deepEqual(usage(reload), { grams: 1333, slots: 2 })
  const aboard = { ...reload.world, entities: reload.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(0, -3.1) } } : e) }
  assert.equal(resourcePayload(reload.snapshot.inventory, aboard, reload.itemCatalog).shipGrams, 1333)
  assert.equal(withResourcePayload(initialShip(), reload.snapshot.inventory, aboard, reload.itemCatalog).massKg, 100001.333)
})
test('malformed requests, unknown IDs, excess removal, stale revisions and overfill never publish', () => {
  const { session, storage, change } = make()
  change('add', 'fixture-single', 12)
  const before = session.snapshot, raw = storage.raw
  for (const [id, count] of [['unknown', 1], ['toString', 1], ['fixture-stack', 1], ['fixture-single', 0], ['fixture-single', -1], ['fixture-single', .5], ['fixture-single', Number.MAX_SAFE_INTEGER]] as const) assert.equal(change('add', id, count), false)
  assert.equal(change('remove', 'fixture-single', 13), false)
  assert.equal(session.changeItem('remove', 'fixture-single', 1, 0), false)
  assert.equal(session.snapshot, before); assert.equal(storage.raw, raw)
  change('remove', 'fixture-single', 12); change('add', 'fixture-stack', 12) // 3 kg
  const prior = session.snapshot
  assert.equal(change('add', 'fixture-stack', 29), false) // 10.25 kg, within slots
  assert.equal(session.snapshot, prior)
})
test('gathering honors combined slot and exact remaining gram limits without failed-source debit', () => {
  for (const mode of ['slots', 'mass']) {
    const { session, change } = make()
    change('add', mode === 'slots' ? 'fixture-single' : 'fixture-stack', 12)
    session.toggleTool(); session.start('source:landing:mineral:01')
    for (let i = 0; i < 600; i++) session.advance(stepTopDown(session.world, [0, 0]))
    const collected = mode === 'slots' ? 0 : 7000
    assert.equal(session.snapshot.inventory.quantities.mineral ?? 0, collected)
    assert.equal(session.snapshot.sources['source:landing:mineral:01']!.remainingGrams, 60000 - collected)
    assert.match(session.feedback, mode === 'slots' ? /slot limit/ : /weight limit/)
    change('remove', mode === 'slots' ? 'fixture-single' : 'fixture-stack', 1)
    session.start('source:landing:mineral:01')
    for (let i = 0; i < 60; i++) session.advance(stepTopDown(session.world, [0, 0]))
    assert.equal(session.snapshot.inventory.quantities.mineral, collected + (mode === 'slots' ? 1000 : 250))
  }
})
test('quota and stale writes keep item changes atomic and block subsequent writes', () => {
  for (const mode of ['quota', 'stale']) {
    const { storage, session, change } = make(); change('add', 'fixture-stack', 5)
    const before = session.snapshot
    if (mode === 'quota') storage.fail = true
    else storage.raw += ' '
    const raw = storage.raw
    assert.equal(change('remove', 'fixture-stack', 1), false)
    assert.equal(session.snapshot, before); assert.equal(storage.raw, raw); assert.match(session.error, /Saving failed/)
    assert.equal(change('add', 'fixture-single', 1), false)
  }
})
test('corrupt saved definitions/counts and unsupported migrations preserve original bytes', () => {
  const { storage, session, change } = make(); change('add', 'fixture-stack', 5)
  const original = storage.raw!
  for (const mutate of [
    (s: any) => { s.itemDefinitions['fixture-stack'].gramsPerItem = 0 },
    (s: any) => { s.itemDefinitions['fixture-stack'].gramsPerItem++ },
    (s: any) => { s.itemDefinitions['fixture-stack'].stackSize = 0.5 },
    (s: any) => { delete s.itemDefinitions['fixture-single'] },
    (s: any) => { s.itemDefinitions = null },
    (s: any) => { s.inventory.items['fixture-stack'] = 0 },
    (s: any) => { s.inventory.items['fixture-stack'] = 1.5 },
    (s: any) => { s.inventory.items.unknown = 1 },
    (s: any) => { s.inventory.items = JSON.parse('{"__proto__":1}') },
    (s: any) => { s.version = 4; delete s.itemDefinitions },
  ]) {
    const s = JSON.parse(original); mutate(s); storage.raw = JSON.stringify(s)
    const raw = storage.raw, restored = new ResourceSession(storage, session.world, undefined, TEST_ITEMS)
    assert.match(restored.error, /could not be loaded/)
    assert.equal(restored.changeItem('add', 'fixture-single', 1, 0), false)
    restored.checkpoint(); assert.equal(storage.raw, raw)
  }
  storage.raw = original
  assert.match(new ResourceSession(storage, world()).error, /could not be loaded/) // fixtures unavailable to ordinary game
})
test('v4 empty-item saves migrate losslessly; failed migration commit preserves v4 bytes', () => {
  const { storage } = make()
  const ordinary = new ResourceSession(storage, world()); ordinary.upgrade(10000, 12)
  const old = JSON.parse(storage.raw!); old.version = 4; delete old.itemDefinitions; delete old.survival; delete old.hazards
  storage.raw = JSON.stringify(old); const raw = storage.raw
  const migrated = new ResourceSession(storage, world(), undefined, TEST_ITEMS)
  assert.equal(migrated.error, ''); assert.equal(migrated.snapshot.version, 7)
  assert.deepEqual(migrated.snapshot.inventory, old.inventory); assert.equal(storage.raw, raw)
  storage.fail = true; assert.equal(migrated.changeItem('add', 'fixture-single', 1, old.revision), false); assert.equal(storage.raw, raw)
  storage.fail = false
  const retry = new ResourceSession(storage, world(), undefined, TEST_ITEMS)
  assert.equal(retry.changeItem('add', 'fixture-single', 1, old.revision), true)
  assert.equal(JSON.parse(storage.getItem()!).version, 7)
})
