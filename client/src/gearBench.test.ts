import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession, decodeResourceSave } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { BENCH_INPUT_ID, GEAR_RECIPES, benchSpentJ60 } from './gearBench.ts'
import { FABRICATOR_RECIPE_ID } from './scoutFabricator.ts'
class Storage { raw:string|null=null;fail=false;getItem(){return this.raw}setItem(_:string,v:string){if(this.fail)throw Error('quota');this.raw=v} }
const world=()=>{const w=createTopDownState('exterior',true);return {...w,entities:w.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout' as const,point:point2(.8,1.1)}}:e)}}
const make=(storage=new Storage())=>new ResourceSession(storage,world(),undefined,undefined,()=>0)
function input(s:ResourceSession,mineral:number,biomass:number){for(const [id,n] of [['mineral',mineral],['biomass',biomass]] as const){if(!n)continue;assert.equal(s.transfer({revision:s.snapshot.revision,from:{kind:'source',id:id==='mineral'?'source:landing:mineral:01':'source:landing:biology:01'},to:{kind:'player'},resourceId:id,grams:n}),true);assert.equal(s.depositBenchInput(id,n,s.snapshot.revision),true,s.feedback)}}
test('all recipes reserve exactly, pay ticks, conserve declared loss and claim fresh unique gear once',()=>{
 for(const r of Object.values(GEAR_RECIPES)){
  const storage=new Storage(),s=make(storage);input(s,r.mineral,r.biomass)
  assert.equal(s.startBench(r.definitionId,s.snapshot.revision),true,s.feedback)
  assert.equal(s.withdrawBenchInput('mineral',1,s.snapshot.revision),false)
  const mass=s.payload().shipGrams;s.advance({...s.world,tick:s.world.tick+r.ticks})
  assert.equal(s.error,'');assert.equal(s.snapshot.bench.tray?.recipeId,r.definitionId)
  assert.equal(benchSpentJ60(s.snapshot.bench),r.ticks*2000)
  assert.ok(s.payload().shipGrams<=mass)
  assert.equal(s.startBench(r.definitionId,s.snapshot.revision),false)
  assert.equal(s.claimBench(1,s.snapshot.revision),true,s.feedback)
  assert.equal(s.claimBench(1,s.snapshot.revision),false)
  const gear=s.snapshot.inventory.equipment!.instances.at(-1)!
  assert.equal(gear.definitionId,r.definitionId);assert.equal(gear.remaining,'1000/1');assert.equal(gear.location.kind,'inventory')
  assert.equal(make(storage).error,'');assert.deepEqual(make(storage).snapshot.bench,s.snapshot.bench)
 }
})
test('simultaneous consumers convert first, prioritize Fabricator, pause/resume and never double spend',()=>{
 const storage=new Storage(),s=make(storage);input(s,500,0)
 assert.equal(s.transfer({revision:s.snapshot.revision,from:{kind:'source',id:'source:landing:mineral:01'},to:{kind:'player'},resourceId:'mineral',grams:1000}),true)
 assert.equal(s.depositFabricatorInput('mineral',1000,s.snapshot.revision),true)
 s.advance({...s.world,tick:1},{watts:s.snapshot.core.reserveJ60-6000})
 assert.equal(s.startFabricator(FABRICATOR_RECIPE_ID,s.snapshot.revision),true)
 assert.equal(s.startBench('module:radiation:t1',s.snapshot.revision),true)
 s.advance({...s.world,tick:2})
 assert.equal(s.snapshot.core.reserveJ60,1000);assert.equal(s.snapshot.bench.job?.remainingTicks,900);assert.equal(s.snapshot.bench.job?.powerPaused,true)
 assert.equal(s.snapshot.fabricator.job?.remainingTicks,1199)
 assert.equal(s.transfer({revision:s.snapshot.revision,from:{kind:'source',id:'source:landing:biology:01'},to:{kind:'store',id:'scout:core:fuel'},resourceId:'biomass',grams:1}),true)
 // Conversion accrues its budget until a whole gram can be converted; both jobs then resume.
 s.advance({...s.world,tick:14});assert.ok(s.snapshot.bench.job!.remainingTicks<900);assert.equal(s.error,'')
})
test('claim capacity/stale/reload/save rollback and legacy migration reject invented receipts',()=>{
 const storage=new Storage(),s=make(storage);input(s,500,0);s.startBench('module:radiation:t1',s.snapshot.revision)
 s.advance({...s.world,tick:900})
 assert.equal(s.transfer({revision:s.snapshot.revision,from:{kind:'source',id:'source:landing:mineral:01'},to:{kind:'player'},resourceId:'mineral',grams:10000}),true)
 const before=s.snapshot;assert.equal(s.claimBench(1,s.snapshot.revision),false);assert.equal(s.snapshot,before)
 assert.equal(s.claimBench(1,s.snapshot.revision-1),false)
 const raw=JSON.parse(storage.raw!);raw.bench.completed['module:radiation:t1']=2
 assert.throws(()=>decodeResourceSave(JSON.stringify(raw),world()))
 const t=make(new Storage());input(t,500,0);const snapshot=t.snapshot;(t as any).storage.fail=true
 assert.equal(t.startBench('module:radiation:t1',t.snapshot.revision),false);assert.equal(t.snapshot,snapshot)
 const legacy=JSON.parse(storage.raw!);legacy.version=18;delete legacy.bench;delete legacy.stores[BENCH_INPUT_ID]
 // A v18 save cannot inherit transformed bench material: conservation fails closed.
 assert.throws(()=>decodeResourceSave(JSON.stringify(legacy),world()))
 const clean=make(new Storage());clean.upgrade(10000,12);const old=JSON.parse((clean as any).storage.raw);old.version=18;delete old.bench;delete old.stores[BENCH_INPUT_ID]
 assert.equal(decodeResourceSave(JSON.stringify(old),world()).bench.started,0)
})
