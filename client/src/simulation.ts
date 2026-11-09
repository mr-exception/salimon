import { type ContactState } from './contact.ts'
import { ORIGIN, translate, type Position } from './coordinates.ts'
import { add, length, normal, scale, type Vec3 } from './math.ts'
export type Controls = { up: boolean; down: boolean; left: boolean; right: boolean }
export const idleControls = (): Controls => ({up:false,down:false,left:false,right:false})
export type Alignment = { readonly yaw: number; readonly pitch: number }
export interface ShipState {
  readonly position: Position
  readonly velocityMps: Vec3
  readonly yaw: number
  readonly pitch: number
  readonly massKg: number
  readonly throttle: number
  readonly alignment: Alignment | null
  readonly mode: 'flying'|'taking-off'|'landing'|'recovering'|'grounded'|'destroyed'
  readonly contact: ContactState|null
  readonly tick: number
}
export const STEP_SECONDS = 1/60
export const ANGULAR_SPEED = 5 * Math.PI/180
export const DRY_MASS_KG = 100_000
export const initialShip = (): ShipState => ({position:ORIGIN,velocityMps:[0,0,0],yaw:0,pitch:0,massKg:DRY_MASS_KG,throttle:0,alignment:null,mode:'flying',contact:null,tick:0})
export const forward = (yaw: number, pitch: number): Vec3 => [-Math.sin(yaw)*Math.cos(pitch), Math.sin(pitch), -Math.cos(yaw)*Math.cos(pitch)]
export const angleDifference = (target: number, current: number) => Math.atan2(Math.sin(target-current),Math.cos(target-current))
function moveAngle(current: number, target: number) {
  const delta = angleDifference(target,current)
  return current + Math.max(-ANGULAR_SPEED*STEP_SECONDS,Math.min(ANGULAR_SPEED*STEP_SECONDS,delta))
}
export function setThrottle(state: ShipState, percent: number): ShipState {
  if (!Number.isFinite(percent)) throw new RangeError('Throttle must be finite')
  if(state.mode!=='flying')return {...state,throttle:0}
  return {...state,throttle:Math.max(-100,Math.min(100,Math.round(percent)))}
}
export function setMass(state: ShipState, massKg: number): ShipState {
  if (!Number.isFinite(massKg) || massKg < DRY_MASS_KG) throw new RangeError('Mass must include the dry hull')
  return {...state,massKg}
}
export function alignTo(state: ShipState, direction: Vec3): ShipState {
  if(state.mode!=='flying')return state
  if (!direction.every(Number.isFinite) || length(direction)===0) return {...state,alignment:null}
  const dir=normal(direction)
  return {...state,alignment:{yaw:Math.atan2(-dir[0],-dir[2]),pitch:Math.asin(Math.max(-1,Math.min(1,dir[1])))}}
}
export function stepShip(state: ShipState, controls: Controls): ShipState {
  if(state.mode!=='flying')return {...state,tick:state.tick+1,throttle:0,alignment:null}
  let {yaw,pitch,alignment}=state
  if (Object.values(controls).some(Boolean)) {
    alignment=null
    yaw+=(Number(controls.left)-Number(controls.right))*ANGULAR_SPEED*STEP_SECONDS
    pitch=Math.max(-Math.PI/2,Math.min(Math.PI/2,pitch+(Number(controls.up)-Number(controls.down))*ANGULAR_SPEED*STEP_SECONDS))
  } else if (alignment) {
    yaw=moveAngle(yaw,alignment.yaw)
    pitch=moveAngle(pitch,alignment.pitch)
    if(Math.abs(angleDifference(alignment.yaw,yaw))<1e-12 && Math.abs(alignment.pitch-pitch)<1e-12) alignment=null
  }
  // Fictional approved law: 1 N·s/J, 100/50 MW, infinite prototype energy.
  const forceN=(state.throttle>=0?100e6:50e6)*state.throttle/100
  const acceleration=scale(forward(yaw,pitch),forceN/state.massKg)
  const displacement=add(scale(state.velocityMps,STEP_SECONDS),scale(acceleration,STEP_SECONDS**2/2))
  return {...state,yaw,pitch,alignment,velocityMps:add(state.velocityMps,scale(acceleration,STEP_SECONDS)),position:translate(state.position,displacement),tick:state.tick+1}
}
// Owns time outside React and Three. Frame batching changes publication cadence only.
export class NavigationClock {
  state: ShipState
  private debt=0
  constructor(state=initialShip()) {this.state=state}
  reset(state=initialShip()) {this.state=state;this.debt=0}
  advance(elapsedSeconds: number, controls: Controls, afterStep?: (before: ShipState, after: ShipState)=>ShipState) {
    if(!Number.isFinite(elapsedSeconds)||elapsedSeconds<0) throw new RangeError('Invalid elapsed time')
    this.debt+=elapsedSeconds
    let steps=0
    while(this.debt+1e-12>=STEP_SECONDS && steps<240) {
      const before=this.state
      const after=stepShip(before,controls)
      this.state=afterStep?afterStep(before,after):after
      this.debt=Math.max(0,this.debt-STEP_SECONDS)
      steps++
    }
    return this.state
  }
}
