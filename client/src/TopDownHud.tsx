import { EquipmentHud } from './EquipmentHud.tsx'
import { GearNoticeTracker, equipmentHudReadout } from './equipmentHud.ts'
import { EquipmentIcon } from './rendering/EquipmentIcon.tsx'
import { MarkerDeployment } from './MarkerDeployment.tsx'
import { METER } from './survival.ts'
import { type TopDownState } from './topDownRuntime.ts'
import { RESOURCE_CATALOG, ResourceSession } from './resourceSession.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { Button, KeyHint, Meter, UtilityPanel } from './ui/primitives.tsx'
import { StatusMessage } from './ui/Feedback.tsx'
import type { RefObject } from 'react'
interface SurvivalHudProps {
  state: TopDownState
  resourceSession: ResourceSession | null
  paused: boolean
  warning: string
  saveStatus: string
  gearNotices: GearNoticeTracker
  powerMessage: string
  onFreshRun(): void
  onTogglePause(): void
  onHelp(): void
}

export function SurvivalHud({ state, resourceSession, paused, warning, saveStatus, gearNotices, powerMessage, onFreshRun, onTogglePause, onHelp }: SurvivalHudProps) {
  const vital = resourceSession?.survival
  return <aside className="survival-hud" aria-label="Survival condition">
      {vital ? <>
        <StatusMessage priority={warning ? 'critical' : 'status'}>{[paused ? 'Paused · active time stopped' : '', warning].filter(Boolean).join(' · ')}</StatusMessage>
        <div className="vital-meters">
        <div className="ui-meter"><span>Health: {Math.ceil(vital.health)} / 200 HP</span><meter aria-label="Health" min={0} max={200} value={vital.health} aria-valuetext={`${vital.health} / 200 HP`} /></div>
        <Meter label="Hunger" value={Math.ceil(vital.hunger / METER * 100)} max={100} unit="" />
        <Meter label="Fatigue" value={Math.ceil(vital.fatigue / METER * 100)} max={100} unit="" />
        </div>
        {vital.health === 0 ? <button onClick={event => { event.currentTarget.blur(); onFreshRun() }}>Start fresh character · keep world resources</button> : <button onClick={event => { event.currentTarget.blur(); onTogglePause() }}>{paused ? 'Resume' : 'Pause'} [P]</button>}
      </> : <p>{resourceSession?.error || saveStatus}</p>}
      {resourceSession && <EquipmentHud readout={equipmentHudReadout(resourceSession,gearNotices)} />}
      {resourceSession?.error && <StatusMessage priority="error">{resourceSession.error}</StatusMessage>}
      {powerMessage && <StatusMessage priority="critical">{powerMessage}</StatusMessage>}
      {state.scoutLocation.boardingDoor === 'closed' && <p data-boarding-door-status>Aft door: closed{state.scoutLocation.binding === null ? ' · flight lock' : ''}</p>}
      {state.cockpit.view === 'radar' && resourceSession?.feedback && <StatusMessage>{resourceSession.feedback}</StatusMessage>}
      <Button data-field-help variant="secondary" onClick={onHelp}>Help</Button>
    </aside>
}

interface InventoryHudProps {
  resourceSession: ResourceSession | null
  paused: boolean
  saveStatus: string
  inventoryOpen: boolean
  inventoryTrigger: RefObject<HTMLButtonElement | null>
  inventoryPanel: RefObject<HTMLDivElement | null>
  toggleInventory(open: boolean, restoreFocus?: boolean): void
  onEquipment(): void
  onChange(): void
}

export function InventoryHud({ resourceSession, paused, saveStatus, inventoryOpen, inventoryTrigger, inventoryPanel, toggleInventory, onEquipment, onChange }: InventoryHudProps) {
  const usage = resourceSession && resourceInventoryUsage(resourceSession.snapshot.inventory, RESOURCE_CATALOG, resourceSession.itemCatalog)
  return <aside className="resource-hud" aria-label="Resource inventory" data-gathering={resourceSession?.activeSourceId ?? ''}>
        <button ref={inventoryTrigger} aria-expanded={inventoryOpen} aria-controls="field-inventory" onClick={() => toggleInventory(!inventoryOpen)}>Field inventory <KeyHint>I</KeyHint></button>
        {resourceSession && usage && <>
          <p data-inventory-weight><strong>{usage.grams / 1000}</strong> / {resourceSession.snapshot.inventory.capacityGrams / 1000} kg</p>
          <p data-inventory-slots><strong>{usage.slots}</strong> / {resourceSession.snapshot.inventory.capacitySlots} slots</p>
          <div id="field-inventory" ref={inventoryPanel} className="hud-utility" data-ui-utility hidden={!inventoryOpen} tabIndex={-1} onKeyDown={event => { if (event.code !== 'Escape') event.stopPropagation() }}>
          <UtilityPanel title="Carried resources and items">
          <ul aria-label="Raw resources">{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <li key={id}>{entry.name} <span data-resource={id}>{(resourceSession.snapshot.inventory.quantities[id] ?? 0) / 1000} kg</span></li>)}</ul>
          <p className="resource-note">Each carried raw type uses 1 slot.</p>
          <ul aria-label="Count items">{Object.entries(resourceSession.snapshot.inventory.items).map(([id, count]) => {
            const meta = resourceSession.itemCatalog[id]!
            return <li className="count-item" key={id} data-item={id}><span>{meta.name ?? id} × {count}</span><span>{count * meta.gramsPerItem / 1000} kg · {Math.ceil(count / meta.stackSize)} slots</span></li>
          })}</ul>
          <Button onClick={onEquipment}>Equipment and storage</Button>
          <MarkerDeployment session={resourceSession} blocked={paused || resourceSession.survival.health === 0 || !!resourceSession.error} onChange={onChange} />
          {!Object.keys(resourceSession.snapshot.inventory.items).length && <p data-empty-items>No items carried.</p>}
          <p className="resource-note">{resourceSession.error || saveStatus}</p>
          <Button onClick={() => toggleInventory(false, true)}>Close inventory</Button>
          </UtilityPanel></div>
        </>}
        <button disabled={!resourceSession || !!resourceSession.error || resourceSession.survival.health === 0 || paused} aria-pressed={resourceSession?.equipped ?? false} onClick={event => { resourceSession?.toggleTool(); event.currentTarget.blur(); onChange() }}><EquipmentIcon />{resourceSession?.equipped ? 'Stow gathering tool' : 'Equip gathering tool'}</button>
        {(!resourceSession || resourceSession.error) && <StatusMessage announce={!resourceSession?.error}>{resourceSession?.error || saveStatus}</StatusMessage>}
      </aside>
}
