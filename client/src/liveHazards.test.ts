import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ResourceSession } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { freshHazards, stepHazards, hazardRates } from './hazards.ts'
import { freshSurvival, advanceSurvival } from './survival.ts'
const move = (s: ResourceSession, frame: 'planet:fixture' | 'ship:scout', x: number, y = 0) => ({ ...s.world, entities: s.world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame, point: point2(x,y) } } : e) })
test('unequal modules share intensity proportionally, split depletion and ignore identity order', () => {
 const modules = [{ id:'a',type:'temperature' as const,protection:2,remaining:'1/300' },{id:'b',type:'temperature' as const,protection:5,remaining:'1000/1'}]
 const rates=hazardRates([{id:'temp',type:'temperature',temperatureC:59}])
 const a=stepHazards({...freshHazards(),modules},rates,freshSurvival())
 assert.equal(a.state.modules[0]!.remaining,'0/1')
 assert.equal(a.state.modules[1]!.remaining,'199989/200')
 assert.equal(a.events[0]!.damage,'0/1'); assert.deepEqual(a.events[0]!.destroyed,['a'])
 const b=stepHazards({...freshHazards(),modules:modules.map(m=>({...m,id:m.id==='a'?'z':'a'})).reverse()},rates,freshSurvival())
 assert.deepEqual(b.state.modules.map(m=>m.remaining).reverse(),a.state.modules.map(m=>m.remaining))
})
test('actual physical safe crossing, live wear, zero elapsed, reload and rollback', () => {
 let raw: string|null=null, fail=false
 const storage={getItem:()=>raw,setItem:(_:string,v:string)=>{if(fail)throw Error('quota');raw=v}}
 const s=new ResourceSession(storage,createTopDownState('exterior',true),undefined,undefined,()=>0)
 assert.equal(s.currentEnvironment(move(s,'planet:fixture',128)).temperatureC,24)
 const outside=move(s,'planet:fixture',129)
 assert.notEqual(s.currentEnvironment(outside).radiationHpPerSecond,0)
 s.advance({...outside,tick:s.world.tick+1})
 assert.equal(s.error,''); assert.ok(s.survival.health<200)
 assert.notEqual(s.snapshot.inventory.equipment!.instances[1]!.remaining,'1000/1')
 const health=s.survival.health, worn=s.snapshot.inventory.equipment
 s.advance({...move(s,'ship:scout',0,3),tick:s.world.tick+1})
 assert.equal(s.survival.health,health);assert.deepEqual(s.snapshot.inventory.equipment,worn)
 const before=raw;s.advance(s.world);assert.equal(raw,before)
 const r=new ResourceSession(storage,createTopDownState('exterior',true),undefined,undefined,()=>0)
 assert.equal(r.error,'');assert.deepEqual(r.snapshot.inventory.equipment,worn)
 const snapshot=r.snapshot, world=r.world;fail=true
 r.advance({...move(r,'planet:fixture',129),tick:r.world.tick+1})
 assert.match(r.error,/Saving failed/);assert.equal(r.snapshot,snapshot);assert.equal(r.world,world);assert.deepEqual(r.hazardEvents,[])
})
test('damage suppresses due regeneration and resets recovery; safe recovery resumes',()=>{
 const hazards=freshHazards(), rates=hazardRates([{id:'heat',type:'temperature',temperatureC:34}])
 const before={...freshSurvival(),health:190,recoveryTicks:3599}
 const next=advanceSurvival(before,1,s=>stepHazards(hazards,rates,s).events.some(e=>e.damage!=='0/1'))
 assert.equal(next.healthExact,'11399/60');assert.equal(next.recoveryTicks,0)
 assert.equal(advanceSurvival(next,3600).healthExact,'11459/60')
})
test('live exhaustion atomically removes module from slot; inactive suit preserves intact modules',()=>{
 let raw:string|null=null
 const storage={getItem:()=>raw,setItem:(_:string,v:string)=>{raw=v}}
 const s=new ResourceSession(storage,createTopDownState('exterior',true),undefined,undefined,()=>0)
 s.upgrade(10000,12)
 const seed=JSON.parse(raw!);seed.inventory.equipment.instances[1].remaining='1/1000';raw=JSON.stringify(seed)
 const r=new ResourceSession(storage,createTopDownState('exterior',true),undefined,undefined,()=>0)
 r.advance({...move(r,'planet:fixture',129),tick:r.world.tick+1})
 assert.equal(r.error,'');assert.equal(r.snapshot.inventory.equipment!.instances[1]!.location.kind,'destroyed')
 assert.equal(r.hazardEvents[0]!.destroyed.length,1)
 const dead=JSON.parse(raw!);dead.inventory.equipment.instances[0].remaining='0/1';dead.inventory.equipment.instances[1]={...seed.inventory.equipment.instances[1],remaining:'1000/1'};raw=JSON.stringify(dead)
 const inactive=new ResourceSession(storage,createTopDownState('exterior',true),undefined,undefined,()=>0)
 inactive.advance({...move(inactive,'planet:fixture',129),tick:inactive.world.tick+1})
 assert.equal(inactive.snapshot.inventory.equipment!.instances[1]!.remaining,'1000/1')
})
