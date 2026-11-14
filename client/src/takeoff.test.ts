import { test } from 'node:test'
import assert from 'node:assert/strict'
import { startingScoutLocation, transitionScout } from './scoutLocation.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { distances, navigationWarnings } from './contact.ts'
import { relativeM, translate } from './coordinates.ts'
import { alignTo, setThrottle, idleControls } from './simulation.ts'
import { captureScout, decodeScout } from './scoutSave.ts'
import { planTakeoff, sampleTakeoff, TAKEOFF_PEAK_ACCELERATION } from './takeoff.ts'
const serialize=(x:unknown)=>JSON.parse(JSON.stringify(x,(_,v)=>typeof v==='bigint'?String(v):v))
function launch(mass=100000) {
 const l=startingScoutLocation();l.boardingDoor='closed';l.ship={...l.ship,massKg:mass}
 Object.assign(l,transitionScout(l,{kind:'begin-takeoff'}))
 return {l,n:new NavigationRuntime(SAMPLE_BODIES,l)}
}
test('bounded continuous lift, locked authority, scanner and automatic handoff',()=>{
 const {l,n}=launch(),anchor=startingScoutLocation().ship
 let height=-.0001,velocity=anchor.velocityMps
 for(let i=0;i<360;i++) {
  assert.throws(()=>transitionScout(l,{kind:'finish-takeoff'}),/automatically/)
  assert.throws(()=>transitionScout(l,{kind:'begin-takeoff'}),/already running/)
  l.ship=alignTo(setThrottle(l.ship,100),[1,0,0]);n.advance(1/60,{...idleControls(),up:true,left:true})
  const h=distances(l.ship,l.body).surfaceDistanceM
  assert.ok(h>=height-1e-4 && h>=-1e-4);height=h
  assert.ok(Math.hypot(...l.ship.velocityMps.map((v,j)=>(v-velocity[j])*60))<=TAKEOFF_PEAK_ACCELERATION+1e-8)
  velocity=l.ship.velocityMps;assert.equal(l.ship.yaw,anchor.yaw);assert.equal(l.ship.pitch,anchor.pitch)
 }
 assert.ok(Math.abs(height-50)<.0001);assert.equal(l.ship.mode,'flying');assert.equal(l.launch,null);assert.equal(l.maneuver,null);assert.equal(l.ship.throttle,0)
 assert.ok(Math.abs(Math.hypot(...velocity)-100/6)<1e-9);assert.equal(l.scanner?.lastTick,360);assert.ok(navigationWarnings(l.ship,[l.body]).length)
 const pose=l.ship.position;n.advance(1/60);assert.ok(Math.abs(Math.hypot(...relativeM(l.ship.position,pose))-100/360)<.0001)
})
test('partition equivalence and durable mid-launch resume',()=>{
 const a=launch(),b=launch()
 for(let i=0;i<720;i++)a.n.advance(1/120)
 for(let i=0;i<60;i++)b.n.advance(.1)
 assert.deepEqual(a.l,b.l)
 const c=launch();c.n.advance(3)
 const restored=decodeScout(serialize(captureScout(c.l)))
 new NavigationRuntime(SAMPLE_BODIES,restored).advance(3);assert.deepEqual(restored.ship,a.l.ship)
 const forged=serialize(captureScout(c.l));forged.launch.startTick++;assert.throws(()=>decodeScout(forged),/trajectory/)
})
test('loaded mass, inherited velocity, nonfinite and unsafe path admission',()=>{
 const light=launch(),loaded=launch(200000);light.n.advance(1);loaded.n.advance(1);assert.deepEqual(light.l.ship.position,loaded.l.ship.position)
 for(const mass of [1e8,Infinity,NaN]) {
  const l=startingScoutLocation();l.boardingDoor='closed';l.ship={...l.ship,massKg:mass};const before=structuredClone(l)
  assert.throws(()=>transitionScout(l,{kind:'begin-takeoff'}));assert.deepEqual(l,before)
 }
 const anchor=startingScoutLocation(),obstacle={id:'obstacle',name:'obstacle',kind:'asteroid' as const,radiusKm:.001,position:translate(anchor.ship.position,[0,25,0])}
 assert.throws(()=>planTakeoff(anchor.ship,anchor.body,[...SAMPLE_BODIES,obstacle]),/intersects/)
 const inherited={...anchor.ship,velocityMps:[2,0,0] as const},plan=planTakeoff(inherited,anchor.body,[anchor.body]),end=sampleTakeoff(inherited,anchor.body,plan,360)
 assert.ok(end.velocityMps[0]>1.9);assert.ok(Math.hypot(...relativeM(end.position,anchor.ship.position))>50)
})

test('world-owned held controls steer after launch, release without zeroing selected thrust',()=>{
 const {l,n}=launch();n.advance(4);n.advance(2)
 n.clock.state=setThrottle(n.clock.state,1)
 const before=n.clock.state
 n.pilotInput.press('ArrowLeft','left')
 n.advance(1,n.pilotInput.controls)
 assert.equal(l.ship.tick,before.tick+60)
 assert.ok(Math.abs(l.ship.yaw-before.yaw-5*Math.PI/180)<1e-12)
 assert.equal(l.ship.throttle,1)
 n.pilotInput.clear();const released=l.ship
 n.advance(1,n.pilotInput.controls)
 assert.equal(l.ship.yaw,released.yaw);assert.equal(l.ship.throttle,1)
 assert.notDeepEqual(l.ship.velocityMps,released.velocityMps)
 assert.equal(l.scanner?.lastTick,l.ship.tick)
})
