import { translate } from './coordinates.ts'
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { enterCockpit, leaveCockpit, setCockpitPower } from './cockpit.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { ResourceSession } from './resourceSession.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { point2, convertAnchor } from './topDownFrames.ts'
import { requestInteraction, createTopDownState, drawList, PLAYER_ID, stepTopDown } from './topDownRuntime.ts'
import { captureScout, decodeScout } from './scoutSave.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { startingScoutLocation } from './scoutLocation.ts'
import type { ScoutAction } from './scoutLocation.ts'
function setup() {
 let raw:string|null=null, fail=false
 const storage={getItem:()=>raw,setItem:(_:string,v:string)=>{if(fail)throw Error('quota');raw=v}}
 let world=createTopDownState('interior',true)
 world={...world,entities:world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout',point:point2(0,4.7)}}:e)}
 world=requestInteraction(world,'scout:entry')
 world={...world,entities:world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout',point:point2(0,-3.1)}}:e)}
 world=enterCockpit({...world,selectedId:'scout:chair'})
 const session=new ResourceSession(storage,world),navigation=new NavigationRuntime(SAMPLE_BODIES,world.scoutLocation)
 assert.equal(session.transfer({revision:0,from:{kind:'source',id:'source:landing:mineral:01'},to:{kind:'player'},resourceId:'mineral',grams:1000}),true)
 assert.equal(session.deposit('mineral',500,session.snapshot.revision),true)
 return {session,navigation,storage,action:(a:ScoutAction)=>session.transitionScout(a,session.snapshot.revision,session.world.cockpit),fail:()=>{fail=true},corrupt:(change:(s:any)=>void)=>{const s=JSON.parse(raw!);change(s);raw=JSON.stringify(s)}}
}
test('explicit flight preserves one pose and scanner; interior works and the exterior is unreachable',()=>{
 const {session:s,navigation:n,action,storage}=setup()
 n.advance(4);n.advance(2);n.scanner.select('vesper')
 const pose=n.clock.state.position,scan=n.scanner.snapshot(),camera=s.world.camera
 assert.equal(action({kind:'begin-takeoff'}),true);n.advance(4);n.advance(2)
 assert.equal(n.clock.state,s.world.scoutLocation.ship);assert.equal(n.clock.state.mode,'flying')
 assert.notDeepEqual(n.clock.state.position,pose);assert.equal(n.scanner.snapshot().lastTick,scan.lastTick+360);assert.equal(s.world.camera,camera)
 assert.throws(()=>convertAnchor({frame:'ship:scout',point:point2(0,0)},'planet:fixture',s.world.frames),/unavailable/)
 s.world=leaveCockpit(s.world)
 assert.ok(drawList(s.world).every(e=>e.entity.anchor.frame==='ship:scout'))
 s.world={...s.world,entities:s.world.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout',point:point2(0,4.89)}}:e)}
 s.world=stepTopDown(s.world,[Math.sin(.12),Math.cos(.12)])
 assert.equal(s.world.entities.find(e=>e.id===PLAYER_ID)!.anchor.point.offsetM[1],4.89);assert.match(s.world.feedback,/Airlock locked/)
 s.checkpoint()
 const restored=new ResourceSession(storage,createTopDownState())
 assert.equal(restored.error,'');assert.deepEqual(captureScout(restored.world.scoutLocation),captureScout(s.world.scoutLocation))
 assert.deepEqual(restored.snapshot.sources,s.snapshot.sources);assert.deepEqual(restored.snapshot.stores,s.snapshot.stores)
 assert.equal(restored.world.tick,n.clock.state.tick)
})
test('each maneuver round trips; validated landing restores stable exterior and resources',()=>{
 const {session:s,navigation:n,action,storage}=setup(),entities=s.world.entities,sources=s.snapshot.sources
 for(const a of [{kind:'begin-takeoff'},{kind:'finish-takeoff'},{kind:'begin-landing',siteId:STARTING_SITE.id,bodyId:'vesper'},{kind:'finish-landing'}] as const){
  if(a.kind==='finish-takeoff'){n.advance(4);n.advance(2);s.checkpoint()}
  else if(a.kind==='finish-landing') { while(s.world.scoutLocation.landing) n.advance(.1); s.advance({...s.world,tick:s.world.tick+1}); s.checkpoint() }
  else assert.equal(action(a),true,s.feedback)
  const restored=new ResourceSession(storage,createTopDownState())
  assert.equal(restored.error,'');assert.deepEqual(restored.world.scoutLocation.ship,n.clock.state);assert.deepEqual(restored.world.frames,s.world.frames)
 }
 assert.equal(s.world.scoutLocation.binding,STARTING_SITE.id);assert.deepEqual(s.world.entities.filter(e=>e.anchor.frame==='ship:scout'&&e.id!==PLAYER_ID),entities.filter(e=>e.anchor.frame==='ship:scout'&&e.id!==PLAYER_ID));assert.deepEqual(s.world.entities.filter(e=>e.anchor.frame==='planet:fixture'),entities.filter(e=>e.anchor.frame==='planet:fixture'));assert.equal(s.snapshot.sources,sources)
 assert.equal(s.world.player.mode,'standing');assert.equal(s.world.cockpit.view,'world')
 assert.ok(drawList(s.world).some(e=>e.entity.anchor.frame==='planet:fixture'))
})
test('unauthorized/stale actions, invalid sites and failed saves cannot publish transitions',()=>{
 const {session:s,navigation:n,action,fail}=setup(),before=captureScout(s.world.scoutLocation)
 assert.equal(s.transitionScout({kind:'begin-takeoff'},s.snapshot.revision,s.world.cockpit,'intruder'),false)
 assert.equal(s.transitionScout({kind:'begin-takeoff'},s.snapshot.revision+1,s.world.cockpit),false)
 s.world=setCockpitPower(s.world,false);assert.equal(action({kind:'begin-takeoff'}),false)
 s.world=setCockpitPower(s.world,true);s.world={...s.world,cockpit:{...s.world.cockpit,view:'radar'}}
 assert.deepEqual(captureScout(s.world.scoutLocation),before)
 assert.equal(action({kind:'begin-takeoff'}),true);n.advance(4);n.advance(2)
 assert.equal(action({kind:'begin-landing',siteId:'missing',bodyId:'vesper'}),false)
 const flight=captureScout(s.world.scoutLocation),world=s.world
 fail();assert.equal(action({kind:'begin-landing',siteId:STARTING_SITE.id,bodyId:'vesper'}),false)
 assert.equal(s.world,world);assert.deepEqual(captureScout(s.world.scoutLocation),flight)
})
test('inconsistent snapshots fail closed; legacy v8 migrates without fabricating flight',()=>{
 for(const corrupt of [(s:any)=>{s.scout.binding=null},(s:any)=>{s.scout.ship.position.cell[0]='NaN'},(s:any)=>{s.scout.scanner={lastTick:99,pulse:0,selected:null,contactIds:[]}},(s:any)=>{s.scout.maneuver='landing'}]){
  const {session,storage,corrupt:mutate}=setup();session.upgrade(10000,12);mutate(corrupt)
  const raw=storage.getItem(),restored=new ResourceSession(storage,createTopDownState())
  assert.match(restored.error,/safely/);restored.checkpoint();assert.equal(storage.getItem(),raw)
 }
 const {session,storage,corrupt}=setup();session.upgrade(10000,12);corrupt(s=>{s.version=8;delete s.sites;delete s.scout})
 const legacy=new ResourceSession(storage,createTopDownState())
 assert.equal(legacy.error,'');assert.equal(legacy.world.scoutLocation.binding,STARTING_SITE.id);assert.equal(legacy.snapshot.version,11)
})

test('physical contact never fabricates a site, and landing cannot teleport a distant ship',()=>{
 const {session:s,navigation:n,action}=setup()
 assert.equal(action({kind:'begin-takeoff'}),true);n.advance(4);n.advance(2)
 n.clock.state={...n.clock.state,position:translate(startingScoutLocation().ship.position,[0,-1,0]),velocityMps:[0,-1,0]}
 n.advance(1/60)
 assert.equal(n.clock.state.mode,'recovering');assert.equal(s.world.scoutLocation.binding,null)
 assert.ok(drawList(s.world).every(e=>e.entity.anchor.frame==='ship:scout'))
 n.advance(1);n.advance(1)
 assert.equal(n.clock.state.mode,'recovering')
 assert.equal(action({kind:'begin-landing',siteId:STARTING_SITE.id,bodyId:'vesper'}),false)
 n.clock.state={...n.clock.state,position:translate(n.clock.state.position,[1000,0,0])}
 const pose=n.clock.state.position
 assert.equal(action({kind:'finish-landing'}),false);assert.deepEqual(n.clock.state.position,pose)
})

test('a retired cockpit session stays unauthorized after the same owner re-enters',()=>{
 const {session:s}=setup(),oldSeat=s.world.cockpit,revision=s.snapshot.revision
 s.world=leaveCockpit(s.world)
 s.world=enterCockpit({...s.world,selectedId:'scout:chair'})
 assert.notEqual(s.world.cockpit,oldSeat)
 assert.equal(s.transitionScout({kind:'begin-takeoff'},revision,oldSeat),false)
 assert.equal(s.transitionScout({kind:'begin-takeoff'},revision,s.world.cockpit),true)
})

test('saved physical contact cannot teleport a ship from an inconsistent galactic pose',()=>{
 const {session:s,navigation:n,action}=setup()
 action({kind:'begin-takeoff'});n.advance(4);n.advance(2)
 n.clock.state={...n.clock.state,position:translate(startingScoutLocation().ship.position,[0,-1,0]),velocityMps:[0,-1,0]}
 n.advance(1/60)
 const save=captureScout(s.world.scoutLocation),json=(v:unknown)=>JSON.parse(JSON.stringify(v,(_,v)=>typeof v==='bigint'?String(v):v))
 assert.doesNotThrow(()=>decodeScout(json(save)))
 assert.throws(()=>decodeScout(json({...save,ship:{...save.ship,position:translate(save.ship.position,[1e6,1e6,0])}})),/contact pose/)
})
