import { memo } from 'react'
import { Meter } from './ui/primitives.tsx'
import './equipmentUi.css'
/** Parent publishes stable rounded readouts: an unchanged admitted tick does not rerender this HUD. */
export const EquipmentHud=memo(function EquipmentHud({readout}:{readout:string}){
 const m=JSON.parse(readout) as {equipped:string;slots:string[];temperature:number;damage:number;radiation:number;poison:number;warnings:string[];notices:string[]}
 const hazardous=m.damage>0||m.radiation>0||m.poison>0
 return <section className="gear-hud" aria-label="Equipment and environmental condition">
  <details><summary>Spacesuit: {m.equipped}</summary>{m.slots.map((text,i)=><p key={i}>Slot {i+1}: {text}</p>)}</details>
  <p>Local temperature: {m.temperature}°C · Damage: {m.damage} HP/s</p>
  {hazardous&&<><Meter label="Radiation exposure" value={m.radiation} max={100} unit=""/><Meter label="Toxicity exposure" value={m.poison} max={100} unit=""/></>}
  <div role="status" aria-label="Equipment warnings" aria-live="polite">{m.warnings.map(w=><p key={w}>⚠ {w}</p>)}</div>
  <div role="alert" aria-label="Equipment destruction">{m.notices.map(n=><p key={n}>! {n}</p>)}</div>
 </section>
})
