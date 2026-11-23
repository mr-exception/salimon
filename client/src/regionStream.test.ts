import { SCOUT_CORE_FUEL_ID } from './scoutEnergyCore.ts'
import {test} from 'node:test'
import assert from 'node:assert/strict'
import {RegionStream,STREAM_REGION_LIMIT,STREAM_SAMPLE_LIMIT,STREAM_FRAME_BUDGET} from './regionStream.ts'
import {geologicalDefinition,planetDefinition} from './planetDefinition.ts'
import {surfaceRegion,localToSurface} from './surfaceCoordinates.ts'
import {createTopDownState,stepTopDown,PLAYER_ID,capturePlayerLocation,drawList,type TopDownState} from './topDownRuntime.ts'
import {surfaceScoutLocation} from './scoutLocation.ts'
import {startingSurface,resolveSurface,siteEntities} from './surfaceSite.ts'
import {point2,relativeAnchor,walkingFrames,worldToView} from './topDownFrames.ts'
import {ORIGIN,relativeM} from './coordinates.ts'
import {ResourceSession} from './resourceSession.ts'

function flatWorld(bodyId='vesper',radialM:readonly[number,number,number]=[0,6200005,0]):TopDownState {
  const address=bodyId==='vesper' ? {...startingSurface(),definition:undefined} : {bodyId,radialM,yaw:0},scoutLocation=surfaceScoutLocation(address),site=resolveSurface(address)
  const base=createTopDownState(),anchor={frame:'planet:fixture' as const,point:point2(-20,-20)}
  return {...base,scoutLocation,frames:scoutLocation.frames,camera:{...base.camera,anchor},entities:[...base.entities.filter(e=>e.layer!=='terrain').map(e=>e.id===PLAYER_ID?{...e,anchor}:e),...siteEntities(site)]}
}
function physical(world:ReturnType<typeof flatWorld>) {
  const anchor=capturePlayerLocation(world).anchor
  return localToSurface(anchor.chart??surfaceRegion(world.scoutLocation.surface),relativeM(anchor.point,ORIGIN).slice(0,2) as [number,number])
}

test('P03 continuous four-direction walking crosses multiple regions, rebases and returns to physical Scout',()=>{
  for(const direction of [[1,0],[-1,0],[0,1],[0,-1]] as const) {
    let world=flatWorld(),start=physical(world),regions=new Set<string>()
    for(let i=0;i<11000;i++) {world=stepTopDown(world,direction);if(i%100===0)regions.add(surfaceRegion(physical(world)).id)}
    assert.ok(regions.size>=6)
    assert.ok(world.frames.surface)
    assert.equal(drawList(world).filter(x=>x.entity.id==='ship:scout').length,0)
    const last=world.entities.find(e=>e.id===PLAYER_ID)!.anchor
    assert.ok(Math.hypot(...relativeAnchor(last,world.camera.anchor,world.frames))<1)
    for(let i=0;i<11000;i++)world=stepTopDown(world,[-direction[0],-direction[1]])
    assert.ok(Math.hypot(...physical(world).radialM.map((n,i)=>n-start.radialM[i]!))<.001)
    assert.equal(world.frames,world.scoutLocation.frames)
    assert.equal(drawList(world).filter(x=>x.entity.id==='ship:scout').length,1)
  }
})

test('P03 face winding, antimeridian and pole routes keep physical position continuous',()=>{
  const r=planetDefinition('vesper-moon').radiusM+5
  for(const v of [[0,1,0],[0,-1,0],[-1,0,0],[1,1,1]] as const) {
    const length=Math.hypot(...v),radial=v.map(n=>n*r/length) as [number,number,number]
    let world=flatWorld('vesper-moon',radial),before=physical(world)
    for(let i=0;i<1400;i++) {
      world=stepTopDown(world,[1,0],10)
      const next=physical(world),distance=Math.hypot(...next.radialM.map((n,j)=>n-before.radialM[j]!))
      assert.ok(distance>.399 && distance<.401)
      assert.ok(Math.abs(Math.hypot(...next.radialM)-r)<1e-6)
      before=next
    }
  }
})

