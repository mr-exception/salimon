import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createTopDownState, PLAYER_ID, stepTopDown, type TopDownState } from './topDownRuntime.ts'
import { point2, type Anchor } from './topDownFrames.ts'
import { decodeResourceSave, RESOURCE_CATALOG, RESOURCE_SAVE_KEY, RESOURCE_SOURCES, ResourceSession as LiveResourceSession, resourcePayload, withResourcePayload, type ResourceStorage } from './resourceSession.ts'
class ResourceSession extends LiveResourceSession {
  constructor(...args: ConstructorParameters<typeof LiveResourceSession>) { args[4] = () => 0; super(...args) }
}
import { resourceInventoryUsage } from './resourceInventory.ts'
import { initialShip } from './simulation.ts'
import { enterCockpit } from './cockpit.ts'

class MemoryStorage implements ResourceStorage {
  raw: string | null = null
  writes = 0
  fail = false
  getItem(key: string) { assert.equal(key, RESOURCE_SAVE_KEY); return this.raw }
  setItem(key: string, value: string) {
    assert.equal(key, RESOURCE_SAVE_KEY)
    if (this.fail) throw new Error('quota/access denied')
    this.raw = value; this.writes++
  }
}
const mineral = 'source:landing:mineral:01', biology = 'source:landing:biology:01'
function place(world: TopDownState, x: number, y: number, frame: Anchor['frame'] = 'planet:fixture'): TopDownState {
  return { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame, point: point2(x, y) } } : e) }
}
function make() {
  const store = new MemoryStorage(), session = new ResourceSession(store, place(createTopDownState('exterior', true), -7.1, 0))
  session.toggleTool()
  return { store, session }
}
function ticks(session: ResourceSession, count: number, direction: readonly [number, number] = [0, 0]) {
  for (let i = 0; i < count; i++) session.advance(stepTopDown(session.world, direction))
}
test('production world interaction commits exactly 1000 grams per 60 ticks', () => {
  const { store, session } = make()
  session.start(mineral); ticks(session, 59)
  assert.equal(session.snapshot.version, 7)
  assert.equal(session.snapshot.inventory.quantities.mineral, undefined)
  assert.equal(store.raw, null)
  ticks(session, 1)
  assert.equal(session.snapshot.inventory.quantities.mineral, 1000)
  assert.equal(session.snapshot.sources[mineral]!.remainingGrams, 59_000)
  assert.equal(store.writes, 1)
  const saved = decodeResourceSave(store.raw!, session.world)
  assert.deepEqual(saved.inventory, session.snapshot.inventory)
  assert.deepEqual(saved.sources, session.snapshot.sources)
  assert.deepEqual(saved.location.anchor, session.world.entities.find(e => e.id === PLAYER_ID)!.anchor)
  // Re-publishing the same world tick never adds time or material.
  for (let i = 0; i < 10; i++) session.advance(session.world)
  assert.equal(session.snapshot.inventory.quantities.mineral, 1000)
})
test('weight cap stops both mineral and biomass gathering and preserves remainder', () => {
  const { session } = make()
  session.start(mineral); ticks(session, 600)
  assert.equal(session.snapshot.inventory.quantities.mineral, 10_000)
  assert.equal(session.snapshot.sources[mineral]!.remainingGrams, 50_000)
  assert.match(session.feedback, /weight limit/)
  assert.equal(session.activeSourceId, null)
  const { session: light } = make()
  light.advance(place(light.world, -5, 4.5)); light.start(biology); ticks(light, 660)
  assert.equal(light.snapshot.inventory.quantities.biomass, 10_000)
  assert.equal(light.snapshot.sources[biology]!.remainingGrams, 5000)
  assert.match(light.feedback, /weight limit/)
  assert.deepEqual(resourceInventoryUsage(light.snapshot.inventory, RESOURCE_CATALOG), { grams: 10000, slots: 1 })
})
test('mixed gathered materials debit exact integer gram masses', () => {
  const { session } = make()
  session.start(mineral); ticks(session, 180); session.cancel()
  session.advance(place(session.world, -5, 4.5)); session.start(biology); ticks(session, 600)
  assert.deepEqual(session.snapshot.inventory.quantities, { mineral: 3000, biomass: 7000 })
  assert.deepEqual(resourceInventoryUsage(session.snapshot.inventory, RESOURCE_CATALOG), { grams: 10000, slots: 2 })
  assert.equal(session.snapshot.sources[biology]!.remainingGrams, 8000)
})
test('movement, tool stow, cancel, range loss and cockpit interruption discard partial progress', () => {
  for (const reason of ['movement', 'stow', 'escape', 'range', 'radar']) {
    const { session } = make()
    session.start(mineral); ticks(session, 59)
    if (reason === 'movement') ticks(session, 1, [1, 0])
    if (reason === 'stow') session.toggleTool()
    if (reason === 'escape') session.cancel()
    if (reason === 'range') session.advance(place(session.world, -6, 3))
    if (reason === 'radar') session.advance({ ...session.world, cockpit: { ...session.world.cockpit, view: 'radar' } })
    assert.equal(session.activeSourceId, null, reason)
    assert.equal(session.progressTicks, 0, reason)
    assert.deepEqual(session.snapshot.inventory.quantities, {}, reason)
  }
})
test('reload and repeated interactions cannot resume an unfinished second or reset depletion', () => {
  const { session, store } = make()
  session.start(mineral); ticks(session, 119)
  const restored = new ResourceSession(store, createTopDownState('exterior', true))
  assert.equal(restored.equipped, false)
  assert.equal(restored.activeSourceId, null)
  assert.equal(restored.progressTicks, 0)
  assert.equal(restored.snapshot.inventory.quantities.mineral, 1000)
  restored.toggleTool(); restored.start(mineral); ticks(restored, 59)
  assert.equal(restored.snapshot.inventory.quantities.mineral, 1000)
  ticks(restored, 1)
  for (let i = 0; i < 30; i++) restored.start(mineral)
  assert.equal(restored.snapshot.inventory.quantities.mineral, 2000)
  assert.equal(restored.snapshot.sources[mineral]!.remainingGrams, 58_000)
})
test('failed atomic replacement publishes neither inventory credit nor source debit', () => {
  const { session, store } = make()
  session.start(mineral); ticks(session, 60)
  const prior = store.raw
  store.fail = true; ticks(session, 60)
  assert.match(session.error, /Saving failed/)
  assert.equal(session.activeSourceId, null)
  assert.equal(session.snapshot.inventory.quantities.mineral, 1000)
  assert.equal(store.raw, prior)
  assert.equal(new ResourceSession(store, createTopDownState()).snapshot.sources[mineral]!.remainingGrams, 59_000)
})
test('stale writers fail closed rather than replace newer depletion or reset a save', () => {
  const { session, store } = make()
  const stale = new ResourceSession(store, session.world)
  session.start(mineral); ticks(session, 60)
  const prior = store.raw
  stale.toggleTool(); stale.start(mineral); ticks(stale, 60)
  assert.match(stale.error, /save changed/)
  assert.equal(store.raw, prior)
  assert.deepEqual(stale.snapshot.inventory.quantities, {})
})
test('valid save restores upgraded independent capacities and precise ship-local standing location', () => {
  const { session, store } = make()
  session.start(mineral); ticks(session, 60); session.cancel()
  session.advance(place(session.world, 0, -3.1, 'ship:scout'))
  assert.equal(session.upgrade(20_500, 16), true)
  const restored = new ResourceSession(store, createTopDownState('exterior', true))
  assert.equal(restored.error, '')
  assert.equal(restored.snapshot.inventory.capacityGrams, 20_500)
  assert.equal(restored.snapshot.inventory.capacitySlots, 16)
  assert.equal(restored.world.player.mode, 'standing')
  assert.deepEqual(restored.world.entities.find(e => e.id === PLAYER_ID)!.anchor, session.world.entities.find(e => e.id === PLAYER_ID)!.anchor)
})
test('cockpit save returns beside the chair and preserves payload without restoring radar or active work', () => {
  const { session, store } = make()
  session.start(mineral); ticks(session, 60); session.cancel()
  let world = place(session.world, 0, -3.1, 'ship:scout')
  world = enterCockpit({ ...world, selectedId: 'scout:chair' })
  assert.equal(world.player.mode, 'seated')
  session.advance(world); session.checkpoint()
  const restored = new ResourceSession(store, createTopDownState('exterior', true))
  assert.equal(restored.error, '')
  assert.equal(restored.world.cockpit.view, 'world')
  assert.deepEqual(restored.snapshot.location.anchor.point, point2(0, -3.1))
  assert.equal(restored.snapshot.inventory.quantities.mineral, 1000)
})
test('resource payload contributes once to ship mass only while aboard', () => {
  const { session } = make()
  session.start(mineral); ticks(session, 60)
  assert.deepEqual(resourcePayload(session.snapshot.inventory, session.world), { playerGrams: 1000, shipGrams: 0 })
  let ship = withResourcePayload(initialShip(), session.snapshot.inventory, session.world)
  assert.equal(ship.massKg, 100000)
  const aboard = place(session.world, 0, 2, 'ship:scout')
  for (let i = 0; i < 300; i++) ship = withResourcePayload(ship, session.snapshot.inventory, aboard)
  assert.equal(ship.massKg, 100001)
  assert.equal(withResourcePayload(ship, session.snapshot.inventory, session.world).massKg, 100000)
})
test('malformed, incompatible, duplicated, fractional and colliding saves preserve original bytes and block writes', () => {
  const { session, store } = make()
  session.start(mineral); ticks(session, 60)
  const original = store.raw!
  const mutations: ((data: any) => void)[] = [
    s => { s.version = 0 }, s => { s.catalogVersion = 'other' }, s => { s.siteId = 'other' },
    s => { s.inventory.quantities.mineral = 1.5 }, s => { s.inventory.quantities.unknown = 1 },
    s => { s.inventory.quantities.mineral = 2 }, s => { s.sources[mineral].remainingGrams = -1 },
    s => { delete s.sources[biology] }, s => { s.sources.extra = s.sources[mineral] },
    s => { s.inventory.capacityGrams = 9999 }, s => { s.inventory.capacitySlots = 11 },
    s => { s.inventory.quantities = null }, s => { s.inventory.items.fake = 1 }, s => { s.location.anchor.frame = 'missing' },
    s => { s.location.anchor.point.cell[0] = 'NaN' }, s => { s.location.anchor.point.offsetM[0] = null },
    s => { s.location.anchor.point.offsetM = [-9, 0, 0] }, s => { s.revision = Number.MAX_SAFE_INTEGER + 1 },
  ]
  for (const mutate of mutations) {
    const data = JSON.parse(original); mutate(data); store.raw = JSON.stringify(data)
    const invalidBytes = store.raw, restored = new ResourceSession(store, createTopDownState())
    assert.match(restored.error, /could not be loaded/)
    restored.toggleTool(); restored.start(mineral); restored.checkpoint()
    assert.equal(store.raw, invalidBytes)
  }
  store.raw = '{'
  assert.match(new ResourceSession(store, createTopDownState()).error, /could not be loaded/)
})
test('source exhaustion with future capacity upgrade retains IDs, colliders and exact source totals', () => {
  const { session } = make()
  session.upgrade(100_000, 20); session.start(mineral); ticks(session, 60 * 65)
  assert.equal(session.snapshot.sources[mineral]!.remainingGrams, 0)
  assert.equal(session.snapshot.inventory.quantities.mineral, RESOURCE_SOURCES[mineral]!.remainingGrams)
  assert.match(session.feedback, /depleted/)
  session.start(mineral); ticks(session, 120)
  assert.equal(session.snapshot.inventory.quantities.mineral, 60_000)
  assert.ok(session.world.entities.find(e => e.id === mineral)!.footprint)
})

