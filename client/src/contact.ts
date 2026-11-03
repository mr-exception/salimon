import { relativeM, translate } from './coordinates.ts'
import { add, cross, dot, length, normal, scale, sub, type Vec3 } from './math.ts'
import { STEP_SECONDS, type ShipState } from './simulation.ts'
import { type Body } from './sampleUniverse.ts'
export const SHIP_RADIUS_M=5 // Conservative physical sphere, independent of radar glyphs.
export const SAFE_LANDING_MPS=20
export const DESTRUCTIVE_MPS=100
export interface ContactState {
 readonly bodyId:string
 readonly outcome:'safe-contact'|'crash'|'destroyed'
 readonly impactSpeedMps:number
 readonly normal:Vec3
 readonly ageTicks:number
}
export type Warning={readonly id:string;readonly bodyId:string;readonly severity:'proximity'|'danger';readonly message:string;readonly surfaceDistanceM:number}
export const physicalRadiusM=(body:Body)=>body.radiusKm*1000+SHIP_RADIUS_M
export function surfaceVelocity(body:Body,radial:Vec3):Vec3 {return cross(body.angularVelocityRadps??[0,0,0],radial)}
function rotate(v:Vec3,omega:Vec3):Vec3 {
 const speed=length(omega);if(speed===0)return v
 const axis=scale(omega,1/speed),angle=speed*STEP_SECONDS
 return add(add(scale(v,Math.cos(angle)),scale(cross(axis,v),Math.sin(angle))),scale(axis,dot(axis,v)*(1-Math.cos(angle))))
}
export function distances(state:ShipState,body:Body) {
 const radial=relativeM(state.position,body.position),centerDistanceM=length(radial)
 const surfaceDistanceM=centerDistanceM-physicalRadiusM(body)
 const velocityRelativeMps=sub(state.velocityMps,surfaceVelocity(body,scale(normal(radial),body.radiusKm*1000)))
 return {centerDistanceM,surfaceDistanceM,velocityRelativeMps,closingMps:-dot(velocityRelativeMps,normal(radial))}
}
// Chord sweep for each fixed simulation step, including complete crossings.
function hitFraction(start:Vec3,delta:Vec3,radius:number):number|null {
 const distance=length(start),a=dot(delta,delta),b=dot(start,delta)
 if(distance<=radius)return 0
 if(a===0||b>=0)return null
 const c=(distance-radius)*(distance+radius),discriminant=b*b-a*c
 if(discriminant<0)return null
 const t=c/(-b+Math.sqrt(discriminant))
 return t>=0&&t<=1?t:null
}
function settledStep(before:ShipState,after:ShipState,body:Body,contact:ContactState):ShipState {
 const n=normal(rotate(contact.normal,body.angularVelocityRadps??[0,0,0]))
 const ticks=contact.ageTicks+1,phase=Math.min(1,ticks/60)
 // Three bounded rebounds with decreasing height; settling is crash response,
 // not gravity, free-flight steering, assisted landing or an autopilot.
 const amplitude=Math.min(2,contact.impactSpeedMps*.02)
 const height=amplitude*Math.sin(3*Math.PI*phase)**2*(1-phase)**2
 const radialSpeed=amplitude*(6*Math.PI*Math.sin(3*Math.PI*phase)*Math.cos(3*Math.PI*phase)*(1-phase)**2-2*Math.sin(3*Math.PI*phase)**2*(1-phase))
 const radial=scale(n,physicalRadiusM(body)+height)
 return {...after,position:translate(body.position,radial),velocityMps:add(surfaceVelocity(body,scale(n,body.radiusKm*1000)),scale(n,radialSpeed)),throttle:0,alignment:null,mode:phase===1?'grounded':before.mode,contact:{...contact,normal:n,ageTicks:ticks}}
}
export function resolveContacts(before:ShipState,after:ShipState,bodies:readonly Body[]):ShipState {
 if(before.mode==='destroyed')return {...before,tick:after.tick}
 if(before.contact && (before.mode==='recovering'||before.mode==='grounded')) {
  const body=bodies.find(b=>b.id===before.contact?.bodyId)
  if(!body)throw new Error('Contact body disappeared from immutable catalog')
  return settledStep(before,after,body,before.contact)
 }
 const delta=relativeM(after.position,before.position)
 let hit:{body:Body;t:number;n:Vec3;speed:number}|null=null
 for(const body of bodies) {
  const start=relativeM(before.position,body.position),t=hitFraction(start,delta,physicalRadiusM(body))
  if(t===null)continue
  const n=normal(add(start,scale(delta,t)))
  const velocity=add(before.velocityMps,scale(sub(after.velocityMps,before.velocityMps),t))
  const relative=sub(velocity,surfaceVelocity(body,scale(n,body.radiusKm*1000)))
  // A touching ship that departs or travels tangent is not an impact.
  if(length(start)>=physicalRadiusM(body)-1e-7 && dot(relative,n)>=0)continue
  if(!hit || t<hit.t || (t===hit.t&&body.id<hit.body.id))hit={body,t,n,speed:length(relative)}
 }
 if(!hit)return after
 const {body,n,speed}=hit
 const outcome=speed>=DESTRUCTIVE_MPS?'destroyed':((body.kind==='planet'||body.kind==='moon')&&speed<=SAFE_LANDING_MPS?'safe-contact':'crash')
 return {...after,position:translate(body.position,scale(n,physicalRadiusM(body))),velocityMps:outcome==='destroyed'?[0,0,0]:surfaceVelocity(body,scale(n,body.radiusKm*1000)),throttle:0,alignment:null,mode:outcome==='destroyed'?'destroyed':'recovering',contact:{bodyId:body.id,outcome,impactSpeedMps:speed,normal:n,ageTicks:0}}
}
export function navigationWarnings(state:ShipState,contacts:readonly Body[]):Warning[] {
 if(state.mode!=='flying')return []
 const warnings=new Map<string,Warning>()
 for(const body of contacts) {
  const d=distances(state,body)
  const danger=d.closingMps>0 && d.surfaceDistanceM/d.closingMps<=30 && length(d.velocityRelativeMps)>=DESTRUCTIVE_MPS
  if(d.surfaceDistanceM>1000&&!danger)continue
  const severity=danger?'danger':'proximity'
  warnings.set(body.id,{id:`${body.id}:${severity}`,bodyId:body.id,severity,surfaceDistanceM:d.surfaceDistanceM,message:`${body.name}: ${danger?'dangerous approach':'surface proximity'}`})
 }
 return [...warnings.values()].sort((a,b)=>Number(b.severity==='danger')-Number(a.severity==='danger')||a.surfaceDistanceM-b.surfaceDistanceM||a.bodyId.localeCompare(b.bodyId)).slice(0,3)
}
