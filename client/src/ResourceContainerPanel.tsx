import { useState } from 'react'
import { RESOURCE_CATALOG } from './resourceSession.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { resourceStoreMass } from './resourceLedger.ts'
import { CORE_MAX_WATTS, CORE_YIELD_J } from './scoutEnergyCore.ts'
import type { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import { Button, EquipmentDialog, Progress } from './ui/primitives.tsx'
import { StatusMessage } from './ui/Feedback.tsx'
import './resourceContainer.css'

export function ResourceContainerPanel({ interaction, onClose, onChange }: { interaction: ResourceContainerInteraction; onClose: () => void; onChange: () => void }) {
  const [resourceId, setResourceId] = useState('mineral'), [amount, setAmount] = useState('1000')
  const { session, pending } = interaction, inventory = session.snapshot.inventory
  const core = interaction.equipment === 'core', title = interaction.title
  const store = session.snapshot.stores[interaction.storeId]!
  const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, interaction.session.itemCatalog), stored = resourceStoreMass(store, RESOURCE_CATALOG)
  const review = (direction: 'deposit' | 'withdraw') => { interaction.review(direction, resourceId, amount); onChange() }
  const blockedReason = session.error ? 'Saving is blocked. Close and reload to inspect the saved expedition.' : undefined
  const tone = interaction.failed || session.error ? 'error' : pending ? 'pending' : interaction.message ? 'confirmation' : 'neutral'
  return <div className="expedition-ui resource-container">
    <EquipmentDialog open title={title} eyebrow={core ? "Scout / Engineering" : "Scout / Cargo bay"} onDismiss={onClose}
      status={<p className={`container-state container-state--${tone}`}>{tone === 'error' ? session.error ? 'Saving blocked · nothing moved' : 'Transfer rejected · nothing moved' : pending ? 'Pending review · not committed' : tone === 'confirmation' ? 'Transfer committed' : core ? 'Fuel transfer · ready' : 'Raw storage · ready'}</p>}
      feedback={<StatusMessage label="Container feedback" priority={tone === 'error' ? 'error' : pending ? 'pending' : tone === 'confirmation' ? 'confirmation' : 'status'}>{session.error || interaction.message || 'Choose a raw resource and amount. Committed transfers save immediately.'}</StatusMessage>}>
    {core && <CoreTelemetry interaction={interaction} />}
    <div className="container-capacities">
      <div><strong>Carried</strong><p data-carried-capacity>{used.grams / 1000} / {inventory.capacityGrams / 1000} kg</p><Progress label="Carried weight" value={used.grams} max={inventory.capacityGrams} unit="g" /><small>{used.slots} / {inventory.capacitySlots} slots · {(inventory.capacityGrams - used.grams) / 1000} kg free</small></div>
      <div><strong>{core ? 'Unconverted fuel buffer' : 'Container'}</strong><p data-container-capacity>{stored / 1000} / {store.capacityGrams / 1000} kg</p><Progress label={core ? 'Fuel buffer weight' : 'Container weight'} max={store.capacityGrams} value={stored} unit="g" /><small>{(store.capacityGrams - stored) / 1000} kg free</small></div>
    </div>
    <table><caption className="container-sr-only">Available raw resources</caption><thead><tr><th>Raw resource</th><th>Carried · kg</th><th>{core ? 'Fuel · g / kg' : 'Container · kg'}</th></tr></thead><tbody>{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <tr key={id}><th scope="row">{entry.name}</th><td data-carried-raw={id}>{(inventory.quantities[id] ?? 0) / 1000}</td><td data-container-raw={id}>{core ? `${store.quantities[id] ?? 0} g / ${(store.quantities[id] ?? 0) / 1000} kg` : (store.quantities[id] ?? 0) / 1000}</td></tr>)}</tbody></table>
    {core && <p className="container-note">Only unconverted fuel can be withdrawn. Conversion continues during review; confirmation rechecks the exact amount. Escape or leaving this console discards the draft.</p>}
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
