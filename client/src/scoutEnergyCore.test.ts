import { freshEquipment } from './equipment.ts'
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createTopDownState } from './topDownRuntime.ts'
import { TEST_ITEMS } from '../tests/itemFixtures.ts'
import { ResourceSession, RESOURCE_SAVE_KEY, type ResourceStorage } from './resourceSession.ts'
import { SCOUT_CORE_FUEL_ID, CORE_INITIAL_RESERVE_J, freshCoreEnergy } from './scoutEnergyCore.ts'

class Storage implements ResourceStorage {
  raw: string | null = null
  fail = false
  getItem(key: string) { assert.equal(key, RESOURCE_SAVE_KEY); return this.raw }
  setItem(key: string, value: string) {
    assert.equal(key, RESOURCE_SAVE_KEY)
    if (this.fail) throw new Error('storage denied')
    this.raw = value
  }
}
const source = 'source:landing:mineral:01'
const session = (store: Storage) => new ResourceSession(store, createTopDownState('exterior', true), undefined, undefined, () => 0)
const collect = (s: ResourceSession, grams: number) =>
  s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: source }, to: { kind: 'player' }, grams, resourceId: 'mineral' })

test('Core is an independent 50 kg unconverted buffer with initial and durable energy', () => {
  const storage = new Storage(), s = session(storage)
  assert.deepEqual(s.snapshot.core, freshCoreEnergy())
  assert.deepEqual(s.snapshot.stores[SCOUT_CORE_FUEL_ID], {
    id: SCOUT_CORE_FUEL_ID, shipId: 'ship:scout', capacityGrams: 50000, quantities: {},
  })
  for (let i = 0; i < 5; i++) {
    assert.equal(collect(s, 10000), true)
    assert.equal(s.depositCoreFuel('mineral', 10000, s.snapshot.revision), true)
  }
  assert.equal(s.snapshot.stores[SCOUT_CORE_FUEL_ID]!.quantities.mineral, 50000)
  assert.equal(s.snapshot.stores['scout:storage']!.quantities.mineral, undefined)
  assert.equal(s.payload().shipGrams, 50000)
  assert.equal(s.snapshot.sources[source]!.remainingGrams, 10000)
  assert.equal(collect(s, 1), true)
  const before = structuredClone(s.snapshot), bytes = storage.raw
  assert.equal(s.depositCoreFuel('mineral', 1, s.snapshot.revision), false)
  assert.match(s.feedback, /Store exceeds capacity/)
  assert.deepEqual(s.snapshot, before)
  assert.equal(storage.raw, bytes)
  assert.equal(s.deposit('mineral', 1, s.snapshot.revision), true)
  const filled = s.snapshot, filledBytes = storage.raw
  assert.equal(s.withdrawCoreFuel('mineral', 50001, s.snapshot.revision), false)
  assert.equal(s.withdrawCoreFuel('mineral', 1, s.snapshot.revision - 1), false)
  assert.equal(s.snapshot, filled)
  assert.equal(storage.raw, filledBytes)
  assert.equal(s.withdrawCoreFuel('mineral', 10000, s.snapshot.revision), true)
  assert.equal(s.payload().shipGrams, 40001) // player is outside; carried mass is not onboard
  assert.equal(s.snapshot.inventory.quantities.mineral, 10000)
  const restored = session(storage)
  assert.equal(restored.error, '')
  assert.deepEqual(restored.snapshot, s.snapshot)
  assert.equal(restored.snapshot.core.reserveJ60, CORE_INITIAL_RESERVE_J * 60)
})

test('Core rejects fractional grams, full player cargo, storage failures and another writer', () => {
  const storage = new Storage(), s = session(storage)
  assert.equal(collect(s, 10000), true)
  for (const grams of [0, -1, 0.5, Infinity, NaN, Number.MAX_SAFE_INTEGER + 1]) {
    const bytes = storage.raw, current = s.snapshot
    assert.equal(s.depositCoreFuel('mineral', grams, s.snapshot.revision), false)
    assert.equal(s.snapshot, current)
    assert.equal(storage.raw, bytes)
  }
  assert.equal(s.depositCoreFuel('mineral', 10000, s.snapshot.revision), true)
  assert.equal(collect(s, 10000), true)
  const current = s.snapshot, bytes = storage.raw
  assert.equal(s.withdrawCoreFuel('mineral', 1, s.snapshot.revision), false)
  assert.equal(storage.raw, bytes)
  assert.equal(s.snapshot, current)
  storage.fail = true
  assert.equal(s.depositCoreFuel('mineral', 1, s.snapshot.revision), false)
  assert.deepEqual(s.snapshot, current)
  assert.equal(storage.raw, bytes)
  storage.fail = false
  const a = session(storage), b = session(storage)
  assert.equal(a.depositCoreFuel('mineral', 10000, a.snapshot.revision), true)
  const updated = storage.raw
  assert.equal(b.depositCoreFuel('mineral', 1, b.snapshot.revision), false)
  assert.equal(storage.raw, updated)
  assert.match(b.error, /Saving failed/)
})

