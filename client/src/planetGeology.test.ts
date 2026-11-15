import { landingReadout } from './landingHud.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import {test} from 'node:test'
import assert from 'node:assert/strict'
import {spawnSync} from 'node:child_process'
import {geologicalDefinition, GEOLOGICAL_PLANET_DEFINITIONS, planetDefinition, validatePlanetDefinition, definitionDigest} from './planetDefinition.ts'
import {localToSurface, surfaceRegion, surfaceToLocal, neighboringRegions, type SurfaceLocation} from './surfaceCoordinates.ts'
import {authoredObstacles, samplePlanetSurface} from './planetSurface.ts'
import {sampleGeology, sampleGeologyBatch, probeGeology, MAX_TERRAIN_BATCH, WALK_MAX_SLOPE_RADIANS, TERRAIN_CURVATURE_BOUND} from './planetGeology.ts'
import {normal, scale, dot, sub} from './math.ts'
import {resolveSurface, startingSurface, validateSurfaceClearance, surfaceContactRadial, captureSite, siteEntities} from './surfaceSite.ts'
import {surfaceScoutLocation} from './scoutLocation.ts'
import {createTopDownState, stepTopDown, PLAYER_ID, type TopDownState} from './topDownRuntime.ts'
import {point2} from './topDownFrames.ts'
import {relativeM, ORIGIN, translate} from './coordinates.ts'
import {ResourceSession} from './resourceSession.ts'
import {captureScout, decodeScout} from './scoutSave.ts'
import {planLanding, sampleLanding} from './landing.ts'
import {SAMPLE_BODIES} from './sampleUniverse.ts'

const at = (bodyId: string, direction: readonly [number,number,number]): SurfaceLocation => ({bodyId, radialM:scale(normal(direction),geologicalDefinition(bodyId).radiusM+5)})
const ref = (bodyId: string) => {const {version,digest} = geologicalDefinition(bodyId);return {bodyId,version,digest}}
const close = (a:number,b:number,tolerance=1e-6) => assert.ok(Math.abs(a-b)<tolerance,`${a} != ${b}`)

test('AC-1/5 immutable v2 publications coexist with unchanged v1; malformed parameters fail', () => {
  assert.deepEqual(GEOLOGICAL_PLANET_DEFINITIONS.map(d=>d.digest),['263916104a5852c0','ee38e47c056e3a66','71194ec053cb6da7'])
  for(const d of GEOLOGICAL_PLANET_DEFINITIONS) {
    assert.equal(validatePlanetDefinition(structuredClone(d)),d)
    const {digest,...content}=d;assert.equal(definitionDigest(content),digest)
    assert.equal(planetDefinition(d.bodyId).version,1)
    for(const change of [{radiusM:-1},{version:99},{domains:{...d.domains,terrain:{...d.domains.terrain,parameters:{reliefM:Infinity}}}}]) assert.throws(()=>validatePlanetDefinition({...d,...change} as any))
  }
})

