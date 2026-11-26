import { test } from 'node:test'
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { ENVIRONMENT_PUBLICATION, ENVIRONMENT_RULES, SCOUT_SAFE_ENVIRONMENT, samplePlanetEnvironment, bodyEnvironmentSummary } from './planetEnvironment.ts'
import { definitionDigest, planetDefinition, geologicalDefinition } from './planetDefinition.ts'
import { surfaceRegion, surfaceToLocal, localToSurface, type SurfaceLocation } from './surfaceCoordinates.ts'
import { RegionStream } from './regionStream.ts'
import { normal, scale, type Vec3 } from './math.ts'
import { ResourceSession } from './resourceSession.ts'
import { createTopDownState } from './topDownRuntime.ts'
const point = (bodyId: string, direction: Vec3): SurfaceLocation => ({ bodyId, radialM: scale(normal(direction), planetDefinition(bodyId).radiusM + 5) })
class Storage { raw: string | null = null; fail = false; getItem() { return this.raw }; setItem(_k: string,v: string) { if(this.fail) throw Error('quota'); this.raw=v } }
const open = (storage = new Storage()) => new ResourceSession(storage,createTopDownState(),undefined,undefined,()=>0)
test('published descriptor and golden world-seed/body/cell vectors are frozen across process and catalog revisions', () => {
 assert.equal(ENVIRONMENT_PUBLICATION,'f9512d32ccbcfd7e'); assert.equal(definitionDigest(ENVIRONMENT_RULES),ENVIRONMENT_PUBLICATION)
 assert.ok(Object.isFrozen(ENVIRONMENT_RULES)); assert.ok(Object.isFrozen(SCOUT_SAFE_ENVIRONMENT))
 for(const [bodyId,temperature,rad,tox] of [['vesper',85,12,16],['vesper-moon',62,11,13],['caligo',72,18,6]] as const) {
  const location=point(bodyId,[1,0,0]), sample=samplePlanetEnvironment(location)
  assert.equal(sample.temperatureC,temperature);assert.equal(sample.radiationHpPerSecond,rad);assert.equal(sample.toxicityHpPerSecond,tox)
  assert.deepEqual(samplePlanetEnvironment(location,geologicalDefinition(bodyId)),sample)
  const run=spawnSync(process.execPath,['--experimental-strip-types','--input-type=module','-e',`import {samplePlanetEnvironment} from './src/planetEnvironment.ts'; console.log(JSON.stringify(samplePlanetEnvironment(${JSON.stringify(location)})))`],{cwd:process.cwd(),encoding:'utf8'})
  assert.equal(run.status,0,run.stderr);assert.deepEqual(JSON.parse(run.stdout),sample)
 }
})
test('all supported body/region queries are finite, bounded, repeatable and regionally variable; body baselines stay fixed', () => {
 for(const bodyId of ['vesper','vesper-moon','caligo']) {
  const temperatures=new Set<number>(), summary=bodyEnvironmentSummary(bodyId)
  assert.deepEqual(summary.temperatureRangeC,[-180,180])
  for(let i=0;i<150;i++) {
   const location=point(bodyId,[Math.cos(i*1.7),Math.sin(i*.91),Math.cos(i*.43)]), sample=samplePlanetEnvironment(location)
   assert.deepEqual(samplePlanetEnvironment(JSON.parse(JSON.stringify(location))),sample)
   assert.ok(Number.isInteger(sample.temperatureC)&&sample.temperatureC>=-180&&sample.temperatureC<=180)
   for(const n of [sample.radiationHpPerSecond,sample.toxicityHpPerSecond]) assert.ok(Number.isFinite(n)&&n>=0&&n<=20)
   assert.equal(sample.radiationHpPerSecond,summary.radiationHpPerSecond);assert.equal(sample.toxicityHpPerSecond,summary.toxicityHpPerSecond)
   temperatures.add(sample.temperatureC)
  }
  assert.ok(temperatures.size>20)
 }
})
test('safe authored square is inclusive ±128 m, positive Y only, independent of region/chart/landing position', () => {
 const definition=planetDefinition('vesper'), origin=surfaceRegion(point('vesper',[0,1,0]),definition)
 for(const x of [-128,0,128]) for(const z of [-128,0,128]) {
  const sample=samplePlanetEnvironment(localToSurface(origin,[x,z]),definition)
  assert.equal(sample.safeStartingRegion,true)
  assert.deepEqual([sample.temperatureC,sample.radiationHpPerSecond,sample.toxicityHpPerSecond],[24,0,0])
 }
 for(const [x,z] of [[128.000001,0],[-128.000001,0],[0,128.000001],[0,-128.000001],[128.000001,128]]) assert.equal(samplePlanetEnvironment(localToSurface(origin,[x!,z!]),definition).safeStartingRegion,false)
 const back=point('vesper',[0,-1,0]);assert.equal(samplePlanetEnvironment(back).safeStartingRegion,false)
 assert.equal(samplePlanetEnvironment(point('vesper-moon',[0,1,0])).safeStartingRegion,false)
 // A generated chart describes the same point; it cannot move the safe zone.
 const location=localToSurface(origin,[100,80]), chart=surfaceRegion(location,definition)
 assert.deepEqual(samplePlanetEnvironment(localToSurface(chart,surfaceToLocal(location,chart)),definition),samplePlanetEnvironment(location,definition))
 assert.equal(samplePlanetEnvironment(location).temperatureC,24)
})
test('streaming eviction/return and session reload retain the exact environmental publication and physical sample', () => {
 const storage=new Storage(), s=open(storage), location=point('vesper',[1,.001,.002]), first=s.environmentAt(location)
 const stream=new RegionStream(), definition=s.bodyDefinition('vesper')
 stream.select(location,definition); stream.prefetch(); stream.sample(location)
 for(const direction of [[0,1,0],[0,0,1],[-1,0,0],[0,-1,0]] as const) { stream.select(point('vesper',direction),definition);stream.prefetch() }
 assert.ok(stream.stats.evicted>0)
 stream.select(location,definition);stream.prefetch(); assert.deepEqual(s.environmentAt(location),first)
 assert.equal(s.upgrade(12000,14),true)
 const reload=open(storage);assert.equal(reload.error,'');assert.equal(reload.snapshot.environmentPublication,ENVIRONMENT_PUBLICATION)
 assert.deepEqual(reload.environmentAt(location),first);assert.deepEqual(reload.environmentSummary('vesper'),s.environmentSummary('vesper'))
})
test('legacy absence binds v1 lazily; unknown publication and failed write preserve bytes/progress', () => {
 const storage=new Storage(), s=open(storage);s.upgrade(12000,14)
 const saved=JSON.parse(storage.raw!), old=structuredClone(saved);delete old.environmentPublication;storage.raw=JSON.stringify(old)
 const bytes=storage.raw,migrated=open(storage)
 assert.equal(migrated.error,'');assert.equal(storage.raw,bytes);assert.equal(migrated.snapshot.environmentPublication,ENVIRONMENT_PUBLICATION)
 assert.deepEqual(migrated.snapshot.inventory,s.snapshot.inventory);assert.deepEqual(migrated.snapshot.sites,s.snapshot.sites)
 storage.fail=true;assert.equal(migrated.upgrade(12000,14),false);assert.equal(storage.raw,bytes)
 storage.fail=false;const retry=open(storage);assert.equal(retry.upgrade(12000,14),true);assert.equal(JSON.parse(storage.raw!).environmentPublication,ENVIRONMENT_PUBLICATION)
 for(const publication of [null,'environment:v2','wrong',{},5]) {
  storage.raw=JSON.stringify({...saved,environmentPublication:publication});const invalid=storage.raw,bad=open(storage)
  assert.match(bad.error,/environmental publication/);bad.checkpoint();assert.equal(storage.raw,invalid)
 }
})
test('reject malformed locations, unknown bodies and changed catalog/publication instead of silently resampling', () => {
 const location=point('vesper',[1,0,0]), d=planetDefinition('vesper')
 for(const radialM of [[NaN,0,0],[Infinity,0,0],[0,0,0],[1,2],['1',0,0]]) assert.throws(()=>samplePlanetEnvironment({...location,radialM} as unknown as SurfaceLocation))
 assert.throws(()=>samplePlanetEnvironment({...location,bodyId:'unknown-atmospheric-name'}))
 assert.throws(()=>samplePlanetEnvironment(location,{...d,digest:'changed'}))
 assert.throws(()=>samplePlanetEnvironment(location,d,'unknown'))
 assert.throws(()=>bodyEnvironmentSummary('vesper',d,'unknown'))
})
