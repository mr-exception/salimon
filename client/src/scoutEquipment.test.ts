import { test } from 'node:test'
import assert from 'node:assert/strict'
import { freshEquipment, equipmentMass, type GearInstance, type GearLocation } from './equipment.ts'
import { applyGearCommand, SUIT_DRAWER_CAPACITIES, MODULE_STORAGE_CAPACITIES, type GearCommand } from './scoutEquipment.ts'
import { ResourceSession } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
const suit = 'gear:0:starter-suit', module = 'gear:0:starter-temperature'
const move = (id: string, from: GearLocation, to: GearLocation): GearCommand => ({ kind: 'move', id, from, to })
class Storage { raw: string | null = null; fail = false; getItem() { return this.raw }; setItem(_k: string, v: string) { if (this.fail) throw Error('quota'); this.raw = v } }
function place(s: ResourceSession, x = .5, y = 4.25, frame: 'ship:scout' | 'planet:fixture' = 'ship:scout') {
 s.world = { ...s.world, entities: s.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame, point: point2(x, y) } } : e) }
}
function setup(extra: GearInstance[] = [], mutate?: (v: any) => void) {
 const storage = new Storage(), initial = new ResourceSession(storage, createTopDownState('interior', true), undefined, undefined, () => 0)
 place(initial); initial.upgrade(10000, 12)
 const raw = JSON.parse(storage.raw!); raw.inventory.equipment.instances.push(...extra); mutate?.(raw); storage.raw = JSON.stringify(raw)
 const s = new ResourceSession(storage, createTopDownState(), undefined, undefined, () => 0)
 assert.equal(s.error, '')
 return { s, storage, command: (c: GearCommand) => s.changeEquipment(c, s.snapshot.revision) }
}
const loose = (id: string, kind: GearLocation['kind'] = 'module-storage'): GearInstance => ({ id: `gear:0:${id}`, definitionId: 'module:radiation:t3', remaining: '1999/2', location: { kind } as GearLocation })
const storedSuit = (i: number): GearInstance => ({ id: `gear:0:suit${i}`, definitionId: 'suit:basic', remaining: '0/1', location: { kind: 'drawer' } })
test('all ownership endpoints, two slots, exact damaged HP, same-type modules, no-suit and reload preserve mass', () => {
 const { s, storage, command } = setup([loose('extra')]); const mass = s.payload().shipGrams
 assert.equal(command(move(suit, { kind: 'equipped' }, { kind: 'drawer' })), true)
 assert.equal(s.snapshot.inventory.equipment!.instances.some(g => g.location.kind === 'equipped'), false)
 assert.equal(command(move(module, { kind: 'installed', suitId: suit, slot: 0 }, { kind: 'module-storage' })), true)
 assert.equal(command(move('gear:0:extra', { kind: 'module-storage' }, { kind: 'inventory' })), true)
 assert.equal(s.payload().playerGrams, 1500)
 assert.equal(command(move('gear:0:extra', { kind: 'inventory' }, { kind: 'installed', suitId: suit, slot: 0 })), true)
 assert.equal(command(move(module, { kind: 'module-storage' }, { kind: 'installed', suitId: suit, slot: 1 })), true)
 assert.equal(command(move(suit, { kind: 'drawer' }, { kind: 'inventory' })), true)
 assert.equal(s.payload().playerGrams, 5000)
 assert.equal(command(move(suit, { kind: 'inventory' }, { kind: 'equipped' })), true)
 assert.equal(s.payload().shipGrams, mass); assert.equal(s.payload().playerGrams, 0)
 const reloaded = new ResourceSession(storage, createTopDownState(), undefined, undefined, () => 0)
 assert.equal(reloaded.error, ''); assert.deepEqual(reloaded.snapshot.inventory.equipment, s.snapshot.inventory.equipment)
 assert.equal(reloaded.snapshot.inventory.equipment!.instances.find(g => g.id === 'gear:0:extra')!.remaining, '1999/2')
 const gear = freshEquipment(); const sameType = { ...gear, instances: [...gear.instances, { ...gear.instances[1]!, id: 'gear:0:second-temp', location: { kind: 'inventory' as const } }] }
 assert.equal(applyGearCommand(sameType, move('gear:0:second-temp', { kind: 'inventory' }, { kind: 'installed', suitId: suit, slot: 1 })).instances.length, 3)
})
test('explicit switch replaces destroyed suit and retains modules; no implicit swap or discard', () => {
 const { s, command } = setup([storedSuit(0)], raw => { raw.inventory.equipment.instances[0].remaining = '0/1' })
 const before = s.snapshot
 assert.equal(command(move('gear:0:suit0', { kind: 'drawer' }, { kind: 'equipped' })), false); assert.equal(s.snapshot, before)
 assert.equal(command({ kind: 'switch', id: 'gear:0:suit0', from: { kind: 'drawer' }, equippedId: suit, unequipTo: { kind: 'drawer' } }), true)
 assert.equal(s.snapshot.inventory.equipment!.instances.find(g => g.id === module)!.location.kind, 'installed')
 assert.equal(s.snapshot.inventory.equipment!.instances.find(g => g.id === suit)!.remaining, '0/1')
})
test('full storage and personal mass/slot destinations reject both endpoints atomically', () => {
 assert.deepEqual(SUIT_DRAWER_CAPACITIES, [4,8,12]); assert.deepEqual(MODULE_STORAGE_CAPACITIES, [12,16,24])
 for (const scenario of ['drawer', 'module-storage', 'mass', 'slots']) {
  const extra = scenario === 'drawer' ? [0,1,2,3].map(storedSuit) : scenario === 'module-storage' ? Array.from({ length: 12 }, (_, i) => loose('m'+i)) : []
  const { s, storage, command } = setup(extra)
  if (scenario === 'mass') assert.equal(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 10000 }), true)
  if (scenario === 'slots') {
   s.snapshot = { ...s.snapshot, inventory: { ...s.snapshot.inventory, capacitySlots: 12, equipment: { ...s.snapshot.inventory.equipment!, instances: [...s.snapshot.inventory.equipment!.instances, ...Array.from({ length: 12 }, (_, i) => loose('carried'+i, 'inventory')).map(g => ({ ...g, definitionId: 'module:temperature:t1' }))] } } }
  }
  const before = s.snapshot, bytes = storage.raw
  const c = scenario === 'module-storage' ? move(module, { kind: 'installed', suitId: suit, slot: 0 }, { kind: 'module-storage' }) : move(suit, { kind: 'equipped' }, { kind: scenario === 'drawer' ? 'drawer' : 'inventory' })
  assert.equal(command(c), false, scenario); assert.equal(s.snapshot, before); assert.equal(storage.raw, bytes)
 }
})
test('commit rechecks frame, reach, revision, seating and destroyed modules; no mutation on failures', () => {
 const { s, storage, command } = setup()
 const c = move(suit, { kind: 'equipped' }, { kind: 'drawer' })
 const before = s.snapshot, bytes = storage.raw
 for (const [x,y,frame] of [[0,2,'ship:scout'],[.5,4.25,'planet:fixture']] as const) {
  place(s,x,y,frame); assert.equal(command(c),false); assert.equal(s.snapshot,before); assert.equal(storage.raw,bytes)
 }
 place(s); assert.equal(s.changeEquipment(c,s.snapshot.revision-1),false)
 s.world = { ...s.world, player: { ...s.world.player, mode: 'seated' } }; assert.equal(command(c),false)
 const gear = freshEquipment(), tombstone = { ...gear, instances: [gear.instances[0]!, { ...gear.instances[1]!, remaining:'0/1',location:{kind:'destroyed' as const} }] }
 assert.throws(() => applyGearCommand(tombstone, move(module,{kind:'destroyed'},{kind:'inventory'})))
 assert.throws(() => applyGearCommand(gear, move(module,{kind:'installed',suitId:suit,slot:0},{kind:'installed',suitId:suit,slot:0})))
})
test('failed and stale save rolls back equipped/storage ownership and all payload endpoints', () => {
 for (const failure of ['quota','stale']) {
  const { s, storage, command } = setup(), before = structuredClone(s.snapshot), mass = s.payload()
  if (failure === 'quota') storage.fail = true; else storage.raw += ' '
  const bytes = storage.raw
  assert.equal(command(move(suit,{kind:'equipped'},{kind:'drawer'})),false)
  assert.deepEqual(s.snapshot,before); assert.deepEqual(s.payload(),mass); assert.equal(storage.raw,bytes); assert.ok(s.error)
 }
 assert.equal(equipmentMass(freshEquipment()).equippedGrams,3500)
})