test('P03 eviction stress is deterministic, bounded, rejects excess work and restores identical physical samples',()=>{
  const definition=geologicalDefinition('vesper'),region=surfaceRegion({bodyId:'vesper',radialM:[0,definition.radiusM+5,0]},definition)
  const run=()=>{
    const stream=new RegionStream(),origin=localToSurface(region,[0,0])
    stream.select(origin,definition);const first=stream.sample(origin)
    for(let i=0;i<100;i++) {
      const p=localToSurface(region,[i*128,40]);stream.select(p,definition)
      for(let j=0;j<32;j++)stream.sample(localToSurface(region,[i*128+j,40]))
      stream.prefetch()
      assert.ok(stream.stats.regions<=STREAM_REGION_LIMIT)
      assert.ok(stream.stats.samples<=STREAM_REGION_LIMIT*STREAM_SAMPLE_LIMIT)
      assert.ok(stream.stats.frameGenerated<=STREAM_FRAME_BUDGET)
    }
    assert.ok(stream.stats.evicted>100)
    stream.select(origin,definition);assert.deepEqual(stream.sample(origin),first);stream.prefetch()
    return stream.stats
  }
  assert.deepEqual(run(),run())
  const stream=new RegionStream();stream.select(localToSurface(region,[0,0]),definition)
  for(let i=0;i<STREAM_FRAME_BUDGET;i++)stream.sample(localToSurface(region,[i/10,0]))
  assert.throws(()=>stream.sample(localToSurface(region,[1000,0])),/budget/)
  stream.select(localToSurface(region,[1000,0]),definition);stream.sample(localToSurface(region,[1000,0]))
})

test('P03 rebased player save reload retains coordinate and source journal; forged chart fails safely',()=>{
  let world=flatWorld()
  for(let i=0;i<850;i++)world=stepTopDown(world,[-1,0],10)
  let raw:string|null=null
  const storage={getItem:()=>raw,setItem:(_:string,v:string)=>{raw=v}}
  const session=new ResourceSession(storage,world)
  assert.equal(session.transfer({revision:session.snapshot.revision,from:{kind:'source',id:'source:landing:mineral:01'},to:{kind:'player'},resourceId:'mineral',grams:1000}),true)
  session.world=stepTopDown(session.world,[-1,0]);session.checkpoint();assert.equal(session.error,'');assert.ok(raw)
  const reload=new ResourceSession(storage,createTopDownState())
  assert.equal(reload.error,'');assert.deepEqual(physical(reload.world),physical(session.world))
  assert.deepEqual(reload.snapshot.sources,session.snapshot.sources)
  assert.equal(reload.snapshot.sources['source:landing:mineral:01']!.remainingGrams,59000)
  const data=JSON.parse(raw!);data.location.anchor.chart.east=[0,0,0];raw=JSON.stringify(data)
  assert.match(new ResourceSession(storage,createTopDownState()).error,/could not be loaded safely/)
})

test('P03 v2 geological walking preserves shared collision and camera follow across chart handoff',()=>{
  let world=flatWorld()
  const scoutLocation=surfaceScoutLocation(startingSurface())
  world={...world,scoutLocation,frames:scoutLocation.frames}
  let maximumLag=0
  for(let i=0;i<8500;i++) {
    world=stepTopDown(world,[-1,0])
    const player=world.entities.find(e=>e.id===PLAYER_ID)!.anchor
    maximumLag=Math.max(maximumLag,Math.hypot(...relativeAnchor(player,world.camera.anchor,world.frames)))
  }
  assert.ok(world.frames.surface)
  assert.ok(maximumLag<.22)
  assert.ok(physical(world).radialM[0]<-350)
})

