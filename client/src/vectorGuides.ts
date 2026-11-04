import * as THREE from 'three'
import { forward, type ShipState } from './simulation.ts'
import { length, type Vec3 } from './math.ts'
export type Point2=readonly [number,number]
export interface DirectionLine {readonly start:Point2;readonly end:Point2}
export const STATIONARY_SPEED_MPS=1e-6
/** Intersect an infinite forward ray with [-1,1]². A panned origin can enter
 * and exit; backward intersections, tangencies and view-axis vectors vanish. */
export function clipViewportRay(origin:Point2,direction:Point2):DirectionLine|null {
 if(![...origin,...direction].every(Number.isFinite))return null
 const magnitude=Math.hypot(...direction)
 if(magnitude<1e-12)return null
 const unit=direction.map(v=>v/magnitude)
 let enter=0,leave=Infinity
 for(let axis=0;axis<2;axis++) {
  const p=origin[axis],d=unit[axis]
  if(Math.abs(d)<1e-14){if(p < -1 || p > 1)return null;continue}
  const a=(-1-p)/d,b=(1-p)/d
  enter=Math.max(enter,Math.min(a,b));leave=Math.min(leave,Math.max(a,b))
  if(enter>=leave)return null
 }
 if(!Number.isFinite(leave))return null
 const at=(t:number):Point2=>[0,1].map(i=>Math.max(-1,Math.min(1,origin[i]+unit[i]*t))) as [number,number]
 return {start:at(enter),end:at(leave)}
}
/** Orthographic directions use the linear part of the view/projection matrix,
 * avoiding cancellation from subtracting two distant projected positions. */
export function projectedVectorRay(camera:THREE.OrthographicCamera,vector:Vec3):DirectionLine|null {
 if(!vector.every(Number.isFinite)||length(vector)<=STATIONARY_SPEED_MPS)return null
 const origin=new THREE.Vector3().project(camera)
 const direction=new THREE.Vector3(...vector).normalize().transformDirection(camera.matrixWorldInverse)
 return clipViewportRay([origin.x,origin.y],[direction.x*camera.projectionMatrix.elements[0],direction.y*camera.projectionMatrix.elements[5]])
}
export function shipVectorGuides(camera:THREE.OrthographicCamera,ship:ShipState) {
 return {heading:projectedVectorRay(camera,forward(ship.yaw,ship.pitch)),velocity:projectedVectorRay(camera,ship.velocityMps)}
}
