import { test } from 'node:test'
import assert from 'node:assert/strict'
import { CELL_SIZE_M, ORIGIN, relativeM, translate } from './coordinates.ts'
import { NavigationClock, alignTo, ANGULAR_SPEED, idleControls, initialShip, setMass, setThrottle, stepShip } from './simulation.ts'
const near=(a:number,b:number,tolerance=1e-8)=>assert.ok(Math.abs(a-b)<tolerance,`${a} != ${b}`)
test('split cells preserve millimetres at galactic anchors and cross both boundaries',()=>{
 const anchor={cell:[10n**20n,-(10n**20n),0n] as const,offsetM:[0,0,0] as const}
 assert.deepEqual(relativeM(translate(anchor,[0.001,0,0]),anchor),[0.001,0,0])
 for(const x of [CELL_SIZE_M/2+10,-CELL_SIZE_M/2-10]) near(relativeM(translate(ORIGIN,[x,0,0]),ORIGIN)[0],x)
 assert.throws(()=>translate(ORIGIN,[NaN,0,0]))
})
test('fixed-step result is independent of publication/frame batching and keeps debt',()=>{
 const a=new NavigationClock(setThrottle(initialShip(),100)),b=new NavigationClock(a.state)
 for(let i=0;i<120;i++) a.advance(1/120,idleControls())
 for(let i=0;i<10;i++) b.advance(.1,idleControls())
 assert.deepEqual(a.state,b.state);assert.equal(a.state.tick,60)
 near(a.state.velocityMps[2],-1000);near(relativeM(a.state.position,ORIGIN)[2],-500)
 const c=new NavigationClock();c.advance(5,idleControls());c.advance(0,idleControls());assert.equal(c.state.tick,300)
 assert.throws(()=>c.advance(-1,idleControls()))
})
test('approved impulse uses current total mass, signed throttle, and independent velocity',()=>{
 for(const [mass,throttle,speed] of [[100000,100,-1000],[100000,-100,500],[200000,100,-500],[100000,50,-500]]) {
  const clock=new NavigationClock(setThrottle(setMass(initialShip(),mass),throttle));clock.advance(1,idleControls());near(clock.state.velocityMps[2],speed)
 }
 const state={...initialShip(),velocityMps:[12,34,56] as const}
 const rotated=stepShip(state,{...idleControls(),left:true});assert.deepEqual(rotated.velocityMps,state.velocityMps);near(rotated.yaw,ANGULAR_SPEED/60)
 assert.throws(()=>setMass(state,0));assert.throws(()=>setThrottle(state,NaN))
})
test('one-time alignment completes, preserves thrust, and cancels on any manual hold',()=>{
 const clock=new NavigationClock(alignTo(setThrottle(initialShip(),100),[1,0,-1]))
 for(let i=0;i<10;i++)clock.advance(1,idleControls())
 assert.equal(clock.state.alignment,null);near(clock.state.yaw,-Math.PI/4);assert.ok(clock.state.velocityMps[0]>0)
 const completed=clock.state.yaw;clock.advance(1,idleControls());near(clock.state.yaw,completed)
 const state=alignTo(initialShip(),[1,0,0]);const cancelled=stepShip(state,{up:true,down:true,left:false,right:false});assert.equal(cancelled.alignment,null)
})