test('AC-1/2 physical samples survive fresh process, reverse order, batches and visual LOD', () => {
  const locations = GEOLOGICAL_PLANET_DEFINITIONS.flatMap(d=>[[1,0,0],[1,1,1],[-1,0,1e-12],[0,1,0],[0,-1,0]].map(v=>at(d.bodyId,v as [number,number,number])))
  const expected=locations.map(l=>sampleGeology(l,ref(l.bodyId)))
  assert.deepEqual([...locations].reverse().map(l=>sampleGeology(l,ref(l.bodyId))).reverse(),expected)
  const script=`import {sampleGeology} from './src/planetGeology.ts';import {geologicalDefinition} from './src/planetDefinition.ts';console.log(JSON.stringify(${JSON.stringify(locations)}.map(l=>sampleGeology(l,geologicalDefinition(l.bodyId)))))`
  const fresh=spawnSync(process.execPath,['--experimental-strip-types','--input-type=module','-e',script],{cwd:new URL('..',import.meta.url),encoding:'utf8'})
  assert.equal(fresh.status,0,fresh.stderr);assert.deepEqual(JSON.parse(fresh.stdout),expected)
  assert.equal(new Set(expected.map(s=>s.heightM)).size,expected.length)
  for(const [i,l] of locations.entries()) {const s=expected[i]!;close(dot(s.normalBody,s.normalBody),1,1e-12);assert.ok(s.slopeRadians>=0 && s.slopeRadians<Math.PI/2);assert.equal(samplePlanetSurface(l,ref(l.bodyId)).heightM,s.heightM)}
  for(const d of GEOLOGICAL_PLANET_DEFINITIONS) {
    const points=locations.filter(l=>l.bodyId===d.bodyId),all=sampleGeologyBatch(points,d)
    assert.deepEqual(all,points.map(l=>sampleGeology(l,d)))
    assert.deepEqual(sampleGeologyBatch(points.filter((_,i)=>i%2===0),d),all.filter((_,i)=>i%2===0))
  }
  assert.equal(sampleGeology(at('vesper-moon',[0,1,0]),ref('vesper-moon')).groundType,'polar-ice')
  assert.notEqual(sampleGeology(at('caligo',[0,1,0]),ref('caligo')).groundType,'polar-ice')
  for (const [bodyId,height,slope] of [['vesper',97.2034305800413,.0018804623307216823],['vesper-moon',140.3087564130005,.10237827469542767],['caligo',110.69806459519052,.003284424897559978]] as const) {
    const sample=sampleGeology(at(bodyId,[1,0,0]),ref(bodyId));close(sample.heightM,height,1e-7);close(sample.slopeRadians,slope,1e-9)
  }
})

test('AC-2 physical borders agree across face seams, poles and authored transition',()=>{
  for(const d of GEOLOGICAL_PLANET_DEFINITIONS) for(const direction of [[1,1,0],[1,1,1],[-1,0,1e-12],[0,1,0],[0,-1,0]] as const) {
    const l=at(d.bodyId,direction),region=surfaceRegion(l,d),expected=sampleGeology(l,d)
    for(const neighbor of neighboringRegions(region)) {
      const back=localToSurface(neighbor,surfaceToLocal(l,neighbor)),sample=sampleGeology(back,d)
      close(sample.heightM,expected.heightM);close(sample.slopeRadians,expected.slopeRadians)
      assert.ok(Math.hypot(...sub(back.radialM,l.radialM))<1e-7)
    }
  }
  const d=ref('vesper'),region=surfaceRegion(startingSurface(),d)
  for(const x of [18,200,2000]) {
    const left=sampleGeology(localToSurface(region,[x-1e-4,0]),d),right=sampleGeology(localToSurface(region,[x+1e-4,0]),d)
    close(left.heightM,right.heightM,.001);close(left.slopeRadians,right.slopeRadians,.001)
  }
})

