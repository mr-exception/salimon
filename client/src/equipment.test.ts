import { test } from 'node:test'
import assert from 'node:assert/strict'
import { freshEquipment, validateEquipment, equipmentMass, functioningEquipment, type EquipmentState } from './equipment.ts'
import { resourceInventoryUsage, startingResourceInventory } from './resourceInventory.ts'
import { ResourceSession, RESOURCE_CATALOG, decodeResourceSave } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
const world = () => createTopDownState('exterior', true)
class Storage {
 raw: string | null = null; fail = false
 getItem() { return this.raw }
 setItem(_k: string, v: string) { if (this.fail) throw Error('disk full'); this.raw = v }
}
const session = (storage = new Storage()) => new ResourceSession(storage, world(), undefined, undefined, () => 0)
test('starter exact mass, capacity and aboard payload once', () => {
 const s = session(), gear = s.snapshot.inventory.equipment!
 assert.deepEqual(equipmentMass(gear), { carriedGrams: 0, carriedSlots: 0, equippedGrams: 3500, storedGrams: 0 })
 assert.deepEqual(resourceInventoryUsage(s.snapshot.inventory, RESOURCE_CATALOG), { grams: 0, slots: 0 })
 assert.equal(s.payload().shipGrams, 0)
 s.world = { ...s.world, entities: s.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout', point: point2(0, 3) } } : e) }
 for (let i = 0; i < 100; i++) assert.equal(s.payload().shipGrams, 3500)
 assert.equal(functioningEquipment(gear).length, 1)
})
test('carried suit includes installed mass and one slot; destroyed suit keeps intact modules', () => {
 const gear = freshEquipment(), suit = gear.instances[0]!
 const carried: EquipmentState = { ...gear, instances: [{ ...suit, location: { kind: 'inventory' } }, gear.instances[1]!] }
 assert.deepEqual(resourceInventoryUsage({ ...startingResourceInventory(), equipment: carried }, {}), { grams: 3500, slots: 1 })
 assert.throws(() => resourceInventoryUsage({ ...startingResourceInventory(), capacityGrams: 3499, equipment: carried }, {}))
 const destroyed = { ...gear, instances: [{ ...suit, remaining: '0/1' }, gear.instances[1]!] }
 assert.deepEqual(functioningEquipment(destroyed), [])
 assert.equal(validateEquipment(destroyed).instances[1]!.remaining, '1000/1')
})
test('reject malformed identity/definition/durability/owner aliases and preserve tombstones', () => {
 const starter = freshEquipment()
 const bad = [
 { ...starter, instances: [...starter.instances, starter.instances[0]!] },
 ...['2001/2', '-1/1', 'NaN', '2/2'].map(remaining => ({ ...starter, instances: starter.instances.map(g => ({ ...g, remaining })) })),
 { ...starter, instances: starter.instances.map(g => ({ ...g, definitionId: 'unknown' })) },
 { ...starter, instances: [starter.instances[1]!] },
 { ...starter, instances: [...starter.instances, { ...starter.instances[1]!, id: 'gear:0:duplicate-slot' }] },
 { ...starter, instances: [starter.instances[0]!, { ...starter.instances[1]!, remaining: '0/1' }] },
 { ...starter, instances: [{ ...starter.instances[0]!, location: { kind: 'equipped', alias: 'inventory' } }, starter.instances[1]!] },
 ]
 for (const g of bad) assert.throws(() => validateEquipment(g))
 const tombstone = { ...starter, instances: [starter.instances[0]!, { ...starter.instances[1]!, remaining: '0/1', location: { kind: 'destroyed' as const } }] }
 assert.equal(equipmentMass(tombstone).equippedGrams, 3000)
})
test('migration commits once preserving progress; reload never refreshes exact gear HP', () => {
 const storage = new Storage(), s = session(storage); s.upgrade(12000, 14)
 const legacy = JSON.parse(storage.raw!); legacy.version = 17; delete legacy.bench; if (legacy.stores) delete legacy.stores['scout:gear-bench:input']; delete legacy.inventory.equipment; storage.raw = JSON.stringify(legacy)
 const migrated = session(storage)
 assert.equal(migrated.error, ''); assert.equal(JSON.parse(storage.raw!).version, 17)
 assert.equal(migrated.upgrade(12000, 14), true); assert.equal(JSON.parse(storage.raw!).version, 19)
 assert.equal(migrated.snapshot.inventory.capacityGrams, 12000); assert.deepEqual(migrated.snapshot.sources, s.snapshot.sources)
 const raw = JSON.parse(storage.raw!); raw.inventory.equipment.instances[1].remaining = '1999/2'; storage.raw = JSON.stringify(raw)
 const reload = session(storage); assert.equal(reload.error, '')
 assert.equal(reload.snapshot.inventory.equipment!.instances[1]!.remaining, '1999/2')
 assert.deepEqual(session(storage).snapshot.inventory.equipment, reload.snapshot.inventory.equipment)
 delete raw.inventory.equipment; assert.throws(() => decodeResourceSave(JSON.stringify(raw), world()))
})
test('failed migration/reset and stale commits preserve state; terminal reset advances generation once', () => {
 const storage = new Storage(), s = session(storage); s.upgrade(12000, 14)
 const raw = JSON.parse(storage.raw!); raw.version = 17; delete raw.bench; if (raw.stores) delete raw.stores['scout:gear-bench:input']; delete raw.inventory.equipment; storage.raw = JSON.stringify(raw); storage.fail = true
 const migration = session(storage); assert.equal(migration.upgrade(12000, 14), false); assert.match(migration.error, /Saving failed/); assert.equal(JSON.parse(storage.raw!).version, 17)
 storage.fail = false; const living = session(storage); assert.equal(living.freshRun(), false); living.upgrade(12000, 14)
 const dead = JSON.parse(storage.raw!); dead.survival.health = 0; delete dead.survival.healthExact; storage.raw = JSON.stringify(dead)
 const terminal = session(storage), before = structuredClone(terminal.snapshot); storage.fail = true
 assert.equal(terminal.freshRun(), false); assert.deepEqual(terminal.snapshot, before)
 storage.fail = false; const reset = session(storage)
 assert.equal(reset.freshRun(), true); assert.equal(reset.snapshot.inventory.equipment!.generation, 1)
 assert.equal(reset.freshRun(), false); assert.deepEqual(reset.snapshot.sources, before.sources)
 const old = session(storage); storage.raw = storage.raw! + ' '
 assert.equal(old.upgrade(14000, 16), false); assert.equal(old.snapshot.inventory.equipment!.generation, 1)
})
