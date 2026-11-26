import * as q from './exact.ts'
import { gearName } from './equipment.ts'
import { GEAR_RECIPES, BENCH_WATTS } from './gearBench.ts'
import { useState } from 'react'
import { RESOURCE_CATALOG } from './resourceSession.ts'
import { resourceStoreMass } from './resourceLedger.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { FABRICATOR_GRAMS, FABRICATOR_TICKS, FABRICATOR_WATTS, FABRICATOR_ITEMS, MARKER_KIT_ID } from './scoutFabricator.ts'
import { CORE_MAX_WATTS, CORE_YIELD_J } from './scoutEnergyCore.ts'
import type { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import { Button, EquipmentDialog, Progress } from './ui/primitives.tsx'
import { StatusMessage } from './ui/Feedback.tsx'
import './resourceContainer.css'

export function ResourceContainerPanel({ interaction, onClose, onChange }: { interaction: ResourceContainerInteraction; onClose: () => void; onChange: () => void }) {
  const [resourceId, setResourceId] = useState('mineral'), [amount, setAmount] = useState('1000')
  const { session, pending } = interaction, inventory = session.snapshot.inventory
  const bench = interaction.equipment === 'bench'
  const core = interaction.equipment === 'core', fabricator = interaction.equipment === 'fabricator', title = interaction.title
  const store = session.snapshot.stores[interaction.storeId]!
  const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, interaction.session.itemCatalog), stored = interaction.bufferGrams
  const review = (direction: 'deposit' | 'withdraw') => { interaction.review(direction, resourceId, amount); onChange() }
  const blockedReason = session.error ? 'Saving is blocked. Close and reload to inspect the saved expedition.' : undefined
  const tone = interaction.failed || session.error ? 'error' : pending ? 'pending' : interaction.message ? 'confirmation' : 'neutral'
  return <div className="expedition-ui resource-container">
    <EquipmentDialog open title={title} eyebrow={core || fabricator || bench ? "Scout / Engineering" : "Scout / Cargo bay"} onDismiss={onClose}
      status={<p className={`container-state container-state--${tone}`}>{tone === 'error' ? session.error ? 'Saving blocked · nothing moved' : fabricator ? 'Action rejected · nothing moved' : 'Transfer rejected · nothing moved' : pending ? 'Pending review · not committed' : tone === 'confirmation' ? fabricator ? 'Action committed' : 'Transfer committed' : fabricator ? 'Input transfer · ready' : core ? 'Fuel transfer · ready' : 'Raw storage · ready'}</p>}
      feedback={<StatusMessage label="Container feedback" priority={tone === 'error' ? 'error' : pending ? 'pending' : tone === 'confirmation' ? 'confirmation' : 'status'}>{session.error || interaction.message || 'Choose a raw resource and amount. Committed transfers save immediately.'}</StatusMessage>}>
    {bench && <BenchTelemetry interaction={interaction} onChange={onChange} />}
    {core && <CoreTelemetry interaction={interaction} />}
    {fabricator && <FabricatorTelemetry interaction={interaction} onChange={onChange} />}
    <div className="container-capacities">
      <div><strong>Carried</strong><p data-carried-capacity>{used.grams / 1000} / {inventory.capacityGrams / 1000} kg</p><Progress label="Carried weight" value={used.grams} max={inventory.capacityGrams} unit="g" /><small>{used.slots} / {inventory.capacitySlots} slots · {(inventory.capacityGrams - used.grams) / 1000} kg free</small></div>
      <div><strong>{fabricator || bench ? 'Input buffer · includes reserved' : core ? 'Unconverted fuel buffer' : 'Container'}</strong><p data-container-capacity>{stored / 1000} / {store.capacityGrams / 1000} kg</p><Progress label={core ? 'Fuel buffer weight' : 'Container weight'} max={store.capacityGrams} value={stored} unit="g" /><small>{(store.capacityGrams - stored) / 1000} kg free</small></div>
    </div>
    <table><caption className="container-sr-only">Available raw resources</caption><thead><tr><th>Raw resource</th><th>Carried · kg</th><th>{fabricator || bench ? 'Unreserved · g / kg' : core ? 'Fuel · g / kg' : 'Container · kg'}</th></tr></thead><tbody>{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <tr key={id}><th scope="row">{entry.name}</th><td data-carried-raw={id}>{(inventory.quantities[id] ?? 0) / 1000}</td><td data-container-raw={id}>{core || fabricator || bench ? `${store.quantities[id] ?? 0} g / ${(store.quantities[id] ?? 0) / 1000} kg` : (store.quantities[id] ?? 0) / 1000}</td></tr>)}</tbody></table>
    {core && <p className="container-note">Only unconverted fuel can be withdrawn. Conversion continues during review; confirmation rechecks the exact amount. Escape or leaving this console discards the draft.</p>}
    {(fabricator || bench) && <p className="container-note">Only unreserved ingredients can be withdrawn. Started jobs cannot be cancelled. Closing the console keeps the world and powered job running; closed tabs make no progress.</p>}
    <form onSubmit={event => { event.preventDefault(); if (pending && !session.error) { interaction.confirm(); onChange() } }}>
      <div className="container-fields"><label>Raw resource<select value={resourceId} onChange={event => { setResourceId(event.target.value); interaction.discard(); onChange() }}>{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <option key={id} value={id}>{entry.name}</option>)}</select></label>
      <label>Amount in grams<input autoFocus inputMode="numeric" value={amount} onChange={event => { setAmount(event.target.value); interaction.discard(); onChange() }} aria-describedby="container-amount-help" /></label></div>
      <p id="container-amount-help" className="container-note">1,000 g = 1 kg. Enter a positive whole gram amount; requests never transfer a smaller amount automatically.</p>
      {pending ? <div className="container-confirm"><p className="container-note">Pending draft · nothing has moved yet</p><p>{pending.direction === 'deposit' ? 'Deposit' : 'Withdraw'} <strong>{pending.grams} g ({pending.grams / 1000} kg)</strong> of {RESOURCE_CATALOG[pending.resourceId as keyof typeof RESOURCE_CATALOG].name}?</p><div className="container-actions"><Button type="submit" disabledReason={blockedReason}>Confirm {pending.direction}</Button><Button variant="secondary" onClick={() => { interaction.discard(); onChange() }}>Cancel transfer</Button></div></div>
        : <div className="container-actions"><Button disabledReason={blockedReason} onClick={() => review('deposit')}>Review deposit</Button><Button variant="secondary" disabledReason={blockedReason} onClick={() => review('withdraw')}>Review withdrawal</Button></div>}
    </form>
    </EquipmentDialog>
  </div>
}

