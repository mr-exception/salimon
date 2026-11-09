import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { startingScoutLocation } from './scoutLocation.ts'
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { enterCockpit, leaveCockpit } from './cockpit.ts'
import { translate } from './coordinates.ts'
import { ResourceSession, decodeResourceSave } from './resourceSession.ts'
import { BOARDING_DOOR_ID } from './scoutDoor.ts'
import { takeoffDenial, transitionScout } from './scoutLocation.ts'
import { captureScout } from './scoutSave.ts'
import { convertAnchor, point2 } from './topDownFrames.ts'
import { createTopDownState, PLAYER_ID, requestInteraction, stepTopDown, type TopDownState } from './topDownRuntime.ts'
import { nearbyObject } from './topDownInteraction.ts'
import { crossScoutEntry } from './topDownTraversal.ts'

function place(world: TopDownState, x: number, y: number, outside = false): TopDownState {
 const anchor = { frame: 'ship:scout' as const, point: point2(x,y) }
 return { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: outside ? convertAnchor(anchor,'planet:fixture',world.frames) : anchor } : e) }
}
function setup() {
 let raw: string | null = null, fail = false
 const storage = { getItem: () => raw, setItem: (_: string, value: string) => { if (fail) throw Error('quota'); raw = value } }
 const session = new ResourceSession(storage,place(createTopDownState('interior',true),0,4.7),undefined,undefined,()=>0)
 const action = () => session.transitionScout({kind:'begin-takeoff'},session.snapshot.revision,session.world.cockpit)
 const seat = () => { session.world=enterCockpit({...place(session.world,0,-3.1),selectedId:'scout:chair'}) }
 return { session, storage, action, seat, fail:()=>{fail=true} }
}
test('door interaction, collision on both sides, threshold safety and persistence use one state',()=>{
 const {session:s,storage}=setup()
 assert.equal(s.world.scoutLocation.boardingDoor,'open')
 s.world=place(s.world,0,5.5)
 const before=s.snapshot
 assert.equal(s.toggleBoardingDoor(before.revision),false);assert.match(s.feedback,/occupied/);assert.equal(s.snapshot,before)
 s.world=place(s.world,0,4.7)
 assert.equal(s.toggleBoardingDoor(s.snapshot.revision),true)
 assert.equal(s.world.scoutLocation.boardingDoor,'closed')
 assert.equal(s.world.entities.find(e=>e.id===BOARDING_DOOR_ID)!.appearance,'door')
 for(const outside of [false,true]) {
  let world=place(s.world,0,outside?6.3:4.7,outside),start=world.entities.find(e=>e.id===PLAYER_ID)!.anchor.frame
  for(let i=0;i<90;i++)world=stepTopDown(world,outside?[-Math.sin(.12),-Math.cos(.12)]:[Math.sin(.12),Math.cos(.12)])
  const p=convertAnchor(world.entities.find(e=>e.id===PLAYER_ID)!.anchor,'ship:scout',world.frames)
  assert.equal(world.entities.find(e=>e.id===PLAYER_ID)!.anchor.frame,start)
  assert.ok(outside?p.point.offsetM[1]>=6.11-1e-6:p.point.offsetM[1]<=4.89+1e-6)
 }
 const restored=new ResourceSession(storage,createTopDownState(),undefined,undefined,()=>0)
 assert.equal(restored.error,'');assert.equal(restored.world.scoutLocation.boardingDoor,'closed')
 assert.ok(restored.world.entities.find(e=>e.id===BOARDING_DOOR_ID)!.footprint)
 assert.equal(restored.toggleBoardingDoor(restored.snapshot.revision),true)
 assert.equal(restored.world.entities.find(e=>e.id===BOARDING_DOOR_ID)!.footprint,undefined)
 // An open door really transfers ownership; even a direct long crossing cannot bypass closed state.
 const a={frame:'ship:scout' as const,point:point2(0,4.7)},b={...a,point:point2(0,6.5)}
 assert.equal(crossScoutEntry(a,b,s.world.frames,false),a)
 assert.equal(crossScoutEntry(a,b,s.world.frames,true).frame,'planet:fixture')
})
test('reachable inspection fixture operates door; unreachable and seated use is denied',()=>{
 let world=place(createTopDownState('interior',true),0,4.7)
 world=requestInteraction(world,BOARDING_DOOR_ID);assert.equal(world.scoutLocation.boardingDoor,'closed')
 world=requestInteraction(place(world,0,2),BOARDING_DOOR_ID);assert.match(world.feedback,/Approach/);assert.equal(world.scoutLocation.boardingDoor,'closed')
})
test('launch admission explains each denial without advancing clocks, resources, pose or save',()=>{
 const cases: [string,(s:ResourceSession)=>void,RegExp][] = [
  ['open door',s=>{s.world.scoutLocation.boardingDoor='open'},/Close the aft/],
  ['exterior player',s=>{s.world=place(s.world,0,7,true)},/All occupants must board/],
  ['exterior crew',s=>{s.world={...s.world,entities:[...s.world.entities,{id:'crew:other',layer:'people',appearance:'player',anchor:{frame:'planet:fixture',point:point2(20,20)}}]}},/All occupants must board/],
  ['unpowered',s=>{s.world={...s.world,cockpit:{...s.world.cockpit,powered:false}}},/no power/],
  ['unauthorized',s=>{s.world={...s.world,cockpit:{...s.world.cockpit,ownerId:'other'}}},/authorized owner/],
  ['disabled',s=>{s.world={...s.world,cockpit:{...s.world.cockpit,enabled:false}}},/authorized owner/],
  ['unseated',s=>{s.world=leaveCockpit(s.world)},/authorized owner/],
  ['unbound site',s=>{s.world.scoutLocation.binding=null},/validated supported/],
  ['invalid mass',s=>{s.world.scoutLocation.ship={...s.world.scoutLocation.ship,massKg:NaN}},/state is invalid/],
  ['bad body',s=>{s.world.scoutLocation.body={...s.world.scoutLocation.body,id:'other'}},/validated supported/],
  ['bad pose',s=>{s.world.scoutLocation.ship={...s.world.scoutLocation.ship,position:translate(s.world.scoutLocation.ship.position,[100,0,0])}},/validated supported/],
  ['bad frame',s=>{s.world.scoutLocation.frames={...s.world.frames,ship:{...s.world.frames.ship,parent:null}}},/validated supported/],
  ['ongoing maneuver',s=>{s.world.scoutLocation.maneuver='takeoff'},/already running/],
  ['flying',s=>{s.world.scoutLocation.ship={...s.world.scoutLocation.ship,mode:'flying'}},/must be grounded/],
  ['destroyed',s=>{s.world.scoutLocation.ship={...s.world.scoutLocation.ship,mode:'destroyed'}},/destroyed/],
  ['conflicting alignment',s=>{s.world.scoutLocation.ship={...s.world.scoutLocation.ship,alignment:{yaw:0,pitch:0}}},/Stop the active/],
  ['conflicting gathering',s=>{s.activeSourceId='source:landing:mineral:01'},/Stop gathering/],
  ['terminal player',s=>{s.survival={...s.survival,health:0}},/survival condition/],
 ]
 for(const [label,change,reason] of cases){
  const {session:s,action,seat,storage}=setup();s.toggleBoardingDoor(s.snapshot.revision);seat();change(s)
  const before=structuredClone(captureScout(s.world.scoutLocation)),snapshot=s.snapshot,raw=storage.getItem(),ticks=s.world.tick,survival=structuredClone(s.survival),active=s.activeSourceId
  assert.equal(action(),false,label);assert.match(s.feedback,reason,label)
  assert.deepEqual(captureScout(s.world.scoutLocation),before,label);assert.equal(s.snapshot,snapshot,label);assert.equal(storage.getItem(),raw,label)
  assert.equal(s.world.tick,ticks,label);assert.deepEqual(s.survival,survival,label);assert.equal(s.activeSourceId,active,label)
 }
})
test('door and launch commands roll back on storage failure or stale revision',()=>{
 const {session:s,fail,storage}=setup(),world=s.world,location=s.world.scoutLocation
 assert.equal(s.toggleBoardingDoor(s.snapshot.revision+1),false);assert.match(s.feedback,/revision/)
 fail();assert.equal(s.toggleBoardingDoor(s.snapshot.revision),false)
 assert.equal(s.world,world);assert.equal(s.world.scoutLocation,location);assert.equal(location.boardingDoor,'open');assert.equal(storage.getItem(),null)
})
test('board, close, pilot, fly and land never auto-opens or auto-closes the door',()=>{
 const {session:s,action,seat}=setup()
 seat();assert.equal(action(),false);assert.match(s.feedback,/Close the aft/)
 s.world=place(leaveCockpit(s.world),0,4.7);assert.equal(s.toggleBoardingDoor(s.snapshot.revision),true)
 seat();assert.equal(takeoffDenial(s.world.scoutLocation),null);assert.equal(action(),true)
 assert.equal(action(),false);assert.match(s.feedback,/already running/)
 const command=(kind:'finish-takeoff'|'begin-landing'|'finish-landing')=>s.transitionScout(kind==='begin-landing'?{kind,siteId:s.world.scoutLocation.siteId,bodyId:'vesper'}:{kind},s.snapshot.revision,s.world.cockpit)
 for(const kind of ['taking-off','flying','landing'] as const){
  assert.equal(s.world.scoutLocation.ship.mode,kind)
  const seatState=s.world;s.world=place(leaveCockpit(s.world),0,4.7)
  const snapshot=s.snapshot
  assert.equal(s.toggleBoardingDoor(s.snapshot.revision),false);assert.match(s.feedback,/Airlock locked/);assert.equal(s.snapshot,snapshot)
  assert.equal(s.world.scoutLocation.boardingDoor,'closed')
  s.world=seatState
  if(kind==='taking-off'){const n=new NavigationRuntime(SAMPLE_BODIES,s.world.scoutLocation);n.advance(4);n.advance(2)}
  if(kind==='flying')assert.equal(command('begin-landing'),true)
 }
 s.world.scoutLocation.ship={...s.world.scoutLocation.ship,position:startingScoutLocation().ship.position}
 assert.equal(command('finish-landing'),true);assert.equal(s.world.scoutLocation.boardingDoor,'closed')
 s.world=place(leaveCockpit(s.world),0,4.7);assert.equal(s.toggleBoardingDoor(s.snapshot.revision),true)
})
test('v9 grounded/airborne migration is explicit; malformed v10 door and embedded player reject',()=>{
 for(const airborne of [false,true]) {
  const {session:s,storage,seat,action}=setup();s.toggleBoardingDoor(s.snapshot.revision)
  if(airborne){seat();action()}
  const data=JSON.parse(storage.getItem()!);data.version=9;delete data.scout.boardingDoor
  const migrated=decodeResourceSave(JSON.stringify(data),createTopDownState())
  assert.equal(migrated.version,10);assert.equal(migrated.scout.boardingDoor,airborne?'closed':'open')
 }
 const {session:s,storage}=setup();s.toggleBoardingDoor(s.snapshot.revision)
 const raw=JSON.parse(storage.getItem()!)
 for(const bad of [undefined,null,'ajar',true]){
  const data=structuredClone(raw);data.scout.boardingDoor=bad
  assert.throws(()=>decodeResourceSave(JSON.stringify(data),createTopDownState()),/boarding door|Scout snapshot/i)
 }
 const embedded=structuredClone(raw);embedded.location.anchor.point.offsetM=[0,5.5,0]
 assert.throws(()=>decodeResourceSave(JSON.stringify(embedded),createTopDownState()),/Unsafe saved location/)
 const {session:flight,seat,action,storage:flightStorage}=setup();flight.toggleBoardingDoor(0);seat();action()
 const openFlight=JSON.parse(flightStorage.getItem()!);openFlight.scout.boardingDoor='open'
 assert.throws(()=>decodeResourceSave(JSON.stringify(openFlight),createTopDownState()),/Open airborne boarding door/)
 const forgedLegacy=structuredClone(raw);forgedLegacy.version=9
 assert.throws(()=>decodeResourceSave(JSON.stringify(forgedLegacy),createTopDownState()),/Invalid Scout snapshot/)
 assert.throws(()=>transitionScout({...flight.world.scoutLocation,boardingDoor:'open',ship:{...flight.world.scoutLocation.ship,mode:'grounded'}},{kind:'begin-takeoff'}))
})

test('open-door operation preserves the existing nearby cargo container action',()=>{
 const world=place(createTopDownState('interior',true),.5,4.25)
 assert.equal(nearbyObject(world.entities.find(e=>e.id===PLAYER_ID)!.anchor,world.entities,world.frames)?.id,'scout:storage')
 const atDoor=place(world,0,4.7)
 assert.equal(nearbyObject(atDoor.entities.find(e=>e.id===PLAYER_ID)!.anchor,atDoor.entities,atDoor.frames)?.id,BOARDING_DOOR_ID)
})