function strip() {
  const definition=ref('caligo'),region=surfaceRegion(at('caligo',[1,1,1]),definition)
  return {definition,region,points:Array.from({length:241},(_,i)=>{const x=i-120,location=localToSurface(region,[x,0]);return {x,location,s:sampleGeology(location,definition)}})}
}
test('AC-3/5 steep ground is swept without tunnelling; player and Scout share blocked footprints',()=>{
  const {definition,region,points}=strip()
  const blocked=points.find(p=>p.s.slopeRadians>WALK_MAX_SLOPE_RADIANS+.02)!
  assert.ok(blocked,'fixed region has genuinely steep terrain')
  assert.equal(samplePlanetSurface(blocked.location,definition,{radiusM:.46,landingYaw:0}).collision,true)
  assert.throws(()=>validateSurfaceClearance(resolveSurface({...blocked.location,yaw:0,definition})),/Unsafe/)
  const pair=points.find((p,i)=>i+48<points.length && !probeGeology(p.location,definition,.46).collision && points.slice(i+1,i+49).some(q=>q.s.slopeRadians>WALK_MAX_SLOPE_RADIANS))!
  assert.ok(pair,'clear start with blocked ground along a long sweep')
  const sweep=probeGeology(pair.location,definition,.46,[48,0]);assert.ok(sweep.moveFraction<1)
  const endpoint=localToSurface(region,[pair.x+48*sweep.moveFraction,0]);assert.equal(probeGeology(endpoint,definition,.46).collision,false)
  const scout=surfaceScoutLocation({...at('caligo',[1,0,0]),yaw:0,definition})
  let world:TopDownState={...createTopDownState(),scoutLocation:{...scout,surface:{...at('caligo',[1,1,1]),yaw:0,definition},binding:region.id},frames:resolveSurface({...at('caligo',[1,1,1]),yaw:0,definition}).frames}
  world={...world,entities:world.entities.filter(e=>e.id===PLAYER_ID).map(e=>({...e,anchor:{frame:'planet:fixture' as const,point:point2(pair.x,0)}}))}
  for(let i=0;i<1600;i++) world=stepTopDown(world,[1,0])
  assert.equal(world.feedback,'Ground too steep for walking. Follow the gentler terrain.')
  const [x,y]=relativeM(world.entities[0]!.anchor.point,ORIGIN)
  assert.ok(x<pair.x+48);assert.equal(probeGeology(localToSurface(region,[x,y]),definition,.46).collision,false)
  for(let i=1;i<points.length;i++) assert.ok(Math.abs(points[i]!.s.grade-points[i-1]!.s.grade)<TERRAIN_CURVATURE_BOUND)
  const safeSite=resolveSurface(scout.surface),safeRegion=surfaceRegion(scout.surface,definition)
  const invalidAnchor={frame:'planet:fixture' as const,point:point2(...surfaceToLocal(blocked.location,safeRegion))}
  const initial=createTopDownState()
  const saveWorld={...initial,scoutLocation:scout,frames:safeSite.frames,entities:[...initial.entities.filter(e=>e.layer!=='terrain').map(e=>e.id===PLAYER_ID?{...e,anchor:invalidAnchor}:e),...siteEntities(safeSite)]}
  const invalid=new ResourceSession({getItem:()=>null,setItem:()=>{}},saveWorld)
  const raw=JSON.stringify(invalid.snapshot,(_,v)=>typeof v==='bigint'?String(v):v)
  assert.notEqual(new ResourceSession({getItem:()=>raw,setItem:()=>{}},createTopDownState()).error,'','reload cannot place a player inside generated blocked ground')
})