test('P03 v11 anchor migration is lossless; rebased metadata cannot masquerade as an old save',()=>{
  const world=flatWorld(),session=new ResourceSession({getItem:()=>null,setItem:()=>{}},world)
  const data=JSON.parse(JSON.stringify(session.snapshot,(_,v)=>typeof v==='bigint'?String(v):v))
  data.version=11; delete data.convertedGrams; delete data.core; delete data.stores[SCOUT_CORE_FUEL_ID]; delete data.journal
  let raw=JSON.stringify(data)
  const storage={getItem:()=>raw,setItem:(_:string,v:string)=>{raw=v}}
  const migrated=new ResourceSession(storage,createTopDownState())
  assert.equal(migrated.error,'');assert.equal(migrated.snapshot.version,15)
  assert.deepEqual(migrated.snapshot.location,session.snapshot.location)
  assert.deepEqual(migrated.snapshot.sources,session.snapshot.sources)
  assert.equal(JSON.parse(raw).version,11) // read migration does not rewrite storage
  data.location.anchor.chart=surfaceRegion(world.scoutLocation.surface)
  raw=JSON.stringify(data)
  assert.match(new ResourceSession(storage,createTopDownState()).error,/could not be loaded safely/)
})


test('P03 distant heading poles transport the camera without an orientation snap',()=>{
  let world=flatWorld('vesper-moon',[1400005,0,0])
  const landing=surfaceRegion(world.scoutLocation.surface)
  const chart=surfaceRegion({bodyId:'vesper-moon',radialM:[0,0,1400005]})
  const anchor={frame:'planet:fixture' as const,point:point2(250,0),chart}
  world={...world,frames:walkingFrames(world.scoutLocation.frames,landing,chart),entities:world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor}:e),camera:{...world.camera,anchor}}
  let before=[0,0] as readonly[number,number]
  for(let i=0;i<500;i++) {
    world=stepTopDown(world,[1,0])
    const player=world.entities.find(e=>e.id===PLAYER_ID)!.anchor
    const next=worldToView(relativeAnchor(player,world.camera.anchor,world.frames),world.frames)
    assert.ok(Math.hypot(next[0]-before[0],next[1]-before[1])<.05)
    before=next
  }
})

test('P08 rapid direction/publication changes discard stale queued regions and reject unknown publications atomically',()=>{
  const stream=new RegionStream(),d=geologicalDefinition('vesper-moon')
  const origin=surfaceRegion({bodyId:d.bodyId,radialM:[d.radiusM+5,0,0]},d),home=localToSurface(origin,[0,0])
  stream.select(home,d);const baseline=stream.sample(home)
  for(let i=0;i<200;i++) {
    const sign=i%2?1:-1,point=localToSurface(origin,[sign*i*128,(i%3-1)*i*128])
    stream.select(point,d)
    // Replace pending work again before prefetch: no old destination may leak.
    stream.select(home,d);stream.prefetch()
    assert.deepEqual(stream.sample(home),baseline)
    assert.ok(stream.stats.regions<=STREAM_REGION_LIMIT)
    assert.ok(stream.stats.samples<=STREAM_REGION_LIMIT*STREAM_SAMPLE_LIMIT)
    assert.ok(stream.stats.frameGenerated<=STREAM_FRAME_BUDGET)
    assert.equal(stream.stats.pending,0)
  }
  const flat=planetDefinition(d.bodyId)
  stream.select(home,flat);assert.equal(stream.sample(home).groundType,'authored-ground')
  stream.select(home,d);assert.deepEqual(stream.sample(home),baseline)
  for(const reference of [{...d,digest:'unknown'},{...d,bodyId:'caligo'},{...d,version:99}]) {
    assert.throws(()=>stream.select(home,reference))
    assert.deepEqual(stream.sample(home),baseline,'rejected selection preserves valid publication')
  }
})
