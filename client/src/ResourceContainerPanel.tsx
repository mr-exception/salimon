import { useEffect, useRef, useState } from 'react'
import { RESOURCE_CATALOG } from './resourceSession.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { resourceStoreMass } from './resourceLedger.ts'
import { SCOUT_RESOURCE_CONTAINER_ID } from './scoutResourceContainer.ts'
import type { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import './resourceContainer.css'

export function ResourceContainerPanel({ interaction, onClose, onChange }: { interaction: ResourceContainerInteraction; onClose: () => void; onChange: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null)
  const [resourceId, setResourceId] = useState('mineral'), [amount, setAmount] = useState('1000')
  useEffect(() => { dialog.current?.showModal() }, [])
  const { session, pending } = interaction, inventory = session.snapshot.inventory
  const store = session.snapshot.stores[SCOUT_RESOURCE_CONTAINER_ID]!
  const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, interaction.session.itemCatalog), stored = resourceStoreMass(store, RESOURCE_CATALOG)
  const review = (direction: 'deposit' | 'withdraw') => { interaction.review(direction, resourceId, amount); onChange() }
  return <dialog ref={dialog} className="resource-container" aria-labelledby="container-title" onCancel={event => { event.preventDefault(); onClose() }}>
    <header><div><p className="container-eyebrow">SCOUT / CARGO BAY</p><h2 id="container-title">Resource Container</h2></div><button onClick={onClose} aria-label="Close Resource Container">Close · Esc</button></header>
    <p className="container-note">Raw resources · exact gram transfers</p>
    <div className="container-capacities">
      <div><strong>Carried</strong><p data-carried-capacity>{used.grams / 1000} / {inventory.capacityGrams / 1000} kg</p><small>{used.slots} / {inventory.capacitySlots} slots · {(inventory.capacityGrams - used.grams) / 1000} kg free</small></div>
      <div><strong>Container</strong><p data-container-capacity>{stored / 1000} / {store.capacityGrams / 1000} kg</p><small>{(store.capacityGrams - stored) / 1000} kg free</small><progress aria-label="Container weight" max={store.capacityGrams} value={stored} /></div>
    </div>
    <table><caption className="container-sr-only">Available raw resources in kilograms</caption><thead><tr><th>Raw resource</th><th>Carried · kg</th><th>Container · kg</th></tr></thead><tbody>{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <tr key={id}><th scope="row">{entry.name}</th><td data-carried-raw={id}>{(inventory.quantities[id] ?? 0) / 1000}</td><td data-container-raw={id}>{(store.quantities[id] ?? 0) / 1000}</td></tr>)}</tbody></table>
    <form onSubmit={event => { event.preventDefault(); if (pending) { interaction.confirm(); onChange() } }}>
      <div className="container-fields"><label>Raw resource<select value={resourceId} onChange={event => { setResourceId(event.target.value); interaction.discard(); onChange() }}>{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <option key={id} value={id}>{entry.name}</option>)}</select></label>
      <label>Amount in grams<input autoFocus inputMode="numeric" value={amount} onChange={event => { setAmount(event.target.value); interaction.discard(); onChange() }} aria-describedby="container-amount-help" /></label></div>
      <p id="container-amount-help" className="container-note">1,000 g = 1 kg. Enter a positive whole gram amount; requests never transfer a smaller amount automatically.</p>
      {pending ? <div className="container-confirm"><p>{pending.direction === 'deposit' ? 'Deposit' : 'Withdraw'} <strong>{pending.grams} g ({pending.grams / 1000} kg)</strong> of {RESOURCE_CATALOG[pending.resourceId as keyof typeof RESOURCE_CATALOG].name}?</p><div className="container-actions"><button type="submit">Confirm {pending.direction}</button><button type="button" onClick={() => { interaction.discard(); onChange() }}>Cancel transfer</button></div></div>
        : <div className="container-actions"><button type="button" disabled={!!session.error} onClick={() => review('deposit')}>Review deposit</button><button type="button" disabled={!!session.error} onClick={() => review('withdraw')}>Review withdrawal</button></div>}
    </form>
    <p className={interaction.failed || session.error ? 'container-feedback container-error' : 'container-feedback'} role="status" aria-label="Container feedback" aria-live="polite">{session.error || interaction.message || 'Choose a raw resource and amount. Committed transfers save immediately.'}</p>
  </dialog>
}
