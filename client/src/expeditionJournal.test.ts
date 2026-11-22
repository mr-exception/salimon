import { test } from 'node:test'
import { SCOUT_CORE_FUEL_ID } from './scoutEnergyCore.ts'
import assert from 'node:assert/strict'
import { ResourceSession, decodeResourceSave, RESOURCE_SAVE_KEY } from './resourceSession.ts'
import { emptyJournal, MAX_VISITS, recordVisit, validateJournal } from './expeditionJournal.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { surfaceScoutLocation } from './scoutLocation.ts'
import { resolveSurface, siteEntities } from './surfaceSite.ts'
import { surfaceRegion, localToSurface } from './surfaceCoordinates.ts'
import { planetDefinition } from './planetDefinition.ts'
import { point2, walkingFrames } from './topDownFrames.ts'

const memory = () => {
  let raw: string | null = null, fail = false, writes = 0
  return { getItem: () => raw, setItem: (_: string, value: string) => { if (fail) throw Error('quota'); raw = value; writes++ },
    fail: () => { fail = true }, writes: () => writes }
}
const open = (storage = memory()) => ({ storage, session: new ResourceSession(storage, createTopDownState(), undefined, undefined, () => 0) })

test('P07 durable notes, physical visits and unchanged save/reload are idempotent in the sole envelope', () => {
  const { storage, session } = open()
  // A mark between periodic checkpoints must include current survival progress.
  for (let i = 0; i < 20; i++) session.advance({ ...session.world, tick: session.world.tick + 1 })
  const currentSurvival = session.survival
  assert.ok(session.markField('Return', 'Beside the Scout'))
  assert.deepEqual(session.snapshot.survival, currentSurvival)
  const saved = storage.getItem(), revision = session.snapshot.revision
  assert.equal(session.snapshot.journal.visits.length, 1)
  for (let i = 0; i < 5; i++) {
    const restored = open(storage).session
    assert.equal(restored.error, '')
    assert.deepEqual(restored.fieldMarks, session.fieldMarks)
    assert.deepEqual(restored.snapshot.journal, session.snapshot.journal)
    restored.checkpoint()
    assert.equal(storage.getItem(), saved)
    assert.equal(restored.snapshot.revision, revision)
  }
  assert.equal(storage.writes(), 1)
})

test('P07 quota and concurrent marks never publish a new observation or overwrite valid bytes', () => {
  for (const concurrent of [false, true]) {
    const { storage, session } = open()
    assert.ok(session.markField('First', ''))
    const prior = session.snapshot
    if (concurrent) assert.ok(open(storage).session.markField('Other writer', ''))
    else storage.fail()
    const retained = storage.getItem()
    assert.equal(session.markField('Rejected', ''), false)
    assert.equal(session.snapshot, prior)
    assert.equal(session.fieldMarks[0]!.name, 'First')
    assert.equal(storage.getItem(), retained)
    assert.match(session.error, /Saving failed/)
    session.checkpoint(); assert.equal(storage.getItem(), retained)
  }
})

test('P07 v12 migration is lossless and lazy; corrupt journals/publications and partial site change preserve original bytes', () => {
  const { storage, session } = open()
  session.upgrade(12000, 13)
  const valid = JSON.parse(storage.getItem()!)
  const old = { ...structuredClone(valid), version: 12 }
  delete old.core; delete old.stores[SCOUT_CORE_FUEL_ID]; delete old.journal
  const legacy = JSON.stringify(old), restored = new ResourceSession({ getItem: () => legacy, setItem: () => assert.fail('migration wrote eagerly') }, createTopDownState(), undefined, undefined, () => 0)
  assert.equal(restored.error, '')
  assert.equal(restored.snapshot.version, 14)
  for (const key of ['sources','inventory','stores','survival','hazards','scout','location','sites'] as const) assert.deepEqual(JSON.parse(JSON.stringify(restored.snapshot, (_,v) => typeof v === 'bigint' ? String(v) : v))[key], valid[key])
  assert.deepEqual(restored.snapshot.journal, emptyJournal())
  session.markField('Here', 'Observed')
  const marked = JSON.parse(storage.getItem()!)
  for (const mutate of [
    (s: any) => { s.version = 99 },
    (s: any) => { s.journal.marks[0].definition.digest = 'unknown' },
    (s: any) => { s.journal.marks[0].location.bodyId = 'caligo' },
    (s: any) => { s.journal.marks[0].id = 'mark:forged' },
    (s: any) => { s.journal.marks.push(s.journal.marks[0]) },
    (s: any) => { s.journal.visits[0].regionId = 'wrong' },
    (s: any) => { s.journal.marks[0].note = 'x'.repeat(161) },
    (s: any) => { s.journal.visits = Array(MAX_VISITS + 1).fill(s.journal.visits[0]) },
    (s: any) => { s.siteId = 'new-site-with-old-scout' },
    (s: any) => { s.scout.surface.definition.digest = 'unknown' },
  ]) {
    const bad = structuredClone(marked); mutate(bad)
    const bytes = JSON.stringify(bad)
    const failed = new ResourceSession({ getItem: () => bytes, setItem: () => assert.fail('corruption overwritten') }, createTopDownState())
    assert.match(failed.error, /Existing data kept/)
    assert.equal(failed.markField('Cannot write', ''), false); failed.checkpoint()
  }
  assert.throws(() => decodeResourceSave('{partial', createTopDownState()))
})

test('P07 physical walk journal is bounded; render-only regions never become visits; revisits deduplicate', () => {
  const d = planetDefinition('vesper-moon'), address = { bodyId: d.bodyId, radialM: [d.radiusM + 5,0,0] as const, yaw: 0 }
  const location = surfaceScoutLocation(address), site = resolveSurface(address), base = createTopDownState()
  const landing = surfaceRegion(address, d)
  let journal = emptyJournal()
  let world = { ...base, scoutLocation: location, frames: site.frames, entities: [...base.entities.filter(e => e.layer !== 'terrain'), ...siteEntities(site)] }
  const session = new ResourceSession(memory(), world, undefined, undefined, () => 0)
  assert.deepEqual(session.snapshot.journal, emptyJournal(), 'loading a world does not claim a visit')
  for (let i = 0; i < 1200; i++) {
    const region = surfaceRegion(localToSurface(landing, [i*64,0]), d)
    const anchor = { frame: 'planet:fixture' as const, chart: region, point: point2(0,0) }
    world = { ...world, frames: walkingFrames(site.frames, landing, region), entities: world.entities.map(e => e.id === PLAYER_ID ? {...e,anchor} : e) }
    journal = recordVisit(journal, world)
    assert.ok(journal.visits.length <= MAX_VISITS)
  }
  assert.equal(journal.visits.length, MAX_VISITS)
  assert.ok(JSON.stringify(journal).length < 60000)
  assert.equal(recordVisit(journal, world), journal)
  assert.deepEqual(validateJournal(journal, session.snapshot.sites), journal)
  const cabin = {...world, entities:world.entities.map(e => e.id === PLAYER_ID ? {...e,anchor:{frame:'ship:scout' as const,point:point2(0,0)}} : e)}
  assert.equal(recordVisit(journal, cabin), journal)
  assert.equal(RESOURCE_SAVE_KEY, 'salimon:expedition:v1')
})
