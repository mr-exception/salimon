import {test} from 'node:test'
import assert from 'node:assert/strict'
import { FrameTimer, NavigationInput, axisForCode, panAxisForCode } from './input.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
test('independent key and pointer owners release without clearing another held input',()=>{
 const input=new NavigationInput();input.press('key:KeyA','left');input.press('pointer:1','left');input.press('pointer:2','up')
 input.release('pointer:1');assert.equal(input.controls.left,true);input.release('key:KeyA');assert.equal(input.controls.left,false);assert.equal(input.controls.up,true)
 input.press('button:left:Space','left');input.releasePrefix('button:left:');assert.equal(input.controls.left,false)
 input.clear();assert.ok(Object.values(input.controls).every(v=>!v));assert.equal(axisForCode('KeyW'),undefined);assert.equal(panAxisForCode('KeyW'),'up');assert.equal(axisForCode('ArrowUp'),'up');assert.equal(axisForCode('Space'),undefined)
})
test('frame adapter rejects negative startup delta, pauses hidden time, and resumes without catchup',()=>{
 const timer=new FrameTimer(100),runtime=new NavigationRuntime([])
 assert.equal(timer.sample(99,false),0)
 runtime.advance(timer.sample(1099,false));assert.equal(runtime.clock.state.tick,60)
 runtime.advance(timer.sample(100000,true));assert.equal(runtime.clock.state.tick,60)
 timer.resume(200000);runtime.advance(timer.sample(200100,false));assert.equal(runtime.clock.state.tick,66)
})
