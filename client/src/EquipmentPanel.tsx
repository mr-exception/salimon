import { memo, useEffect, useRef, useState } from 'react'
import { GEAR_DEFINITIONS, gearName, type GearInstance, type GearLocation } from './equipment.ts'
import type { ResourceSession } from './resourceSession.ts'
import type { GearCommand } from './scoutEquipment.ts'
import { EquipmentDraft, equipmentActionDenial } from './equipmentUi.ts'
import { EquipmentDialog, Button, Progress } from './ui/primitives.tsx'
import { StatusMessage } from './ui/Feedback.tsx'
import * as q from './exact.ts'
import './equipmentUi.css'
const hp=(value:string)=>q.number(q.decode(value))
export const EquipmentPanel=memo(function EquipmentPanel({session,onClose,onChange}:{session:ResourceSession;onClose:()=>void;onChange:()=>void;viewKey:string}) {
 const [,refresh]=useState(0)
 const changed=()=>{refresh(n=>n+1);onChange()}
 const draft=useRef(new EquipmentDraft()).current
 const confirmation=useRef<HTMLElement>(null)
 useEffect(()=>{if(draft.pending)confirmation.current?.querySelector<HTMLButtonElement>('button')?.focus()},[draft.pending])
 const [removedTo,setRemovedTo]=useState<'inventory'|'drawer'>('inventory')
 const gear=session.snapshot.inventory.equipment!.instances,equipped=gear.find(g=>g.location.kind==='equipped')
 const suits=gear.filter(g=>GEAR_DEFINITIONS[g.definitionId]!.kind==='suit'&&g.location.kind!=='bench')
 const [selectedSuit,setSelectedSuit]=useState(equipped?.id??suits[0]?.id??'')
 const suit=suits.find(g=>g.id===selectedSuit)??equipped??suits[0]
 const review=(command:GearCommand)=>{draft.review(session,command);changed()}
 const move=(g:GearInstance,to:GearLocation):GearCommand=>({kind:'move',id:g.id,from:g.location,to})
 const action=(label:string,command:GearCommand)=><Button key={label} disabledReason={equipmentActionDenial(session,command)} onClick={()=>review(command)}>{label}</Button>
 return <div className="expedition-ui equipment-panel"><EquipmentDialog open title="Equipment and storage" eyebrow="Scout / Survival equipment" onDismiss={onClose}
  feedback={<StatusMessage label="Equipment feedback" priority={draft.failed?'error':draft.pending?'pending':draft.message?'confirmation':'status'}>{draft.message||'Equipment changes require standing inside Scout. Approach the cargo cabinet for storage.'}</StatusMessage>}>
  <section aria-label="Equipped suit"><h3>Equipped spacesuit</h3>
   <p>{equipped?`${hp(equipped.remaining).toFixed(2)} / 1000 HP${hp(equipped.remaining)===0?' · Destroyed / inactive · modules disabled':''}`:'No spacesuit equipped · no protection'}</p>
   {[0,1].map(slot=>{const module=gear.find(g=>g.location.kind==='installed'&&g.location.suitId===equipped?.id&&g.location.slot===slot);return <p key={slot}>Slot {slot+1}: {module?`${gearName(module.definitionId)} · ${GEAR_DEFINITIONS[module.definitionId]!.protection} HP/s · ${hp(module.remaining).toFixed(2)} / 1000 HP`:'Empty'}</p>})}
  </section>
  <section aria-label="Equipment storage capacity"><h3>Scout storage</h3><p>Suit drawer: {gear.filter(g=>g.location.kind==='drawer').length} / 4 suits</p><p>Module storage: {gear.filter(g=>g.location.kind==='module-storage').length} / 12 modules</p><p>No additional storage weight limit. Equipped gear is excluded from carried capacity.</p></section>
  <label>Removed suit destination<select value={removedTo} onChange={e=>setRemovedTo(e.target.value as typeof removedTo)}><option value="inventory">Carried inventory</option><option value="drawer">Suit drawer</option></select></label>
  <label>Module installation suit<select value={suit?.id??''} onChange={e=>setSelectedSuit(e.target.value)}>{suits.map((g,i)=><option key={g.id} value={g.id}>Suit {i+1} · {g.location.kind} · {hp(g.remaining).toFixed(2)} HP</option>)}</select></label>
  <ul className="gear-list" aria-label="Equipment inventory">{gear.filter(g=>g.location.kind!=='destroyed').map((g,i)=>{
   const d=GEAR_DEFINITIONS[g.definitionId]!,blocked=g.location.kind==='bench'
   return <li key={g.id} data-gear-id={g.id}><strong>{gearName(g.definitionId)}</strong><p>Item {i+1} · {g.location.kind==='installed'?'Installed in suit':g.location.kind==='bench'?'Reserved by bench':g.location.kind} · {d.grams/1000} kg</p>
    <Progress label={`${gearName(g.definitionId)} condition ${i+1}`} value={hp(g.remaining)} max={1000} unit="HP"/>
    {d.kind==='module'&&<p>{d.protection} HP/s protection · {g.location.kind==='installed'&&gear.some(s=>s.id===(g.location as {suitId:string}).suitId&&hp(s.remaining)===0)?'Inactive: suit destroyed':'Requires a functioning equipped suit'}</p>}
    {!blocked&&<div className="gear-actions">
     {g.location.kind!=='inventory'&&action('Carry item',move(g,{kind:'inventory'}))}
     {d.kind==='suit'?<>
      {g.location.kind!=='equipped'&&action('Equip suit',equipped?{kind:'switch',id:g.id,from:g.location,equippedId:equipped.id,unequipTo:{kind:removedTo}}:move(g,{kind:'equipped'}))}
      {g.location.kind!=='drawer'&&action('Store suit',move(g,{kind:'drawer'}))}
     </>:<>
      {g.location.kind!=='module-storage'&&action('Store module',move(g,{kind:'module-storage'}))}
      {suit&&[0,1].map(slot=>action(`Install in slot ${slot+1}`,move(g,{kind:'installed',suitId:suit.id,slot:slot as 0|1})))}
     </>}
    </div>}
   </li>
  })}</ul>
  {draft.pending&&<section ref={confirmation} aria-label="Confirm equipment change"><p>Move selected equipment from {draft.pending.command.from.kind} to {draft.pending.command.kind==='switch'?`equipped; removed suit goes to ${draft.pending.command.unequipTo.kind}`:draft.pending.command.to.kind}?</p><Button onClick={()=>{draft.confirm(session);changed()}}>Confirm equipment change</Button><Button variant="secondary" onClick={()=>{draft.pending=null;draft.message='Equipment change cancelled.';changed()}}>Cancel change</Button></section>}
 </EquipmentDialog></div>
},(before,after)=>before.viewKey===after.viewKey)
