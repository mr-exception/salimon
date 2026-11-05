import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession, RESOURCE_SAVE_KEY, type ResourceStorage } from './resourceSession.ts'
import { SCOUT_RESOURCE_CONTAINER_ID as id } from './scoutResourceContainer.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { DRY_MASS_KG, initialShip, setMass } from './simulation.ts'
import { transferResource, validateResourceLedger, type ResourceLedger } from './resourceLedger.ts'
import { startingResourceInventory } from './resourceInventory.ts'
class Storage implements ResourceStorage {
  raw: string | null = null
  fail = false
  getItem(key: string) { assert.equal(key, RESOURCE_SAVE_KEY); return this.raw }
  setItem(key: string, value: string) { assert.equal(key, RESOURCE_SAVE_KEY); if (this.fail) throw Error('quota'); this.raw = value }
}
const sourceId = 'source:landing:mineral:01'
function make() {
  const storage = new Storage(), session = new ResourceSession(storage, createTopDownState('exterior', true))
  const collect = (grams: number) => session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id: sourceId }, to: { kind: 'player' }, resourceId: 'mineral', grams })
  return { storage, session, collect }
}
test('Scout has an empty identified 50 kg raw-only store; invalid commands leave bytes/state untouched', () => {
  const { session, storage } = make()
  assert.deepEqual(session.snapshot.stores[id], { id, shipId: 'ship:scout', capacityGrams: 50000, quantities: {} })
  for (const resource of ['unknown', 'suit', '__proto__', 'mineral']) for (const grams of [-1, 0, .5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1, 1]) {
    const before = session.snapshot
    assert.equal(session.deposit(resource, grams, before.revision), false)
    assert.equal(session.withdraw(resource, grams, before.revision), false)
    assert.equal(session.snapshot, before); assert.equal(storage.raw, null)
  }
})
test('exact repeated deposits fill 50 kg; explicit partial withdrawals respect player weight/revision and reload', () => {
  const { session, storage, collect } = make()
  for (let i = 0; i < 5; i++) { assert.equal(collect(10000), true); assert.equal(session.deposit('mineral', 10000, session.snapshot.revision), true) }
  assert.equal(collect(1000), true)
  const before = session.snapshot, raw = storage.raw
  for (const request of [() => session.deposit('mineral', 1, before.revision), () => session.withdraw('mineral', 9001, before.revision), () => session.withdraw('mineral', 333, before.revision - 1)]) {
    assert.equal(request(), false); assert.equal(session.snapshot, before); assert.equal(storage.raw, raw)
  }
  assert.equal(session.withdraw('mineral', 333, before.revision), true)
  assert.equal(session.snapshot.inventory.quantities.mineral, 1333)
  assert.equal(session.snapshot.stores[id]!.quantities.mineral, 49667)
  assert.equal(session.snapshot.sources[sourceId]!.remainingGrams, 9000)
  const restored = new ResourceSession(storage, createTopDownState('exterior', true))
  assert.equal(restored.error, ''); assert.deepEqual(restored.snapshot, session.snapshot)
})
test('container has no slot cap, rejects count metadata; player shared slots still constrain withdrawals', () => {
  const resources = Object.fromEntries(Array.from({ length: 13 }, (_, i) => ['r' + i, {}]))
  const sources = Object.fromEntries(Object.keys(resources).map(r => [r, { id: r, resourceId: r, remainingGrams: 1 }]))
  const stores = { [id]: { shipId: 'ship:scout', capacityGrams: 50000 } }
  let ledger: ResourceLedger = { revision: 0, sources, inventory: startingResourceInventory(), stores: { [id]: { ...stores[id], id, quantities: {} } } }
  for (const r of Object.keys(resources)) ledger = transferResource(ledger, { revision: ledger.revision, from: { kind: 'source', id: r }, to: { kind: 'store', id }, resourceId: r, grams: 1 }, sources, stores, resources)
  for (const r of Object.keys(resources).slice(0, 12)) ledger = transferResource(ledger, { revision: ledger.revision, from: { kind: 'store', id }, to: { kind: 'player' }, resourceId: r, grams: 1 }, sources, stores, resources)
  assert.throws(() => transferResource(ledger, { revision: ledger.revision, from: { kind: 'store', id }, to: { kind: 'player' }, resourceId: 'r12', grams: 1 }, sources, stores, resources), /capacity/)
  assert.throws(() => validateResourceLedger({ ...ledger, stores: { [id]: { ...ledger.stores[id]!, items: { suit: 1 } } } } as any, sources, stores, resources), /metadata/)
})
test('v3 missing-container migration preserves mass/location/revision and bytes until successful commit', () => {
  const { session, storage, collect } = make(); collect(1333)
  const prior = session.snapshot
  const old = JSON.parse(storage.raw!); old.version = 3; delete old.itemDefinitions; old.stores = {}; storage.raw = JSON.stringify(old)
  const original = storage.raw, migrated = new ResourceSession(storage, session.world)
  assert.equal(migrated.error, ''); assert.equal(migrated.snapshot.version, 5)
  assert.deepEqual(migrated.snapshot.inventory, prior.inventory); assert.deepEqual(migrated.snapshot.sources, prior.sources)
  assert.deepEqual(migrated.snapshot.location, prior.location); assert.equal(migrated.snapshot.revision, prior.revision)
  assert.equal(storage.raw, original); storage.fail = true
  assert.equal(migrated.deposit('mineral', 333, prior.revision), false); assert.equal(storage.raw, original)
  storage.fail = false
  const retry = new ResourceSession(storage, session.world)
  assert.equal(retry.deposit('mineral', 333, prior.revision), true)
  assert.equal(JSON.parse(storage.raw!).version, 5)
  assert.equal(new ResourceSession(storage, session.world).snapshot.stores[id]!.quantities.mineral, 333)
})
test('corrupt current saves and failed/stale transfers preserve bytes and published state', () => {
  const { session, storage, collect } = make(); collect(1000); session.deposit('mineral', 333, session.snapshot.revision)
  const original = storage.raw!
  for (const mutate of [(s: any) => { delete s.stores[id] }, (s: any) => { s.stores[id].capacityGrams++ }, (s: any) => { s.stores[id].shipId = 'other' }, (s: any) => { s.stores[id].quantities.mineral++ }, (s: any) => { s.stores[id].items = {} }]) {
    const data = JSON.parse(original); mutate(data); storage.raw = JSON.stringify(data)
    const raw = storage.raw, restored = new ResourceSession(storage, session.world)
    assert.match(restored.error, /could not be loaded/)
    assert.equal(restored.withdraw('mineral', 1, restored.snapshot.revision), false)
    restored.checkpoint(); assert.equal(storage.raw, raw)
  }
  for (const failure of ['quota', 'stale']) {
    storage.raw = original; storage.fail = false
    const restored = new ResourceSession(storage, session.world), before = restored.snapshot
    if (failure === 'quota') storage.fail = true
    else storage.raw = original + ' '
    const raw = storage.raw
    assert.equal(restored.withdraw('mineral', 1, before.revision), false)
    assert.equal(restored.snapshot, before); assert.equal(storage.raw, raw); assert.match(restored.error, /Saving failed/)
  }
})
test('ship mass counts container and aboard player once without changing site entities', () => {
  const { session, collect } = make(); collect(1000)
  const entities = session.world.entities
  session.deposit('mineral', 333, session.snapshot.revision)
  assert.deepEqual(session.payload(), { playerGrams: 667, storeGrams: 333, shipGrams: 333 })
  let ship = initialShip()
  session.world = { ...session.world, entities: session.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout', point: point2(0, 4) } } : e) }
  for (let i = 0; i < 300; i++) ship = setMass(ship, DRY_MASS_KG + session.payload().shipGrams / 1000)
  assert.equal(ship.massKg, DRY_MASS_KG + 1)
  session.deposit('mineral', 667, session.snapshot.revision)
  assert.equal(session.payload().shipGrams, 1000)
  session.world = { ...session.world, entities }; assert.equal(session.payload().shipGrams, 1000)
  assert.equal(session.world.entities, entities)
})
