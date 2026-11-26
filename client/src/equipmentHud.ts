import { functioningEquipment, GEAR_DEFINITIONS, gearName, type EquipmentState } from './equipment.ts'
import * as q from './exact.ts'
import type { ResourceSession } from './resourceSession.ts'
/** Uses admitted active ticks for temporary notices, with no second timer. */
export class GearNoticeTracker {
 private seen=new Map<string,string>()
 private notices=new Map<string,{text:string;until:number}>()
 update(e:EquipmentState,tick:number):string[] {
  for(const g of e.instances){
   if(g.location.kind==='destroyed'&&this.seen.has(g.id)&&this.seen.get(g.id)!=='0/1')this.notices.set(g.id,{text:`${gearName(g.definitionId)} destroyed · protection lost`,until:tick+300})
   this.seen.set(g.id,g.remaining)
  }
  for(const [id,n] of this.notices)if(tick>=n.until)this.notices.delete(id)
  return [...this.notices.values()].map(n=>n.text)
 }
}
export function equipmentWarnings(e:EquipmentState):string[] {
 return functioningEquipment(e).flatMap(g=>q.cmp(q.decode(g.remaining),q.decimal(200))<=0?[`${gearName(g.definitionId)} · ${q.cmp(q.decode(g.remaining),q.decimal(50))<=0?'Critical: 5% or less':'Low: 20% or less'}`]:[])
}
const rounded=(n:number)=>Math.round(n*10)/10
export function equipmentHudReadout(s:ResourceSession,tracker:GearNoticeTracker):string {
 const gear=s.snapshot.inventory.equipment!,suit=gear.instances.find(g=>g.location.kind==='equipped'),environment=s.currentEnvironment()
 const equipped=suit?`${rounded(q.number(q.decode(suit.remaining)))} / 1000 HP${suit.remaining==='0/1'?' · Destroyed / inactive':''}`:'No spacesuit · no protection'
 const slots=[0,1].map(slot=>{const m=gear.instances.find(g=>g.location.kind==='installed'&&g.location.suitId===suit?.id&&g.location.slot===slot);return m?`${gearName(m.definitionId)} · ${GEAR_DEFINITIONS[m.definitionId]!.protection} HP/s · ${rounded(q.number(q.decode(m.remaining)))} HP`:'Empty'})
 return JSON.stringify({equipped,slots,temperature:environment.temperatureC,damage:rounded(s.environmentDamageHpPerSecond),radiation:rounded(q.number(q.decode(s.hazards.exposure.radiation))),poison:rounded(q.number(q.decode(s.hazards.exposure.poison))),warnings:equipmentWarnings(gear),notices:tracker.update(gear,s.survival.activeTicks)})
}
