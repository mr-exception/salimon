import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createFieldMark, localLandmarks, regionLandmarks, recordFieldMark, scoutGuidance, updateWalkingFix, MAX_FIELD_MARKS } from './exploration.ts'
import { createTopDownState, PLAYER_ID, rechartWalking, stepTopDown } from './topDownRuntime.ts'
import { normal, scale } from './math.ts'
import { point2, worldToView, relativeAnchor } from './topDownFrames.ts'
import { surfaceRegion, localToSurface, neighboringRegions, surfaceToLocal } from './surfaceCoordinates.ts'
import { geologicalDefinition, planetDefinition } from './planetDefinition.ts'
import { ResourceSession } from './resourceSession.ts'
import { probeGeology } from './planetGeology.ts'
import { samplePlanetSurface } from './planetSurface.ts'

const storage = () => { let raw: string | null = null; return { getItem: () => raw, setItem: (_: string, value: string) => { raw = value } } }
const at = (x:number,y:number, base=createTopDownState()) => ({...base,entities:base.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'planet:fixture' as const,point:point2(x,y)}}:e)})

test('explicit inspect and mark preserve physical source identity, optional text and bounded durable events',()=>{
  const world=createTopDownState(),session=new ResourceSession(storage(),world)
  const before=structuredClone(session.snapshot),landmarks=localLandmarks(world)
  assert.ok(landmarks.length>0);assert.equal(session.fieldMarks.length,0)
  const rock=landmarks.find(l=>l.physicalSource==='rock:west')!
  assert.deepEqual(rock.footprintM,[1.7,1.4])
  assert.ok(session.markField('West marker','Return around this boulder',rock.id))
  assert.equal(session.fieldMarks[0]!.landmarkId,rock.id)
  assert.equal(session.fieldMarks[0]!.provenance,'known-physical-source')
  assert.equal(session.snapshot.revision,before.revision+1);assert.deepEqual(session.snapshot.inventory,before.inventory)
  assert.ok(session.markField('Renamed','',rock.id));assert.equal(session.fieldMarks.length,1)
  assert.equal(session.fieldMarks[0]!.name,'Renamed')
  assert.equal(session.markField('x'.repeat(49),''),false)
  assert.equal(session.markField('','line\nbreak'),false)
  assert.equal(session.markField('','', 'invisible:far'),false)
  assert.equal(session.fieldMarks.length,1)
  assert.equal(createFieldMark(world,'','').provenance,'personally-observed-estimate')
  assert.equal(createFieldMark(world,'note','text').provenance,'user-note')
  let marks=Array.from({length:MAX_FIELD_MARKS},(_,i)=>({...createFieldMark(world,'',''),id:`mark:${i}`}))
  assert.throws(()=>recordFieldMark(marks,createFieldMark(at(1,2),'','')),/full/)
  marks=[...recordFieldMark(marks,{...marks[0]!,name:'Updated'})];assert.equal(marks.length,MAX_FIELD_MARKS)
  const cabin={...world,entities:world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout' as const,point:point2(0,0)}}:e)}
  assert.throws(()=>createFieldMark(cabin,'',''),/surface/)
  assert.equal(scoutGuidance(cabin).available,false)
})

test('geological landmarks agree with physical terrain across streaming order, reload and neighboring cells',()=>{
  const d=geologicalDefinition('caligo'),origin=surfaceRegion({bodyId:d.bodyId,radialM:scale(normal([1,1,1]),d.radiusM+5)},d)
  const regions=Array.from({length:100},(_,i)=>surfaceRegion(localToSurface(origin,[i*64,1000]),d))
  const samples=regions.flatMap(regionLandmarks)
  assert.ok(samples.length>0);assert.ok(samples.length<regions.length)
  const original=JSON.stringify(samples)
  const reverse=[...regions].reverse().flatMap(regionLandmarks).sort((a,b)=>a.id.localeCompare(b.id))
  assert.deepEqual(reverse,[...samples].sort((a,b)=>a.id.localeCompare(b.id)))
  for(const landmark of samples) {
    const region=surfaceRegion(landmark.location,d),physical=samplePlanetSurface(landmark.location,d),geology=probeGeology(landmark.location,d)
    assert.equal(landmark.physicalSource,geology.geologyId)
    assert.equal(physical.collision,geology.collision)
    assert.ok(geology.grade>=.7)
    const all=[region,...neighboringRegions(region)].flatMap(regionLandmarks)
    assert.equal(all.filter(l=>l.id===landmark.id).length,1)
    assert.deepEqual(regionLandmarks(JSON.parse(JSON.stringify(region))),regionLandmarks(region))
  }
  assert.equal(JSON.stringify(regions.flatMap(regionLandmarks)),original)
  const legacy=planetDefinition('caligo')
  assert.deepEqual(regionLandmarks(surfaceRegion({bodyId:legacy.bodyId,radialM:[legacy.radiusM+5,0,0]},legacy)),[])
})

test('local walking guidance uses actual ship frame, uncertainty and a spent travel budget',()=>{
  const initial=createTopDownState(),near=at(2,12),far=at(-140,30)
  let fix=updateWalkingFix(initial,initial,null)
  assert.ok(fix);assert.equal(scoutGuidance(far).available,false)
  fix=updateWalkingFix(initial,far,fix)
  const guidance=scoutGuidance(far,fix)
  assert.ok(guidance.available);assert.match(guidance.text,/roughly.*±.*estimate/)
  assert.deepEqual(guidance.direction,worldToView(relativeAnchor({frame:'ship:scout',point:point2(0,0)},far.entities.find(e=>e.id===PLAYER_ID)!.anchor,far.frames),far.frames))
  const outside=at(500,30)
  assert.equal(updateWalkingFix(far,outside,fix),null)
  assert.equal(scoutGuidance(outside,fix).available,false)
  assert.equal(updateWalkingFix(outside,far,null),null)
  assert.ok(updateWalkingFix(far,near,null))
  assert.equal(scoutGuidance(initial,updateWalkingFix(initial,initial,null)).available,true)
})

test('marks and Scout retain physical identity through chart handoff; commands never touch balances',()=>{
  const initial=createTopDownState(),mark=createFieldMark(initial,'Start',''),far=at(-300,30)
  const world=rechartWalking(far),chart=world.frames.surface!.chart
  const physical=localToSurface(chart,world.entities.find(e=>e.id===PLAYER_ID)!.anchor.point.offsetM.slice(0,2) as [number,number])
  const reference=planetDefinition('vesper'),landing=surfaceRegion(world.scoutLocation.surface,reference)
  const local=surfaceToLocal(physical,landing)
  assert.ok(Math.hypot(local[0]+300,local[1]-30)<1e-7)
  assert.deepEqual(world.scoutLocation,initial.scoutLocation)
  const startFix=updateWalkingFix(initial,initial,null), farFix=updateWalkingFix(initial,far,startFix)
  const uncharted=scoutGuidance(far,farFix),charted=scoutGuidance(world,farFix)
  assert.ok(uncharted.available && charted.available)
  assert.ok(Math.abs(Math.hypot(...uncharted.direction!)-Math.hypot(...charted.direction!))<.001)
  assert.ok(Math.abs(Math.atan2(...uncharted.direction!)-Math.atan2(...charted.direction!))<.001)
  const changedHome={...world,scoutLocation:{...world.scoutLocation,surface:{...world.scoutLocation.surface,yaw:0}}}
  assert.equal(scoutGuidance(changedHome,farFix).available,false)
  assert.deepEqual(createFieldMark(initial,'Start',''),mark)
  const nearby=at(-.000000001,0)
  assert.equal(createFieldMark(nearby,'','').id,createFieldMark(at(0,0),'','').id)
  // Marks live in the existing session, outside immutable terrain cache residency.
  const session=new ResourceSession(storage(),initial)
  session.markField('Start','')
  for(let i=0;i<60;i++)session.advance(stepTopDown(session.world,[0,1]))
  assert.equal(session.fieldMarks[0]!.id,mark.id)
  assert.equal(session.snapshot.inventory.quantities.mineral,undefined)
})
