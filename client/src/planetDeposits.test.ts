import { dot, normal, scale } from './math.ts'
import { allowedMove, objectAngle } from './topDownInteraction.ts'
import { ORIGIN } from './coordinates.ts'
import { test } from 'node:test'
import { execFileSync } from 'node:child_process'
import assert from 'node:assert/strict'
import { depositFromId, depositNeighborhood, regionDeposits, DEPOSIT_PUBLICATION } from './planetDeposits.ts'
import { geologicalDefinition, planetDefinition } from './planetDefinition.ts'
import { localToSurface, surfaceRegion } from './surfaceCoordinates.ts'
import { probeGeology } from './planetGeology.ts'
import { samplePlanetSurface } from './planetSurface.ts'
import { ResourceSession, RESOURCE_SOURCES } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID, stepTopDown } from './topDownRuntime.ts'
import { point2, walkingFrames, type WorldFrames } from './topDownFrames.ts'
import { resolveSurface, siteEntities } from './surfaceSite.ts'
import { surfaceScoutLocation } from './scoutLocation.ts'

const definition = geologicalDefinition('vesper-moon')
const origin = surfaceRegion({ bodyId: definition.bodyId, radialM: [definition.radiusM + 5, 0, 0] }, definition)
const regionAt = (i: number) => surfaceRegion(localToSurface(origin, [i * 64, 0]), definition)
const ticks = (session: ResourceSession, n: number) => { for (let i = 0; i < n; i++) session.advance(stepTopDown(session.world, [0, 0])) }
function fixture(index = 1, items = {}) {
  const region = regionAt(index), deposit = regionDeposits(region)[0]!, [x, y] = deposit.entity.anchor.point.offsetM
  const address = { ...localToSurface(origin, [22528, 1000]), yaw: 0, definition: { bodyId: definition.bodyId, version: definition.version, digest: definition.digest } }
  const site = resolveSurface(address), location = surfaceScoutLocation(address), base = createTopDownState()
  const anchor = { frame: 'planet:fixture' as const, chart: region, point: point2(x + 1.5, y) }
  const world = { ...base, scoutLocation: location, frames: walkingFrames(site.frames, surfaceRegion(address, definition), region),
    camera: { ...base.camera, anchor }, entities: [...base.entities.filter(e => e.layer !== 'terrain').map(e => e.id === PLAYER_ID ? { ...e, anchor } : e), ...siteEntities(site)] }
  let raw: string | null = null, fail = false
  const storage = { getItem: () => raw, setItem: (_: string, value: string) => { if (fail) throw Error('quota'); raw = value } }
  const session = new ResourceSession(storage, world, undefined, items, () => 0)
  assert.equal(session.error, '')
  return { session, storage, deposit, world, fail: () => { fail = true }, raw: () => raw }
}