/** Read committed session state and receipts; presentation never advances power. */
function CoreTelemetry({ interaction }: { interaction: ResourceContainerInteraction }) {
  const { session } = interaction, { core, stores } = session.snapshot
  const fuel = stores[interaction.storeId]!, grams = resourceStoreMass(fuel, RESOURCE_CATALOG)
  const eligible = Object.entries(fuel.quantities).filter(([id, amount]) => amount > 0 && CORE_YIELD_J[id])
  const canFit = eligible.some(([id]) => core.capacityJ * 60 - core.reserveJ60 >= CORE_YIELD_J[id]! * 60)
  const state = session.error ? 'Saving blocked · processing stopped' : core.reserveJ60 === 0 && !grams ? 'Outage · no reserve or fuel' : core.reserveJ60 === core.capacityJ * 60 ? 'Reserve full · conversion waiting' : eligible.length && canFit ? 'Processing · converting fuel' : eligible.length ? 'Idle · waiting for reserve space' : 'Fuel empty · reserve available'
  return <section aria-label="Core telemetry">
    <StatusMessage label="Core operating status" priority={session.error || core.reserveJ60 === 0 && !grams ? 'error' : 'status'}>{state}</StatusMessage>
    <p data-core-reserve>Reserve: {core.reserveJ60 / 60} / {core.capacityJ} J</p>
    <Progress label="Energy reserve" value={core.reserveJ60 / 60} max={core.capacityJ} unit="J" />
    <p data-core-generation>Generation: {session.corePowerTick === null ? '— W · awaiting simulation tick' : `${session.corePower.generatedJ * 60} W · last committed tick`}</p>
    <p data-core-consumption>Consumption: {session.corePowerTick === null ? '— W · awaiting simulation tick' : `${session.corePower.drawnJ60} W · last committed tick`}</p>
    <p className="container-note">Whole-gram conversion pulses use a {CORE_MAX_WATTS.toLocaleString('en-US')} W budget. Consumer: {session.corePowerTick === null ? 'awaiting simulation tick' : session.corePower.consumer}. Flight uses independent prototype power.</p>
    {grams === fuel.capacityGrams && <StatusMessage label="Fuel warning">Fuel buffer full · withdraw fuel or wait for conversion.</StatusMessage>}
    {session.corePower.consumer === 'insufficient-power' && <StatusMessage priority="error" label="Power warning">Insufficient reserve · consumer paused.</StatusMessage>}
  </section>
}