test('legacy litre saves migrate exactly and never reset source depletion', () => {
  const { session, store } = make()
  session.start(mineral); ticks(session, 60); session.cancel()
  const current = JSON.parse(store.raw!)
  const legacy = {
    ...current, version: 1, catalogVersion: 'landing-resources:v1',
    inventory: { capacityGrams: 10_000, capacityLitres: 10, quantities: { mineral: 1 } },
    sources: {
      [mineral]: { id: mineral, resourceId: 'mineral', remainingLitres: 23 },
      [biology]: { id: biology, resourceId: 'biomass', remainingLitres: 30 },
    },
  }
  delete (legacy as any).stores
  delete (legacy as any).itemDefinitions
  delete (legacy as any).survival
  delete (legacy as any).hazards
  store.raw = JSON.stringify(legacy)
  const restored = new ResourceSession(store, createTopDownState())
  assert.equal(restored.error, '')
  assert.equal(restored.snapshot.version, 7)
  assert.deepEqual(restored.snapshot.inventory.quantities, { mineral: 2500 })
  assert.equal(restored.snapshot.sources[mineral]!.remainingGrams, 57_500)
  assert.equal(restored.snapshot.inventory.capacitySlots, 12)
  assert.equal(JSON.parse(store.raw!).version, 1)
  assert.equal(restored.upgrade(10_000, 12), true)
  assert.equal(JSON.parse(store.raw!).version, 7)
  const again = new ResourceSession(store, createTopDownState())
  assert.equal(again.error, '')
  assert.deepEqual(again.snapshot.inventory.quantities, { mineral: 2500 })
})

