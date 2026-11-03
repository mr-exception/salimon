import {test} from 'node:test'
import assert from 'node:assert/strict'
import { ORIGIN, relativeM, translate } from './coordinates.ts'
import { length, sub } from './math.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { distances, navigationWarnings, physicalRadiusM, resolveContacts, surfaceVelocity } from './contact.ts'
import { STEP_SECONDS, idleControls, initialShip, stepShip, type ShipState } from './simulation.ts'
import { type Body } from './sampleUniverse.ts'
const body:Body={id:'planet',name:'Planet',kind:'planet',radiusKm:1,position:ORIGIN}
const ship=(speed:number,kind=body):ShipState=>({...initialShip(),position:translate(ORIGIN,[physicalRadiusM(kind),0,0]),velocityMps:[-speed,0,0],throttle:0})
const collide=(state:ShipState,b=body)=>resolveContacts(state,stepShip(state,idleControls()),[b])
test('20 m/s safe surface boundary and 100 m/s destruction use total surface-relative speed',()=>{
 for(const [speed,outcome] of [[19.999,'safe-contact'],[20,'safe-contact'],[20.001,'crash'],[99.999,'crash'],[100,'destroyed'],[100.001,'destroyed']] as const) {
  const state=collide(ship(speed));assert.equal(state.contact?.outcome,outcome);assert.equal(state.throttle,0);assert.equal(state.alignment,null)
 }
 const tangent={...ship(1),velocityMps:[-1,100,0] as const};assert.equal(collide(tangent).mode,'destroyed')
})
test('departing/tangent touching states do not collide, and a symbol-only flyby is safe',()=>{
 for(const velocityMps of [[20,0,0],[0,100,0]] as const)assert.equal(collide({...ship(0),velocityMps}).contact,null)
 const before={...initialShip(),position:translate(ORIGIN,[-2000,physicalRadiusM(body)+1,0]),velocityMps:[240000,0,0] as const}
 assert.equal(collide(before).contact,null)
})
test('sweep catches complete crossing, picks earliest body independently of input order',()=>{
 const before={...initialShip(),position:translate(ORIGIN,[-2000,0,0]),velocityMps:[300000,0,0] as const}
 const later={...body,id:'later',position:translate(ORIGIN,[2000,0,0])}
 const result=resolveContacts(before,stepShip(before,idleControls()),[later,body]);assert.equal(result.mode,'destroyed');assert.equal(result.contact?.bodyId,body.id)
 // 1e-9 m tolerance covers normalization roundoff at a 1 km sphere.
 assert.ok(Math.abs(length(relativeM(result.position,body.position))-physicalRadiusM(body))<1e-9)
})
test('rotation changes impact-relative velocity and destruction decision',()=>{
 const rotating={...body,angularVelocityRadps:[0,0,.1] as const}
 const surface=surfaceVelocity(rotating,[1000,0,0]);assert.deepEqual(surface,[0,100,0])
 assert.equal(collide({...ship(1),velocityMps:[-1,0,0]},rotating).mode,'destroyed')
 assert.equal(collide({...ship(1),velocityMps:[-1,100,0]},rotating).contact?.outcome,'safe-contact')
})
test('low-speed nonplanetary contact rebounds then settles without becoming a landing',()=>{
 const asteroid={...body,kind:'asteroid' as const}
 const runtime=new NavigationRuntime([asteroid]);runtime.clock.state=ship(10,asteroid)
 runtime.advance(STEP_SECONDS);assert.equal(runtime.clock.state.contact?.outcome,'crash')
 const heights:number[]=[]
 for(let i=0;i<60;i++){runtime.advance(STEP_SECONDS);heights.push(distances(runtime.clock.state,asteroid).surfaceDistanceM)}
 assert.equal(runtime.clock.state.mode,'grounded');assert.ok(heights.some(h=>h>0));assert.ok(heights.every(h=>h>=-1e-8&&h<=2));assert.equal(runtime.clock.state.throttle,0)
 runtime.reset();assert.equal(runtime.clock.state.mode,'flying');assert.equal(runtime.scanner.contacts.length,0)
})
test('warnings are bounded/deduplicated observations, do not change thrust or motion',()=>{
 const state={...ship(150),position:translate(ORIGIN,[2000,0,0]),throttle:100}
 const before=structuredClone(state),warnings=navigationWarnings(state,[body,body]);assert.equal(warnings.length,1);assert.equal(warnings[0].severity,'danger');assert.deepEqual(state,before)
 assert.equal(navigationWarnings({...state,velocityMps:[150,0,0]},[body])[0].severity,'proximity')
 assert.equal(navigationWarnings(state,[]).length,0)
})
test('production fixed-step composition is deterministic through collision, scanner and restart',()=>{
 const a=new NavigationRuntime([body]),b=new NavigationRuntime([body]);a.clock.state=ship(50);b.clock.state=ship(50)
 for(let i=0;i<120;i++)a.advance(1/120)
 for(let i=0;i<10;i++)b.advance(.1)
 assert.deepEqual(a.clock.state,b.clock.state);assert.ok(length(sub(a.clock.state.velocityMps,b.clock.state.velocityMps))===0)
})

test('grounded rotation follows the surface, terminal state stops thrust, and restart clears time debt',()=>{
 const rotating={...body,angularVelocityRadps:[0,0,.001] as const}
 const runtime=new NavigationRuntime([rotating]);runtime.clock.state=ship(20,rotating)
 runtime.advance(2);assert.equal(runtime.clock.state.mode,'grounded');assert.ok(runtime.clock.state.position.offsetM[1]>0)
 const dead=new NavigationRuntime([body]);dead.clock.state=ship(100);dead.advance(STEP_SECONDS)
 const stopped=dead.clock.state.position;dead.advance(1,{...idleControls(),up:true});assert.deepEqual(dead.clock.state.position,stopped);assert.equal(dead.clock.state.throttle,0)
 dead.advance(10);dead.reset();dead.advance(0);assert.equal(dead.clock.state.tick,0)
})