test('P05 fixed published barren, common, dense and rare candidates; regenerated IDs and safety', () => {
  assert.equal(DEPOSIT_PUBLICATION, 'd703783c21cc35a6')
  assert.equal(regionDeposits(regionAt(15)).length, 0)
  assert.equal(regionDeposits(regionAt(0)).length, 1)
  assert.equal(regionDeposits(regionAt(1)).length, 3)
  assert.equal(regionDeposits(regionAt(11))[0]!.rare, true)
  const regions = depositNeighborhood(origin)
  const forward = regions.flatMap(regionDeposits)
  assert.deepEqual([...regions].reverse().flatMap(regionDeposits).sort((a,b) => a.initial.id.localeCompare(b.initial.id)), [...forward].sort((a,b) => a.initial.id.localeCompare(b.initial.id)))
  for (const p of forward) {
    assert.deepEqual(depositFromId(p.initial.id), p)
    const location = localToSurface(p.entity.anchor.chart!, p.entity.anchor.point.offsetM.slice(0,2) as [number,number])
    assert.equal(surfaceRegion(location, definition).id, p.entity.source!.siteId)
    assert.equal(probeGeology(location, definition, 1.11).collision, false)
    const [x,y] = p.entity.anchor.point.offsetM
    assert.ok([[1.5,0],[-1.5,0],[0,1.5],[0,-1.5]].some(([dx,dy]) => !samplePlanetSurface(localToSurface(p.entity.anchor.chart!, [x+dx!,y+dy!]), definition, {radiusM:.46}).collision))
    assert.equal(p.initial.resourceId, 'mineral')
    assert.ok(Number.isSafeInteger(p.initial.remainingGrams))
  }
  const physical = fixture(0), startX = physical.session.world.entities.find(e=>e.id===PLAYER_ID)!.anchor.point.offsetM[0]
  for(let i=0;i<60;i++) physical.session.advance(stepTopDown(physical.session.world,[-1,0]))
  const stoppedX = physical.session.world.entities.find(e=>e.id===PLAYER_ID)!.anchor.point.offsetM[0]
  assert.ok(startX-stoppedX > .3 && startX-stoppedX < .4, 'visible source footprint blocks actual walking')
  assert.deepEqual(regionDeposits(surfaceRegion({bodyId: 'vesper', radialM: [0,6200005,0]}, geologicalDefinition('vesper'))), [])
  assert.deepEqual(regionDeposits(surfaceRegion({bodyId: 'vesper-moon',radialM:[1400005,0,0]}, planetDefinition('vesper-moon'))), [])
  for (const bad of [forward[0]!.initial.id + ':extra', forward[0]!.initial.id.replace('ee38e47c056e3a66','0000000000000000'), 'source:deposit:v1:garbage']) assert.throws(() => depositFromId(bad))
})

test('P05 production work is atomic, source bound, partial-capacity, interrupted, persisted after eviction/revisit', () => {
  for (const index of [1,11]) {
    const { session, storage, deposit, world } = fixture(index), id = deposit.initial.id
    session.toggleTool(); session.start(id); assert.equal(session.activeSourceId, id)
    ticks(session, 59); session.cancel()
    assert.equal(session.snapshot.inventory.quantities.mineral, undefined)
    const resumed = new ResourceSession(storage, world, undefined, undefined, () => 0)
    assert.equal(resumed.error, '')
    resumed.toggleTool(); resumed.start(id); ticks(resumed, 1)
    assert.equal(resumed.snapshot.inventory.quantities.mineral, 1000)
    assert.equal(resumed.sourceDeltas()[id], 1000)
    assert.equal(resumed.snapshot.sources[id]!.remainingGrams, deposit.initial.remainingGrams - 1000)
    resumed.cancel()
    const savedSources = resumed.snapshot.sources
    const anchor = {frame: 'planet:fixture' as const, chart: regionAt(100), point: point2(0,0)}
    resumed.advance({...resumed.world, tick: resumed.world.tick+1, entities: resumed.world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor}:e)})
    assert.ok(!resumed.world.entities.some(e=>e.id===id))
    resumed.advance({...resumed.world,tick:resumed.world.tick+1,entities:resumed.world.entities.map(e=>e.id===PLAYER_ID?world.entities.find(e=>e.id===PLAYER_ID)!:e)})
    assert.ok(resumed.world.entities.some(e=>e.id===id))
    assert.deepEqual(resumed.snapshot.sources,savedSources)
    const loaded = new ResourceSession(storage, world, undefined, undefined, () => 0)
    assert.equal(loaded.source(id)!.remainingGrams, deposit.initial.remainingGrams - 1000)
    assert.deepEqual(Object.fromEntries(Object.keys(RESOURCE_SOURCES).map(id=>[id,loaded.snapshot.sources[id]])),RESOURCE_SOURCES)
  }
})