const stores = { cargo: { shipId: 'ship:scout', capacityGrams: 2333 }, remote: { shipId: 'ship:other', capacityGrams: 500 } }
const cargo = { kind: 'store', id: 'cargo' } as const, player = { kind: 'player' } as const
function ledgerSession() {
  const store = new MemoryStorage()
  const session = new ResourceSession(store, place(createTopDownState('exterior', true), -7.1, 0), stores)
  const transfer = (grams: number) => session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id: mineral }, to: cargo, resourceId: 'mineral', grams })
  return { store, session, transfer }
}
test('production session saves full ledger, store metadata and one revision through repeated restarts', () => {
  const { store, session, transfer } = ledgerSession()
  assert.equal(transfer(2333), true)
  assert.equal(store.writes, 1)
  assert.equal(session.snapshot.revision, 1)
  assert.equal(session.transfer({ revision: 1, from: cargo, to: player, resourceId: 'mineral', grams: 333 }), true)
  assert.equal(store.writes, 2)
  let prior = session.snapshot
  for (let i = 0; i < 10; i++) {
    const restored = new ResourceSession(store, createTopDownState('exterior', true), stores)
    assert.equal(restored.error, '')
    assert.deepEqual(restored.snapshot, prior)
    assert.equal(restored.transfer({ revision: restored.snapshot.revision, from: player, to: cargo, resourceId: 'mineral', grams: 1 }), true)
    assert.equal(restored.transfer({ revision: restored.snapshot.revision, from: cargo, to: player, resourceId: 'mineral', grams: 1 }), true)
    prior = restored.snapshot
  }
  assert.equal(prior.sources[mineral]!.remainingGrams, 57667)
  assert.equal(prior.stores.cargo!.quantities.mineral, 2000)
  assert.equal(prior.inventory.quantities.mineral, 333)
  assert.deepEqual(prior.stores.remote, { id: 'remote', ...stores.remote, quantities: {} })
})
test('capacity and stale-revision rejection leave storage, state and revision unchanged', () => {
  const { store, session, transfer } = ledgerSession()
  assert.equal(transfer(2334), false)
  assert.equal(store.writes, 0)
  assert.equal(session.snapshot.revision, 0)
  assert.equal(transfer(2333), true)
  const before = structuredClone(session.snapshot), raw = store.raw
  assert.equal(session.transfer({ revision: 0, from: cargo, to: player, resourceId: 'mineral', grams: 1 }), false)
  assert.deepEqual(session.snapshot, before)
  assert.equal(store.raw, raw)
  assert.equal(session.error, '')
})
test('failed and stale ledger writes preserve both ends, source and restart bytes', () => {
  for (const failure of ['quota', 'stale']) {
    const { store, session, transfer } = ledgerSession()
    assert.equal(transfer(1000), true)
    const before = structuredClone(session.snapshot)
    if (failure === 'quota') store.fail = true
    else { const other = new ResourceSession(store, session.world, stores); assert.equal(other.upgrade(11000, 12), true) }
    const raw = store.raw
    assert.equal(session.transfer({ revision: 1, from: cargo, to: player, resourceId: 'mineral', grams: 333 }), false)
    assert.deepEqual(session.snapshot, before)
    assert.equal(store.raw, raw)
    assert.match(session.error, /Saving failed/)
    assert.equal(transfer(1), false)
    const restart = new ResourceSession(store, createTopDownState('exterior', true), stores)
    assert.equal(restart.error, '')
    assert.equal(restart.snapshot.stores.cargo!.quantities.mineral, 1000)
    assert.deepEqual(restart.snapshot.inventory.quantities, {})
  }
})
test('v2 gram migration preserves all mass and old bytes until a successful v6 commit', () => {
  const { session, store } = make()
  session.start(mineral); ticks(session, 60)
  const v2 = JSON.parse(store.raw!); v2.version = 2; delete v2.stores; delete v2.itemDefinitions; delete v2.survival; delete v2.hazards
  store.raw = JSON.stringify(v2)
  const oldBytes = store.raw, migrated = new ResourceSession(store, session.world, stores)
  assert.equal(migrated.error, '')
  assert.equal(migrated.snapshot.version, 7)
  assert.equal(migrated.snapshot.revision, v2.revision)
  assert.equal(migrated.snapshot.inventory.quantities.mineral, 1000)
  assert.equal(migrated.snapshot.sources[mineral]!.remainingGrams, 59000)
  assert.equal(store.raw, oldBytes)
  store.fail = true
  assert.equal(migrated.transfer({ revision: v2.revision, from: player, to: cargo, grams: 333, resourceId: 'mineral' }), false)
  assert.equal(store.raw, oldBytes)
  store.fail = false
  const retry = new ResourceSession(store, session.world, stores)
  assert.equal(retry.transfer({ revision: v2.revision, from: player, to: cargo, grams: 333, resourceId: 'mineral' }), true)
  assert.equal(JSON.parse(store.raw!).version, 7)
  assert.equal(retry.snapshot.revision, v2.revision + 1)
})
test('corrupt and unsupported ledgers preserve bytes and block every write', () => {
  const { session, store, transfer } = ledgerSession()
  transfer(1000)
  const original = store.raw!
  for (const mutate of [
    (s: any) => { s.version = 8 }, (s: any) => { s.version = 2 },
    (s: any) => { s.stores.cargo.inventory = s.inventory }, (s: any) => { s.futureLedger = {} },
    (s: any) => { s.stores = null }, (s: any) => { s.stores.cargo.id = 'renamed' },
    (s: any) => { s.stores.cargo.shipId = 'other' }, (s: any) => { s.stores.cargo.capacityGrams++ },
    (s: any) => { s.stores.cargo.quantities.mineral++ }, (s: any) => { s.stores.cargo.quantities.mineral = 0.5 },
    (s: any) => { s.stores.unknown = s.stores.cargo }, (s: any) => { delete s.stores.cargo },
  ]) {
    const data = JSON.parse(original); mutate(data); store.raw = JSON.stringify(data)
    const bad = store.raw, restored = new ResourceSession(store, session.world, stores)
    assert.match(restored.error, /could not be loaded/)
    assert.equal(restored.transfer({ revision: 0, from: cargo, to: player, grams: 1, resourceId: 'mineral' }), false)
    restored.checkpoint()
    assert.equal(store.raw, bad)
  }
  store.raw = original
  assert.match(new ResourceSession(store, session.world).error, /could not be loaded/)
})
test('session mass counts flat stores and onboard player exactly once', () => {
  const { session, transfer } = ledgerSession()
  transfer(1000)
  session.transfer({ revision: 1, from: cargo, to: player, grams: 333, resourceId: 'mineral' })
  assert.deepEqual(session.payload(), { playerGrams: 333, storeGrams: 667, shipGrams: 667 })
  session.world = place(session.world, 0, 2, 'ship:scout')
  for (let i = 0; i < 300; i++) assert.deepEqual(session.payload(), { playerGrams: 333, storeGrams: 667, shipGrams: 1000 })
})
