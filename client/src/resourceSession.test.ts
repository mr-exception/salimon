import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createTopDownState, PLAYER_ID, stepTopDown, type TopDownState } from './topDownRuntime.ts'
import { point2, type Anchor } from './topDownFrames.ts'
import { decodeResourceSave, RESOURCE_CATALOG, RESOURCE_SAVE_KEY, RESOURCE_SOURCES, ResourceSession, resourcePayload, withResourcePayload, type ResourceStorage } from './resourceSession.ts'
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
  assert.equal(restored.snapshot.inventory.quantities.mineral, 1)
  restored.toggleTool(); restored.start(mineral); ticks(restored, 59)
  assert.equal(restored.snapshot.inventory.quantities.mineral, 1)
  ticks(restored, 1)
  for (let i = 0; i < 30; i++) restored.start(mineral)
  assert.equal(restored.snapshot.inventory.quantities.mineral, 2)
  assert.equal(restored.snapshot.sources[mineral]!.remainingLitres, 22)
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
  assert.equal(restored.snapshot.inventory.quantities.mineral, 1)
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
  assert.equal(session.snapshot.inventory.quantities.mineral, 20004)
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
  store.raw = JSON.stringify(legacy)
  const restored = new ResourceSession(store, createTopDownState())
  assert.equal(restored.error, '')
  assert.equal(restored.snapshot.version, 2)
  assert.deepEqual(restored.snapshot.inventory.quantities, { mineral: 2500 })
  assert.equal(restored.snapshot.sources[mineral]!.remainingGrams, 57_500)
  assert.equal(restored.snapshot.inventory.capacitySlots, 12)
  assert.equal(JSON.parse(store.raw!).version, 1)
  assert.equal(restored.upgrade(10_000, 12), true)
  assert.equal(JSON.parse(store.raw!).version, 2)
  const again = new ResourceSession(store, createTopDownState())
  assert.equal(again.error, '')
  assert.deepEqual(again.snapshot.inventory.quantities, { mineral: 2500 })
})