test('P05 partial source tail and storage failure cannot duplicate or lose grams', () => {
  const {session,storage,deposit,world,fail,raw}=fixture(0),id=deposit.initial.id
  session.toggleTool();session.start(id);ticks(session,180)
  assert.equal(session.snapshot.inventory.quantities.mineral,deposit.initial.remainingGrams)
  assert.equal(session.snapshot.sources[id]!.remainingGrams,0)
  assert.equal(session.activeSourceId,null)
  const before=raw(); fail()
  assert.equal(session.deposit('mineral',1,session.snapshot.revision),false)
  assert.equal(raw(),before)
  const loaded=new ResourceSession(storage,world,undefined,undefined,()=>0)
  assert.equal(loaded.source(id)!.remainingGrams,0)
  const tampered=JSON.parse(before!);tampered.sources[id].remainingGrams++
  assert.notEqual(new ResourceSession({getItem:()=>JSON.stringify(tampered),setItem:()=>{}},world,undefined,undefined,()=>0).error,'')
})

test('P05 shared full-slot and fractional weight capacity use the ordinary ledger and ship container', () => {
  const {session,deposit}=fixture(11,{gear:{gramsPerItem:1,stackSize:1}}),id=deposit.initial.id
  assert.equal(session.changeItem('add','gear',12,session.snapshot.revision),true)
  session.toggleTool();session.start(id)
  assert.equal(session.activeSourceId,null);assert.match(session.feedback,/slot limit/)
  assert.equal(session.changeItem('remove','gear',1,session.snapshot.revision),true)
  session.start(id);ticks(session,600)
  assert.equal(session.snapshot.inventory.quantities.mineral,9989)
  assert.equal(session.snapshot.sources[id]!.remainingGrams,deposit.initial.remainingGrams-9989)
  assert.match(session.feedback,/weight limit/)
  assert.equal(session.deposit('mineral',9989,session.snapshot.revision),true)
  assert.equal(session.snapshot.inventory.quantities.mineral,undefined)
  assert.equal(Object.values(session.snapshot.stores)[0]!.quantities.mineral,9989)
  assert.equal(session.withdraw('mineral',989,session.snapshot.revision),true)
  assert.equal(session.snapshot.inventory.quantities.mineral,989)
})


test('P05 fresh process, body differentiation, seams and bounded immutable neighborhoods', () => {
  const script = `import {regionDeposits} from './planetDeposits.ts';import {geologicalDefinition} from './planetDefinition.ts';import {surfaceRegion} from './surfaceCoordinates.ts';const d=geologicalDefinition('vesper-moon');console.log(JSON.stringify(regionDeposits(surfaceRegion({bodyId:d.bodyId,radialM:[d.radiusM+5,0,0]},d)).map(p=>({initial:p.initial,point:p.entity.anchor.point.offsetM}))));`
  const raw = execFileSync(process.execPath, ['--experimental-strip-types', '--input-type=module', '-e', script], {cwd: new URL('.', import.meta.url), encoding: 'utf8'})
  assert.deepEqual(JSON.parse(raw), regionDeposits(origin).map(p=>({initial:p.initial,point:p.entity.anchor.point.offsetM})))
  for (const body of ['vesper','vesper-moon','caligo']) {
    const d=geologicalDefinition(body),radius=d.radiusM+5
    for (const direction of [[1,0,0],[0,1,0],[0,0,-1],[1,1,0],[1,1,1]]) {
      const length=Math.hypot(...direction)
      const region=surfaceRegion({bodyId:body,radialM:direction.map(v=>v*radius/length) as [number,number,number]},d)
      const regions=depositNeighborhood(region)
      assert.ok(regions.length<=17)
      for(const candidate of regions.flatMap(regionDeposits)) {
        assert.equal(candidate.entity.anchor.chart!.bodyId,body)
        assert.deepEqual(depositFromId(candidate.initial.id),candidate)
      }
    }
  }
})

