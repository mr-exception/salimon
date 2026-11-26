import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession, decodeResourceSave } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { benchSpentJ60 } from './gearBench.ts'
import * as q from './exact.ts'
class Storage {raw:string|null=null;fail=false;getItem(){return this.raw}setItem(_:string,v:string){if(this.fail)throw Error('quota');this.raw=v}}
const world=()=>{const w=createTopDownState('exterior',true);return {...w,entities:w.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout' as const,point:point2(.8,1.1)}}:e)}}
function setup(hp='0/1'){
 const storage=new Storage(),open=()=>new ResourceSession(storage,world(),undefined,undefined,()=>0),s=open()
 s.transfer({revision:0,from:{kind:'source',id:'source:landing:mineral:01'},to:{kind:'player'},resourceId:'mineral',grams:1000});s.depositBenchInput('mineral',1000,s.snapshot.revision)
 const seed=JSON.parse(storage.raw!);seed.inventory.equipment.instances[0].remaining=hp;seed.inventory.equipment.instances[0].location={kind:'inventory'};storage.raw=JSON.stringify(seed)
 return {storage,s:open(),open}
}
test('zero, partial and fractional repair restore only requested HP with exact final paid tick',()=>{
 for(const [initial,amount] of [['0/1','1000/1'],['997/1','3/1'],['1999/2','1/3'],['999/1','1/7'],['999/1','1/10000']]){
  const {s,open}=setup(initial),suit=s.snapshot.inventory.equipment!.instances[0]!,modules=s.snapshot.inventory.equipment!.instances.slice(1)
  assert.equal(s.startSuitRepair(suit.id,amount,s.snapshot.revision),true,s.feedback)
  assert.equal(s.snapshot.inventory.equipment!.instances[0]!.location.kind,'bench')
  const before=s.snapshot
  const module=modules[0]!
  assert.equal(s.changeEquipment({kind:'move',id:module.id,from:module.location,to:{kind:'inventory'}},s.snapshot.revision),false)
  assert.equal(s.changeEquipment({kind:'move',id:suit.id,from:{kind:'bench'},to:{kind:'equipped'}},s.snapshot.revision),false)
  assert.equal(s.snapshot,before)
  const hp=q.decode(amount),cost=Number((hp[0]*6000n+hp[1]-1n)/hp[1]),ticks=Math.ceil(cost/2000)
  s.advance({...s.world,tick:s.world.tick+ticks})
  assert.equal(s.error,'');assert.equal(s.snapshot.bench.tray?.repair?.suitId,suit.id)
  assert.equal(benchSpentJ60(s.snapshot.bench),cost)
  assert.equal(s.snapshot.inventory.equipment!.instances[0]!.remaining,q.encode(q.add(q.decode(initial),hp)))
  assert.deepEqual(s.snapshot.inventory.equipment!.instances.slice(1),modules)
  const restored=open();assert.equal(restored.error,'');assert.equal(restored.claimBench(1,restored.snapshot.revision),true,restored.feedback)
  assert.equal(restored.claimBench(1,restored.snapshot.revision),false)
  assert.equal(restored.snapshot.inventory.equipment!.instances[0]!.id,suit.id)
 }
})
test('invalid/equipped/full suit, paused custody, rollback, stale action and full claim preserve endpoints',()=>{
 const {s,storage,open}=setup('999/1'),id=s.snapshot.inventory.equipment!.instances[0]!.id
 for(const amount of ['0/1','-1/1','2/1','2/2','NaN'])assert.equal(s.startSuitRepair(id,amount,s.snapshot.revision),false)
 assert.equal(s.startSuitRepair('missing','1/1',s.snapshot.revision),false)
 assert.equal(s.startSuitRepair(id,'1/1',s.snapshot.revision-1),false)
 s.advance({...s.world,tick:1},{watts:s.snapshot.core.reserveJ60})
 assert.equal(s.startSuitRepair(id,'1/1',s.snapshot.revision),true)
 s.advance({...s.world,tick:2});assert.equal(s.snapshot.bench.job?.powerPaused,true)
 const paused=s.snapshot;storage.fail=true;s.advance({...s.world,tick:3});assert.equal(s.snapshot,paused)
 storage.fail=false;const r=open();assert.equal(r.error,'');assert.equal(r.snapshot.inventory.equipment!.instances[0]!.location.kind,'bench')
 r.transfer({revision:r.snapshot.revision,from:{kind:'source',id:'source:landing:biology:01'},to:{kind:'store',id:'scout:core:fuel'},resourceId:'biomass',grams:1})
 r.advance({...r.world,tick:r.world.tick+20});assert.ok(r.snapshot.bench.tray)
 r.transfer({revision:r.snapshot.revision,from:{kind:'source',id:'source:landing:mineral:01'},to:{kind:'player'},resourceId:'mineral',grams:8000})
 const before=r.snapshot;assert.equal(r.claimBench(1,r.snapshot.revision),false);assert.equal(r.snapshot,before)
 const forged=JSON.parse(storage.raw!);forged.inventory.equipment.instances[0].remaining='999/1';assert.throws(()=>decodeResourceSave(JSON.stringify(forged),world()))
 const full=setup('1000/1');assert.equal(full.s.startSuitRepair(id,'1/1',full.s.snapshot.revision),false)
 const eq=setup('999/1');eq.s.changeEquipment({kind:'move',id,from:{kind:'inventory'},to:{kind:'equipped'}},eq.s.snapshot.revision);assert.equal(eq.s.startSuitRepair(id,'1/1',eq.s.snapshot.revision),false)
})
test('terminal reset keeps bench custody and attached modules, then permits same-identity claim',()=>{
 const {s,storage,open}=setup('0/1'),id=s.snapshot.inventory.equipment!.instances[0]!.id
 assert.equal(s.startSuitRepair(id,'1/1',s.snapshot.revision),true)
 const seed=JSON.parse(storage.raw!);seed.survival.health=0;delete seed.survival.healthExact;storage.raw=JSON.stringify(seed)
 const r=open();assert.equal(r.freshRun(),true);assert.equal(r.error,'')
 r.advance({...r.world,tick:r.world.tick+3});assert.equal(r.error,'')
 assert.equal(r.claimBench(1,r.snapshot.revision),true,r.feedback)
 assert.equal(r.snapshot.inventory.equipment!.instances.find(g=>g.id===id)!.remaining,'1/1')
})

test('failed repair start, paid completion and claim leave suit, minerals and energy unchanged',()=>{
 const {s,storage,open}=setup('999/1'),id=s.snapshot.inventory.equipment!.instances[0]!.id
 const initial=s.snapshot;storage.fail=true
 assert.equal(s.startSuitRepair(id,'1/1',s.snapshot.revision),false);assert.equal(s.snapshot,initial)
 storage.fail=false;const r=open();r.startSuitRepair(id,'1/1',r.snapshot.revision);r.advance({...r.world,tick:r.world.tick+2})
 const before=r.snapshot,raw=storage.raw;storage.fail=true
 r.advance({...r.world,tick:r.world.tick+1});assert.equal(r.snapshot,before);assert.equal(storage.raw,raw)
 storage.fail=false;const t=open();t.advance({...t.world,tick:t.world.tick+1});const complete=t.snapshot;storage.fail=true
 assert.equal(t.claimBench(1,t.snapshot.revision),false);assert.equal(t.snapshot,complete)
})
