import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceContainerInteraction, withdrawalCapacityError } from './resourceContainerInteraction.ts'
import { ResourceSession, type ResourceStorage } from './resourceSession.ts'
import { SCOUT_RESOURCE_CONTAINER_ID as id } from './scoutResourceContainer.ts'
import { createTopDownState, PLAYER_ID, stepTopDown } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
class Storage implements ResourceStorage {
  raw: string | null = null
  fail = false
  getItem() { return this.raw }
  setItem(_key: string, value: string) { if (this.fail) throw Error('quota'); this.raw = value }
}
function make(grams = 1000) {
  const storage = new Storage(), initial = createTopDownState('exterior', true)
  const world = { ...initial, selectedId: id, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(.5, 4.25) } } : e) }
  const session = new ResourceSession(storage, world)
  assert.equal(session.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams }), true)
  const ui = new ResourceContainerInteraction(session)
  assert.equal(ui.open(world), true)
  return { ui, session, storage, world }
}
test('review is non-mutating; confirmation commits exact grams once; editing or closing cancels only the draft', () => {
  const { ui, session, storage } = make()
  const before = session.snapshot, raw = storage.raw
  assert.equal(ui.review('deposit', 'mineral', '333'), true)
  assert.equal(session.snapshot, before); assert.equal(storage.raw, raw)
  assert.equal(ui.confirm(), true); assert.equal(ui.confirm(), false)
  assert.equal(session.snapshot.stores[id]!.quantities.mineral, 333)
  assert.equal(session.snapshot.inventory.quantities.mineral, 667)
  assert.match(ui.message, /333 g \(0.333 kg\)/)
  const committed = storage.raw
  ui.review('withdraw', 'mineral', '111'); ui.discard(); assert.equal(ui.confirm(), false)
  ui.review('withdraw', 'mineral', '111'); ui.close(); assert.equal(ui.confirm(), false)
  assert.equal(storage.raw, committed)
  assert.equal(new ResourceSession(storage, createTopDownState('exterior', true)).snapshot.stores[id]!.quantities.mineral, 333)
})
test('invalid/unknown/insufficient amounts produce readable errors and never alter either endpoint', () => {
  const { ui, session, storage } = make()
  const before = session.snapshot, raw = storage.raw
  for (const value of ['', '0', '-1', '0.333', '1e3', 'Infinity', '9007199254740992', 'e']) {
    assert.equal(ui.review('deposit', 'mineral', value), false); assert.match(ui.message, /positive whole/)
  }
  assert.equal(ui.review('deposit', 'suit', '1'), false); assert.match(ui.message, /Invalid raw resource/)
  assert.equal(ui.review('deposit', 'mineral', '1001'), false); assert.match(ui.message, /Insufficient amount/)
  assert.equal(ui.review('withdraw', 'mineral', '1'), false); assert.match(ui.message, /Insufficient amount/)
  assert.equal(session.snapshot, before); assert.equal(storage.raw, raw)
})
test('weight errors report which endpoint is full, preserve bytes, and allow an explicit smaller request', () => {
  const { ui, session, storage } = make(10000)
  assert.equal(session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'store', id }, resourceId: 'mineral', grams: 50000 }), true)
  const before = session.snapshot, raw = storage.raw
  assert.equal(ui.review('deposit', 'mineral', '1'), false); assert.match(ui.message, /Container overweight.*0 g free/)
  assert.equal(ui.review('withdraw', 'mineral', '1'), false); assert.match(ui.message, /Player overweight.*0 g free/)
  assert.equal(session.snapshot, before); assert.equal(storage.raw, raw)
})
test('stale confirmation revisions are rejected; quota and externally changed save bytes block safely', () => {
  for (const failure of ['revision', 'quota', 'external']) {
    const { ui, session, storage } = make()
    assert.equal(ui.review('deposit', 'mineral', '333'), true)
    if (failure === 'revision') assert.equal(session.deposit('mineral', 1, session.snapshot.revision), true)
    if (failure === 'quota') storage.fail = true
    if (failure === 'external') storage.raw += ' '
    const before = session.snapshot, raw = storage.raw
    assert.equal(ui.confirm(), false); assert.equal(ui.confirm(), false)
    assert.equal(session.snapshot, before); assert.equal(storage.raw, raw)
    assert.match(ui.message, failure === 'revision' ? /Stale transaction/ : /Saving failed/)
  }
})
test('out-of-range/wrong-target opens are rejected; movement/range/cockpit/scene loss closes without rolling back commits', () => {
  for (const reason of ['movement', 'range', 'radar', 'seated', 'target']) {
    const { ui, session, storage, world } = make()
    ui.review('deposit', 'mineral', '333'); ui.confirm()
    ui.review('withdraw', 'mineral', '111')
    const raw = storage.raw
    let next = session.world
    if (reason === 'movement') next = stepTopDown(next, [-1, 0])
    if (reason === 'range') next = { ...next, entities: next.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout', point: point2(0, -3) } } : e) }
    if (reason === 'radar') next = { ...next, cockpit: { ...next.cockpit, view: 'radar' } }
    if (reason === 'seated') next = { ...next, player: { ...next.player, mode: 'seated' } }
    if (reason === 'target') next = { ...next, player: { ...next.player, targetId: 'scout:core' } }
    ui.synchronize(next)
    if (reason === 'target') assert.equal(session.world.player.targetId, 'scout:core')
    assert.equal(ui.opened, false, reason); assert.equal(ui.pending, null); assert.equal(ui.confirm(), false)
    assert.equal(storage.raw, raw); assert.equal(session.snapshot.stores[id]!.quantities.mineral, 333)
    assert.equal(ui.open({ ...world, selectedId: 'scout:core' }), false)
    assert.equal(ui.open({ ...world, entities: world.entities.filter(e => e.id !== id) }), false)
  }
})

test('slot feedback distinguishes adding a raw type from an existing stack without rounding gram capacity', () => {
  // Real shared-slot usage from synthetic count-item definitions, no gameplay catalog grant.
  const inventory = { capacityGrams: 10000, capacitySlots: 12, quantities: { mineral: 333 }, items: { module: 11 } }
  const resources = { mineral: {}, biomass: {} }, items = { module: { gramsPerItem: 1, stackSize: 1 } }
  const used = resourceInventoryUsage(inventory, resources, items)
  assert.deepEqual(used, { grams: 344, slots: 12 })
  assert.match(withdrawalCapacityError(inventory, used, 'biomass', 1), /Player slots full: 12 \/ 12/)
  assert.equal(withdrawalCapacityError(inventory, used, 'mineral', 1), '')
  const heavy = { ...inventory, quantities: { mineral: 9999 }, items: {} }
  const heavyUsage = resourceInventoryUsage(heavy, resources, items)
  assert.equal(withdrawalCapacityError(heavy, heavyUsage, 'mineral', 1), '')
  assert.match(withdrawalCapacityError(heavy, heavyUsage, 'mineral', 2), /Player overweight: only 1 g free/)

})

test('opening saves arrival before review, so a location-only autosave cannot stale the confirmation', () => {
  const { session, storage } = make()
  const ui = new ResourceContainerInteraction(session)
  const arrived = { ...session.world, tick: 59, selectedId: id, entities: session.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(.6, 4.25) } } : e) }
  assert.equal(ui.open(arrived), true)
  assert.equal(ui.review('deposit', 'mineral', '333'), true)
  const revision = session.snapshot.revision
  session.advance(stepTopDown(session.world, [0, 0])); ui.synchronize(session.world)
  assert.equal(session.snapshot.revision, revision)
  assert.equal(ui.confirm(), true)
  assert.equal(session.snapshot.stores[id]!.quantities.mineral, 333)
  assert.equal(new ResourceSession(storage, createTopDownState('exterior', true)).snapshot.stores[id]!.quantities.mineral, 333)
})