test('P05 generated work obeys survival limits and a failed gram write rolls back both sides', () => {
  const registration=fixture(11)
  ticks(registration.session,59)
  const liveSurvival=registration.session.survival
  registration.session.toggleTool();registration.session.start(registration.deposit.initial.id)
  assert.deepEqual(registration.session.snapshot.survival,liveSurvival,'registering a new source preserves live survival progress')
  const {session,deposit}=fixture(11),id=deposit.initial.id
  session.survival={...session.survival,fatigue:388800}
  session.snapshot={...session.snapshot,survival:session.survival}
  session.toggleTool();session.start(id);ticks(session,60)
  assert.equal(session.snapshot.inventory.quantities.mineral,undefined)
  ticks(session,20)
  assert.equal(session.snapshot.inventory.quantities.mineral,1000)
  session.cancel();session.survival={...session.survival,health:0}
  session.start(id);assert.equal(session.activeSourceId,null)
  const {session:working,deposit:other,fail:deny,raw:bytes,storage:store,world:base}=fixture(11)
  working.toggleTool();working.start(other.initial.id);ticks(working,59)
  const before=bytes();deny();ticks(working,1)
  assert.equal(bytes(),before)
  assert.equal(working.snapshot.inventory.quantities.mineral,undefined)
  assert.equal(working.snapshot.sources[other.initial.id]!.remainingGrams,other.initial.remainingGrams)
  assert.notEqual(working.error,'')
  const restored=new ResourceSession(store,base,undefined,undefined,()=>0)
  assert.equal(restored.error,'')
})


test('P05 physical footprint and illustrated direction survive a cube-face chart handoff', () => {
  const d=geologicalDefinition('vesper')
  const corner=surfaceRegion({bodyId:d.bodyId,radialM:scale(normal([1,1,1]),d.radiusM+5)},d)
  const regions=depositNeighborhood(corner),deposit=regions.flatMap(regionDeposits)[0]!,source=deposit.entity,original=source.anchor.chart!
  const target=regions.find(r=>r.cell.axis===1)!
  const base:WorldFrames={planet:{id:'planet:fixture',universeOrigin:ORIGIN,angle:0},ship:{id:'ship:scout',parent:'planet:fixture',origin:point2(0,0),angle:0}}
  const frames=walkingFrames(base,original,target)
  const [x,y]=source.anchor.point.offsetM
  const player={frame:'planet:fixture' as const,chart:original,point:point2(x+1.5,y)}
  const direction=[-dot(original.east,target.east),-dot(original.east,target.north)] as const
  const before=allowedMove(player,[-1,0],[source],base),after=allowedMove(player,direction,[source],frames)
  assert.ok(Math.abs(before-.39)<1e-8)
  assert.ok(Math.abs(after-before)<1e-5,`same physical contact across charts: ${before} vs ${after}`)
  assert.ok(Math.abs(objectAngle(source,frames)-2*Math.PI/3)<1e-4,'source retains its body-space east across the differently oriented face')
})

test('P07 canceled unworked generated sources are compacted while partial work and exact debit journals survive', () => {
  const {session,storage,deposit,world} = fixture(1), id = deposit.initial.id
  session.toggleTool(); session.start(id)
  assert.equal(JSON.parse(storage.getItem()!).sources[id], undefined, 'pristine generated baseline is never stored')
  session.cancel()
  assert.ok(session.markField('Unworked outcrop', 'Return later'))
  assert.equal(session.snapshot.sources[id], undefined)
  assert.equal(session.source(id)!.remainingGrams, deposit.initial.remainingGrams)
  assert.deepEqual(session.sourceDeltas(), {})
  session.start(id); ticks(session, 15); session.cancel()
  assert.equal(session.snapshot.miningWork[id], 240)
  const resumed = new ResourceSession(storage, world, undefined, undefined, () => 0)
  assert.equal(resumed.error, '')
  assert.equal(resumed.fieldMarks.length, 1)
  resumed.toggleTool(); resumed.start(id); ticks(resumed, 45); resumed.cancel()
  assert.equal(resumed.sourceDeltas()[id], 1000)
  const again = new ResourceSession(storage, world, undefined, undefined, () => 0)
  assert.equal(again.sourceDeltas()[id], 1000)
  assert.deepEqual(again.fieldMarks, session.fieldMarks)
})
