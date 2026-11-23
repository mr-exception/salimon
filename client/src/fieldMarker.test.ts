import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession, decodeResourceSave } from './resourceSession.ts'
import { FABRICATOR_RECIPE_ID, MARKER_KIT_ID } from './scoutFabricator.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { createFieldMark } from './exploration.ts'
import { markerGuidance, validateDeployedMarkers } from './fieldMarker.ts'
import { resourceLedgerMass } from './resourceLedger.ts'
import { RESOURCE_CATALOG, ITEM_CATALOG } from './resourceSession.ts'

function setup() {
  const storage = { raw: null as string | null, fail: false, getItem: () => storage.raw, setItem: (_: string, v: string) => { if (storage.fail) throw Error('quota'); storage.raw = v } }
  const open = () => new ResourceSession(storage, createTopDownState('exterior', true), undefined, undefined, () => 0)
  const s = open()
  assert.equal(s.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 }), true)
  assert.equal(s.depositFabricatorInput('mineral', 1000, s.snapshot.revision), true)
  assert.equal(s.startFabricator(FABRICATOR_RECIPE_ID, s.snapshot.revision), true)
  s.advance({ ...s.world, tick: 1200 })
  assert.equal(s.claimFabricator(1, s.snapshot.revision), true)
  const deploy = (name = 'Home') => s.deployMarker(name, s.snapshot.revision, createFieldMark(s.world, '', '').id)
  return { s, storage, open, deploy }
}
function move(s: ResourceSession, x: number, y: number, frame: 'planet:fixture' | 'ship:scout' = 'planet:fixture') {
  s.world = { ...s.world, entities: s.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame, point: point2(x,y) } } : e) }
}

test('E07 paid manufacture → placement conserves kit, frees carrying capacity, survives transfers, cabin and reload; free notes and rename', () => {
  const { s, storage, open, deploy } = setup()
  const before = s.snapshot
  assert.equal(s.markField('Free notebook', ''), true)
  assert.equal(deploy(), true)
  assert.equal(s.snapshot.inventory.items[MARKER_KIT_ID], undefined)
  assert.equal(s.snapshot.deployedMarkers.length, 1)
  assert.equal(s.fieldMarks.length, 1)
  assert.match(markerGuidance(s.world, s.snapshot.deployedMarkers[0]!), /marker here/)
  assert.equal(resourceLedgerMass(s.snapshot, 'ship:scout', true, RESOURCE_CATALOG, ITEM_CATALOG).playerGrams, 0)
  assert.equal(s.snapshot.sources['source:landing:mineral:01']!.remainingGrams, before.sources['source:landing:mineral:01']!.remainingGrams)
  assert.equal(s.snapshot.core.spentJ60, before.core.spentJ60)
  assert.equal(s.renameMarker(s.snapshot.deployedMarkers[0]!.id, 'Return', s.snapshot.revision), true)
  assert.equal(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 }), true)
  move(s, 0, 4, 'ship:scout'); s.checkpoint()
  assert.equal(s.error, '')
  const restored = open(); assert.equal(restored.error, '')
  assert.equal(restored.snapshot.deployedMarkers[0]!.name, 'Return')
  assert.match(markerGuidance(restored.world, restored.snapshot.deployedMarkers[0]!), /unavailable/)
  assert.equal(restored.snapshot.inventory.items[MARKER_KIT_ID], undefined)
  assert.equal(JSON.parse(storage.getItem()!).version, 17)
})

test('E07 authority, changed position/revision, malformed names and duplicate clicks cannot consume or deploy remotely', () => {
  const { s, storage, deploy } = setup()
  const bytes = storage.raw, rev = s.snapshot.revision, id = createFieldMark(s.world, '', '').id
  assert.equal(s.deployMarker('Home', rev - 1, id), false)
  assert.equal(s.deployMarker('Home', rev, 'remote-id'), false)
  assert.equal(deploy('x'.repeat(49)), false)
  move(s, 0, 4, 'ship:scout'); assert.equal(s.deployMarker('Home', rev, id), false)
  move(s, -10, 2); s.survival = { ...s.survival, health: 0 }; assert.equal(s.deployMarker('Home', rev, id), false)
  assert.equal(storage.raw, bytes)
  const other = setup(); assert.equal(other.deploy(), true)
  const placed = other.storage.raw
  assert.equal(other.deploy(), false); assert.equal(other.storage.raw, placed)
  assert.equal(other.s.renameMarker(other.s.snapshot.deployedMarkers[0]!.id, 'Again', other.s.snapshot.revision - 1), false)
})

for (const stale of [false, true]) test(`E07 ${stale ? 'concurrent writer' : 'quota failure'} preserves kit and deployed receipts`, () => {
  const { s, storage, deploy } = setup()
  const before = s.snapshot
  if (stale) storage.raw += ' '; else storage.fail = true
  const bytes = storage.raw
  assert.equal(deploy(), false)
  assert.deepEqual(s.snapshot, before); assert.equal(storage.raw, bytes)
})

test('E07 fail-closed receipt/location/capacity validation and lossless v16 migration', () => {
  const { s, storage, deploy } = setup()
  const old = JSON.parse(storage.raw!); old.version = 16; delete old.deployedMarkers
  const migrated = decodeResourceSave(JSON.stringify(old), s.world)
  assert.deepEqual(migrated.deployedMarkers, []); assert.equal(migrated.inventory.items[MARKER_KIT_ID], 1)
  assert.equal(deploy(), true)
  const saved = JSON.parse(storage.raw!)
  for (const mutate of [
    (v: any) => { v.deployedMarkers = [] },
    (v: any) => { v.inventory.items[MARKER_KIT_ID] = 1 },
    (v: any) => { v.deployedMarkers.push(v.deployedMarkers[0]) },
    (v: any) => { v.deployedMarkers[0].location.bodyId = 'unknown' },
    (v: any) => { delete v.deployedMarkers },
    (v: any) => { v.deployedMarkers[0].landmarkId = 'remote' },
  ]) {
    const bad = structuredClone(saved); mutate(bad)
    assert.throws(() => decodeResourceSave(JSON.stringify(bad), s.world))
  }
  assert.throws(() => validateDeployedMarkers(Array(257).fill(saved.deployedMarkers[0]), s.snapshot.sites), /capacity/)
  const mark = s.snapshot.deployedMarkers[0]!
  move(s, -7, 4); assert.match(markerGuidance(s.world, mark), /local marker guidance/)
  move(s, 800, 800); assert.match(markerGuidance(s.world, mark), /unavailable/)
})