test('AC-3/4 elevated Scout contact, continuous landing endpoint and revision-bound reload',()=>{
  const definition=ref('caligo'),address={...at('caligo',[1,0,0]),yaw:0,definition},site=resolveSurface(address)
  assert.doesNotThrow(()=>validateSurfaceClearance(site))
  const scout=surfaceScoutLocation(address),sample=samplePlanetSurface(address,definition),radial=surfaceContactRadial(address)
  close(Math.hypot(...radial)-(geologicalDefinition('caligo').radiusM+5),sample.heightM)
  const body=SAMPLE_BODIES.find(b=>b.id==='caligo')!,up=normal(radial)
  const anchor={...scout.ship,mode:'flying' as const,contact:null,position:translate(scout.ship.position,scale(up,100))}
  const plan=planLanding(anchor,body,address,SAMPLE_BODIES),landed=sampleLanding(plan,body,plan.startTick+plan.durationTicks)
  assert.ok(Math.hypot(...relativeM(landed.position,scout.ship.position))<1e-6)
  const navigation=new NavigationRuntime(SAMPLE_BODIES,scout);navigation.advance(1/60)
  close(landingReadout(navigation).altitudeM!,0,.001)
  const targetTelemetry=new NavigationRuntime(SAMPLE_BODIES,createTopDownState().scoutLocation)
  targetTelemetry.clock.state={...anchor,position:anchor.position}
  targetTelemetry.scanner.contacts=[body];targetTelemetry.scanner.selected=body.id
  const telemetrySession=new ResourceSession({getItem:()=>null,setItem:()=>{}},createTopDownState())
  targetTelemetry.surfaceDefinitionForBody=id=>telemetrySession.bodyDefinition(id)
  close(landingReadout(targetTelemetry).altitudeM!,100,.001)
  // An imported v1 first-visit landing plan is authoritative before that body
  // has any completed site; telemetry must not silently adopt the v2 default.
  telemetrySession.world.scoutLocation.landing={...plan,surface:{...address,definition:undefined}}
  close(landingReadout(targetTelemetry).altitudeM!,100+sample.heightM,.001)
  const encoded=JSON.parse(JSON.stringify(captureScout(scout),(_,v)=>typeof v==='bigint'?String(v):v))
  assert.deepEqual(decodeScout(encoded),scout)
  encoded.surface.definition.digest='wrong';assert.throws(()=>decodeScout(encoded),/mismatch/)
  const legacy=resolveSurface({...address,definition:undefined});assert.equal(surfaceContactRadial(legacy.address)[0],legacy.address.radialM[0])
  assert.equal(legacy.id,site.id)
  let raw:string|null=null;const storage={getItem:()=>raw,setItem:(_:string,value:string)=>{raw=value}}
  const session=new ResourceSession(storage,createTopDownState('interior',true))
  assert.equal(session.error,'');session.checkpoint();raw??=JSON.stringify(session.snapshot,(_,v)=>typeof v==='bigint'?String(v):v);const old=raw!
  const reopened=new ResourceSession(storage,createTopDownState('interior',true));assert.equal(reopened.error,'');assert.equal(raw,old)
  assert.deepEqual(reopened.snapshot.sources,session.snapshot.sources)
  assert.deepEqual(captureSite(resolveSurface(startingSurface()),siteEntities(resolveSurface(startingSurface()))),session.snapshot.sites[resolveSurface(startingSurface()).id])
})

test('AC-4 authored Vesper remains exactly flat with original source IDs and landing clearance',()=>{
  const definition=ref('vesper'),location=startingSurface(),sample=samplePlanetSurface(location,definition,{landingYaw:location.yaw})
  assert.equal(sample.heightM,0);assert.equal(sample.slopeRadians,0);assert.equal(sample.landingClear,true)
  const anchors=Object.fromEntries(authoredObstacles().map(o=>[o.id,[o.x,o.y] as const]))
  assert.throws(()=>samplePlanetSurface(location,definition,{}, {definition:planetDefinition('vesper'),anchors}),/revision mismatch/)
  const region=surfaceRegion(location,definition)
  for(const [x,y,id] of [[-9,0,'source:landing:mineral:01'],[-5,6,'source:landing:biology:01']] as const) assert.deepEqual(samplePlanetSurface(localToSurface(region,[x,y]),definition).sourceIds,[id])
})
test('AC-5 workload budgets reject invalid/extreme probes deterministically',()=>{
  const l=at('caligo',[1,0,0]),d=ref('caligo')
  for(const radius of [-1,5,Infinity,NaN]) assert.throws(()=>probeGeology(l,d,radius))
  for(const motion of [[65,0],[NaN,0],[0,0,0]]) assert.throws(()=>probeGeology(l,d,0,motion as any))
  assert.throws(()=>sampleGeologyBatch(Array(MAX_TERRAIN_BATCH+1).fill(l),d),/bounded/)
  assert.throws(()=>sampleGeology(l,{...d,version:3}),/mismatch/)
  assert.throws(()=>sampleGeology(l,planetDefinition('caligo')),/Unsupported/)
  assert.throws(()=>sampleGeology({bodyId:'nacre',radialM:[1,0,0]},d))
  assert.equal(sampleGeologyBatch([],d).length,0)
})