/** One trusted recipe; job and tray are read directly from the saved domain. */
function FabricatorTelemetry({ interaction, onChange }: { interaction: ResourceContainerInteraction; onChange: () => void }) {
  const { session } = interaction, { fabricator: f, core } = session.snapshot
  const job = f.job
  const status = session.error ? 'Saving blocked · processing stopped' : f.tray ? 'Complete · output ready to claim' : job ? job.powerPaused ? 'Paused · insufficient Core power' : 'Powered · fabricating' : interaction.startBlocked ? 'Blocked · missing ingredients' : 'Ready · one job available'
  return <section aria-label="Fabricator job and output">
    <StatusMessage label="Fabricator operating status" priority={session.error || job?.powerPaused ? 'error' : 'status'}>{status}</StatusMessage>
    <p>Selected recipe: <strong>{FABRICATOR_ITEMS[MARKER_KIT_ID].name}</strong></p>
    <p className="container-note">{FABRICATOR_GRAMS.toLocaleString('en-US')} g mineral → 1 kit · 1,000 g · 1 slot. Deploy outdoors from Field inventory.</p>
    <p className="container-note">{FABRICATOR_TICKS / 60} s at {FABRICATOR_WATTS.toLocaleString('en-US')} W · 100,000 J. Queue: {job ? '1 / 1 active' : '0 / 1 active'}.</p>
    {job && <><Progress label="Fabrication progress" value={FABRICATOR_TICKS - job.remainingTicks} max={FABRICATOR_TICKS} unit="ticks" /><p data-fabricator-job>Job #{job.id} · {job.remainingTicks} ticks remaining ({(job.remainingTicks / 60).toFixed(2)} s) · {job.reservedGrams} g reserved</p></>}
    <p data-fabricator-power>Core reserve: {core.reserveJ60 / 60} J · job draw: {session.corePowerTick === null ? '— W · awaiting simulation tick' : `${session.corePower.drawnJ60} W · last committed tick`}</p>
    <div className="container-actions"><Button disabledReason={interaction.startBlocked} onClick={() => { interaction.start(); onChange() }}>Start fabrication</Button></div>
    <p data-fabricator-tray>Output tray: {f.tray ? `1 Field marker kit · 1,000 g · job #${f.tray.jobId}` : 'Empty · 0 / 1'}</p>
    {f.tray && <div className="container-actions"><Button disabledReason={interaction.machineBlocked} onClick={() => { interaction.claim(); onChange() }}>Claim output</Button></div>}
  </section>
}

function BenchTelemetry({interaction,onChange}:{interaction:ResourceContainerInteraction;onChange:()=>void}) {
 const [recipe,setRecipe]=useState('suit:basic'), [suitId,setSuitId]=useState(''), [repairHp,setRepairHp]=useState('1')
 const {bench:b}=interaction.session.snapshot,r=GEAR_RECIPES[recipe]!,job=b.job
 const suits=interaction.session.snapshot.inventory.equipment!.instances.filter(g=>g.definitionId==='suit:basic'&&g.location.kind==='inventory')
 const reason=interaction.machineBlocked||(job?'One bench job is active.':b.tray?'Claim the output tray first.':undefined)
 return <section aria-label="Gear bench job and output">
  <label>Equipment recipe<select value={recipe} onChange={e=>setRecipe(e.target.value)}>{Object.keys(GEAR_RECIPES).map(id=><option key={id} value={id}>{gearName(id)}</option>)}</select></label>
  <p>{r.mineral} g mineral + {r.biomass} g biomass · {r.ticks/60} s at {BENCH_WATTS} W</p>
  <p>Recipe input minus equipment mass is consumed without byproducts.</p>
  <p data-bench-status>{b.tray?'Complete · output ready':job?job.powerPaused?'Paused · insufficient power':'Powered · crafting':'Ready'}</p>
  {job&&<><Progress label="Gear crafting progress" value={(job.repair?Math.ceil(q.number(q.decode(job.repair.amount))*3):GEAR_RECIPES[job.recipeId]!.ticks)-job.remainingTicks} max={job.repair?Math.ceil(q.number(q.decode(job.repair.amount))*3):GEAR_RECIPES[job.recipeId]!.ticks} unit="ticks"/><p>{job.remainingTicks} paid ticks remaining · {job.repair?'Suit repair':gearName(job.recipeId)}</p></>}
  <Button disabledReason={reason} onClick={()=>{interaction.startBench(recipe);onChange()}}>Start gear crafting</Button>
  <fieldset><legend>Repair a carried unequipped spacesuit</legend>
    <label>Spacesuit to repair<select value={suitId} onChange={e=>setSuitId(e.target.value)}><option value="">Choose a suit</option>{suits.map((g,i)=><option key={g.id} value={g.id}>Suit {i+1} · {q.number(q.decode(g.remaining))} / 1000 HP · {g.location.kind}</option>)}</select></label>
    <label>HP to restore<input value={repairHp} onChange={e=>setRepairHp(e.target.value)} inputMode="decimal" /></label>
    <p>1 g mineral per HP, rounded up · 20 HP/s at 2 kW. Modules stay attached unchanged.</p>
    <Button disabledReason={reason||(!suitId?'Choose an unequipped suit.':undefined)} onClick={()=>{try{const amount=repairHp.includes('/')?q.encode(q.decode(repairHp)):q.encode(q.decimal(Number(repairHp)));interaction.repairSuit(suitId,amount)}catch{interaction.failed=true;interaction.message='Enter a positive HP amount or exact fraction.'}onChange()}}>Start suit repair</Button>
  </fieldset>
  <p data-bench-tray>Output tray: {b.tray?b.tray.repair?'Repaired spacesuit':gearName(b.tray.recipeId):'Empty'}</p>
  {b.tray&&<Button disabledReason={interaction.machineBlocked} onClick={()=>{interaction.claimBench();onChange()}}>Claim equipment</Button>}
 </section>
}