test('Core transfer keeps onboard mass exact and honors shared inventory slots', () => {
  const storage = new Storage()
  const s = new ResourceSession(storage, createTopDownState('interior', true), undefined, TEST_ITEMS, () => 0)
  assert.equal(collect(s, 1000), true)
  assert.deepEqual(s.payload(), { playerGrams: 1000, storeGrams: 0, shipGrams: 4500 })
  assert.equal(s.depositCoreFuel('mineral', 1000, s.snapshot.revision), true)
  assert.deepEqual(s.payload(), { playerGrams: 0, storeGrams: 1000, shipGrams: 4500 })
  assert.equal(s.changeItem('add', 'fixture-single', 12, s.snapshot.revision), true)
  const before = s.snapshot, bytes = storage.raw
  assert.equal(s.withdrawCoreFuel('mineral', 1, s.snapshot.revision), false)
  assert.match(s.feedback, /Inventory exceeds its capacity/)
  assert.equal(s.snapshot, before)
  assert.equal(storage.raw, bytes)
  assert.equal(s.changeItem('remove', 'fixture-single', 1, s.snapshot.revision), true)
  const onboard = s.payload().shipGrams
  assert.equal(s.withdrawCoreFuel('mineral', 1000, s.snapshot.revision), true)
  assert.equal(s.payload().shipGrams, onboard)
})

test('v13 migrates empty Core without minting fuel or rewriting bytes until commit', () => {
  const storage = new Storage(), s = session(storage)
  assert.equal(collect(s, 1000), true)
  const v13 = JSON.parse(storage.raw!)
  v13.version = 13; delete v13.bench; if (v13.stores) delete v13.stores['scout:gear-bench:input']; delete v13.inventory.equipment
  delete v13.fabricator; delete v13.deployedMarkers; if (v13.stores) delete v13.stores['scout:fabricator:input']; if (v13.itemDefinitions) delete v13.itemDefinitions['field-marker-kit:v1']; delete v13.convertedGrams; delete v13.core
  delete v13.stores[SCOUT_CORE_FUEL_ID]
  storage.raw = JSON.stringify(v13)
  const prior = storage.raw, restored = session(storage)
  assert.equal(restored.error, '')
  assert.equal(restored.snapshot.version, 19)
  assert.equal(restored.snapshot.revision, v13.revision)
  assert.deepEqual(restored.snapshot.sources, v13.sources)
  assert.deepEqual(restored.snapshot.inventory, { ...v13.inventory, equipment: freshEquipment() })
  assert.deepEqual(restored.snapshot.stores[SCOUT_CORE_FUEL_ID]!.quantities, {})
  assert.equal(restored.snapshot.core.reserveJ60, CORE_INITIAL_RESERVE_J * 60)
  assert.equal(storage.raw, prior)
  assert.equal(restored.depositCoreFuel('mineral', 333, restored.snapshot.revision), true)
  assert.equal(JSON.parse(storage.raw!).version, 19)
  assert.equal(session(storage).snapshot.stores[SCOUT_CORE_FUEL_ID]!.quantities.mineral, 333)
})

test('corrupt or future Core saves fail closed with original bytes preserved', () => {
  const storage = new Storage(), s = session(storage)
  assert.equal(collect(s, 1000), true)
  const original = JSON.parse(storage.raw!)
  const mutations: ((save: any) => void)[] = [
    save => { save.version = 20 },
    save => { save.version = 13; delete save.bench; if (save.stores) delete save.stores['scout:gear-bench:input']; delete save.inventory.equipment; delete save.fabricator; delete save.deployedMarkers; if (save.stores) delete save.stores['scout:fabricator:input']; if (save.itemDefinitions) delete save.itemDefinitions['field-marker-kit:v1']; delete save.convertedGrams; delete save.core; save.stores[SCOUT_CORE_FUEL_ID].capacityGrams = 1 },
    save => { delete save.fabricator; delete save.deployedMarkers; if (save.stores) delete save.stores['scout:fabricator:input']; if (save.itemDefinitions) delete save.itemDefinitions['field-marker-kit:v1']; delete save.convertedGrams; delete save.core },
    save => { save.core.capacityJ++ },
    save => { save.core.reserveJ60 = -1 },
    save => { save.core.reserveJ60 = 120_000_001 },
    save => { save.core.reserveJ60 = 0.5 },
    save => { save.core.conversionBudgetJ60 = 1.5 },
    save => { save.core.other = 1 },
    save => { delete save.stores[SCOUT_CORE_FUEL_ID] },
    save => { save.stores[SCOUT_CORE_FUEL_ID].quantities.mineral = 0.1 },
    save => { save.stores[SCOUT_CORE_FUEL_ID].capacityGrams = 50001 },
  ]
  for (const mutate of mutations) {
    const data = structuredClone(original)
    mutate(data)
    storage.raw = JSON.stringify(data)
    const bytes = storage.raw, bad = session(storage)
    assert.match(bad.error, /could not be loaded/)
    assert.equal(bad.depositCoreFuel('mineral', 1, bad.snapshot.revision), false)
    bad.checkpoint()
    assert.equal(storage.raw, bytes)
  }
})
