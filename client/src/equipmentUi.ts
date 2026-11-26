import { applyGearCommand, gearCommandDenial, type GearCommand } from './scoutEquipment.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { RESOURCE_CATALOG, type ResourceSession } from './resourceSession.ts'
/** Presentation drafts never hold custody; current domain admission remains authoritative. */
export function equipmentActionDenial(session:ResourceSession,command:GearCommand):string|undefined {
 try{
  if(session.error) return session.error
  if(session.survival.health===0)return 'The expedition has ended.'
  const before=session.snapshot.inventory.equipment!,after=applyGearCommand(before,command)
  const denial=gearCommandDenial(session.world,before,after)
  if(denial)return denial
  resourceInventoryUsage({...session.snapshot.inventory,equipment:after},RESOURCE_CATALOG,session.itemCatalog)
  return undefined
 }catch(e){return (e as Error).message}
}
export class EquipmentDraft {
 pending:{command:GearCommand;revision:number;inventory:string}|null=null
 message='';failed=false
 review(session:ResourceSession,command:GearCommand):void {
  const reason=equipmentActionDenial(session,command)
  this.failed=!!reason;this.message=reason??'Review the equipment move. Nothing has changed.'
  this.pending=reason?null:{command,revision:session.snapshot.revision,inventory:JSON.stringify(session.snapshot.inventory)}
 }
 confirm(session:ResourceSession):boolean {
  const draft=this.pending;this.pending=null
  if(!draft)return false
  const revision=draft.inventory===JSON.stringify(session.snapshot.inventory)?session.snapshot.revision:draft.revision
  const ok=session.changeEquipment(draft.command,revision)
  this.failed=!ok;this.message=ok?'Saved: equipment moved.':session.feedback||session.error
  return ok
 }
}
