import {test} from 'node:test'
import assert from 'node:assert/strict'
import { EquipmentDraft, equipmentActionDenial } from './equipmentUi.ts'
import { equipmentWarnings, GearNoticeTracker, equipmentHudReadout } from './equipmentHud.ts'
import { ResourceSession } from './resourceSession.ts'
import { createTopDownState, PLAYER_ID } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { freshEquipment } from './equipment.ts'
const make=()=>new ResourceSession({getItem:()=>null,setItem:()=>{}},createTopDownState('exterior',true),undefined,undefined,()=>0)
test('field UI explains ship-only rejection and review never changes custody',()=>{
 const s=make(),suit=s.snapshot.inventory.equipment!.instances[0]!,draft=new EquipmentDraft()
 const command={kind:'move' as const,id:suit.id,from:suit.location,to:{kind:'inventory' as const}}
 assert.match(equipmentActionDenial(s,command)!,/inside Scout/)
 const before=s.snapshot;draft.review(s,command);assert.equal(draft.pending,null);assert.equal(s.snapshot,before)
 s.world={...s.world,entities:s.world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout',point:point2(0,3.5)}}:e)}
 draft.review(s,command);assert.ok(draft.pending);assert.equal(s.snapshot,before)
})
test('exact warning thresholds are persistent levels, resolved equipment clears, notices expire on sole clock',()=>{
 const base=freshEquipment(),module=base.instances[1]!
 const state=(remaining:string,kind:'installed'|'destroyed'='installed')=>({...base,instances:[base.instances[0]!,{...module,remaining,location:kind==='destroyed'?{kind:'destroyed' as const}:module.location}]})
 assert.deepEqual(equipmentWarnings(state('200001/1000')),[])
 assert.match(equipmentWarnings(state('200/1'))[0]!,/20%/)
 assert.match(equipmentWarnings(state('50/1'))[0]!,/5%/)
 assert.deepEqual(equipmentWarnings(state('0/1','destroyed')),[])
 const tracker=new GearNoticeTracker();assert.deepEqual(tracker.update(base,0),[])
 assert.equal(tracker.update(state('0/1','destroyed'),1).length,1)
 assert.equal(tracker.update(state('0/1','destroyed'),300).length,1)
 assert.deepEqual(tracker.update(state('0/1','destroyed'),301),[])
 assert.deepEqual(tracker.update(state('0/1','destroyed'),302),[])
 const s=make(),t=new GearNoticeTracker(),first=equipmentHudReadout(s,t)
 s.advance({...s.world,tick:s.world.tick+1});assert.equal(equipmentHudReadout(s,t),first)
})
test('confirmation tolerates clock-only commits but rejects changed equipment and field crossing',()=>{
 let raw:string|null=null
 const w=createTopDownState('exterior',true),world={...w,entities:w.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout' as const,point:point2(.7,4.1)}}:e)}
 const s=new ResourceSession({getItem:()=>raw,setItem:(_:string,v:string)=>{raw=v}},world,undefined,undefined,()=>0)
 const suit=s.snapshot.inventory.equipment!.instances[0]!,command={kind:'move' as const,id:suit.id,from:suit.location,to:{kind:'inventory' as const}},draft=new EquipmentDraft()
 draft.review(s,command);s.advance({...s.world,tick:s.world.tick+1});s.checkpoint()
 assert.equal(draft.confirm(s),true);assert.equal(s.snapshot.inventory.equipment!.instances[0]!.location.kind,'inventory')
 const carried=s.snapshot.inventory.equipment!.instances[0]!
 const equip={kind:'move' as const,id:carried.id,from:carried.location,to:{kind:'equipped' as const}}
 draft.review(s,equip)
 assert.equal(s.changeEquipment({kind:'move',id:carried.id,from:carried.location,to:{kind:'drawer'}},s.snapshot.revision),true)
 const before=s.snapshot;assert.equal(draft.confirm(s),false);assert.equal(s.snapshot,before)
 const stored=s.snapshot.inventory.equipment!.instances[0]!
 draft.review(s,{kind:'move',id:stored.id,from:stored.location,to:{kind:'inventory'}})
 s.world={...s.world,entities:s.world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'planet:fixture',point:point2(0,10)}}:e)}
 assert.equal(draft.confirm(s),false);assert.equal(s.snapshot,before)
})
