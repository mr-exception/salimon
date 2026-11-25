import { useEffect, useRef, useState } from 'react'
import { navigationWarnings } from './contact.ts'
import './sensorMarkers.css'
import { StatusMessage } from './ui/Feedback.tsx'
import { contactCommandAvailability, lookAtContact } from './contactCommands.ts'
import type { ShipState } from './simulation.ts'
import type { Position } from './coordinates.ts'
import type { ObservationCameraState } from './observationCamera.ts'
import type { Body } from './sampleUniverse.ts'
import type { Scanner } from './scanner.ts'
import { currentContacts, perspectiveMarkers, selectContact, selectedContact } from './perspectiveContacts.ts'

/** DOM companion to ObservationCamera + OpticalSky. Parent owns scanner ticks,
 * physical origin and optional flight action. No optical raycaster/hit meshes.
 * Every handler rechecks live scanner authority, including stale DOM events.
 */
export function PerspectiveContacts({ scanner, geometry, view, getOrigin, getShip, canPilot = () => true, onSetDirection, destinationId = null, compact = false, browserOpen = true, onBrowserChange }: {
 destinationId?: string | null
 compact?: boolean; browserOpen?: boolean; onBrowserChange?: (open: boolean) => void
 scanner: Scanner; geometry: readonly Body[]; view: ObservationCameraState; getOrigin: () => Position
 getShip: () => ShipState; canPilot?: () => boolean
 onSetDirection?: (body: Body) => void
}) {
 // Preserve native disabled/tab behavior while consuming the shared control skin.
 const buttonClass = compact ? 'ui-button ui-button--secondary' : undefined
 const root = useRef<HTMLDivElement>(null)
 const [, redraw] = useState(0)
 const [query, setQuery] = useState(''), [page, setPage] = useState(0)
 const [group, setGroup] = useState<string[] | null>(null)
 useEffect(() => {
  let frame = 0
  const update = () => { redraw(n => n + 1); frame = requestAnimationFrame(update) }
  frame = requestAnimationFrame(update)
  return () => cancelAnimationFrame(frame)
 }, [])
 const origin = getOrigin(), contacts = currentContacts(scanner, origin)
 const selected = selectedContact(scanner, origin)
 const previousSelection = useRef<string | null>(null)
 const [contactLost, setContactLost] = useState(false)
 useEffect(() => {
  if (selected) { previousSelection.current = selected.id; setContactLost(false) }
  else if (previousSelection.current) { setContactLost(true); previousSelection.current = null }
 }, [selected?.id])
 const dangers = new Map(navigationWarnings(getShip(), contacts).filter(w => w.severity === 'danger').map(w => [w.bodyId, w.message]))
 const availability = selected && contactCommandAvailability(getShip(), selected, canPilot())
 // The compact shell uses display:contents so its panel joins the HUD grid;
 // marker coordinates still cover the entire panoramic parent.
 const bounds = (compact ? root.current?.parentElement : root.current)?.getBoundingClientRect()
 const markers = perspectiveMarkers(contacts, geometry, origin, view.snapshot(), bounds?.width ?? 0, bounds?.height ?? 0)
 // A grouping menu belongs to the current visible group, never to stale IDs.
 const activeGroup = group && markers.find(m => group.includes(m.id) && m.ids.length === group.length && m.ids.every(id => group.includes(id)))
 const entries = contacts.filter(b => (!group || activeGroup?.ids.includes(b.id)) && b.name.toLowerCase().includes(query.toLowerCase()))
 const last = Math.max(0, Math.ceil(entries.length / 8) - 1), currentPage = Math.min(page, last)
 const choose = (id: string) => { selectContact(scanner, getOrigin(), id); redraw(n => n + 1) }
 return <div ref={root} className={compact ? 'panoramic-projection' : undefined} style={compact ? undefined : { position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'hidden' }}>
  {markers.map(marker => {
   const isSelected = !!selected && marker.ids.includes(selected.id)
   const destination = !!destinationId && marker.ids.includes(destinationId)
   const danger = marker.ids.map(id => dangers.get(id)).filter(Boolean).join('; ')
   const name = marker.ids.length === 1 ? contacts.find(b => b.id === marker.id)?.name : `Group of ${marker.ids.length} contacts`
   const description = [name, isSelected && `Selected: ${selected.name}`, destination && 'Landing destination', danger].filter(Boolean).join(' · ')
   return <button key={marker.id} data-sensor-marker className="sensor-marker" data-selected={isSelected} data-destination={destination} data-danger={!!danger} aria-pressed={isSelected} title={description} aria-description={description} aria-label={`Sensor contact ${marker.ids.length === 1 ? contacts.find(b => b.id === marker.id)?.name : `group of ${marker.ids.length}`}`}
   style={{ left: marker.x, top: marker.y }}
   onClick={() => {
    const live = perspectiveMarkers(currentContacts(scanner, getOrigin()), geometry, getOrigin(), view.snapshot(), (compact ? root.current!.parentElement! : root.current!).clientWidth, (compact ? root.current!.parentElement! : root.current!).clientHeight).find(m => m.id === marker.id)
    if (!live) return
    if (live.ids.length === 1) choose(live.id)
    else { setGroup(live.ids); setPage(0); setQuery(''); onBrowserChange?.(true) }
   }}>
    <svg viewBox="0 0 32 32" aria-hidden="true">
     <path className="marker-brackets" d="M4 11V4H11 M21 4H28V11 M28 21V28H21 M11 28H4V21"/>
     <path className="marker-core" d={destination ? 'M16 10 22 16 16 22 10 16Z' : 'M12 16H14 M18 16H20 M16 12V14 M16 18V20'}/>
     {marker.ids.length > 1 && <text className="marker-count" x={danger ? 14 : 16} y={danger ? 26 : 20} textAnchor="middle">{marker.ids.length}</text>}
    </svg>
    {!!danger && <span className="marker-danger" aria-hidden="true"><b>!</b></span>}
   </button>
  })}
  <section className={compact ? "panoramic-contacts" : undefined} aria-label="Detected contacts" style={compact ? undefined : { position: 'absolute', right: 8, top: 8, width: 240, padding: 12, color: 'white', background: '#102431ee', pointerEvents: 'auto' }}>
   {!compact && <details><summary>Observation help</summary><p>Look at turns only the camera once, preserving zoom and ship motion. Drag, W/A/S/D or wheel cancels it. Set direction turns only the ship without thrust; arrow keys cancel that turn. Grounded ships can observe other detected bodies, but cannot aim at their landing planet center. Typing and focused controls do not steer the view.</p></details>}
   {compact && <button className={`${buttonClass} contacts-toggle`} aria-expanded={browserOpen} aria-controls="scanned-browser" onClick={() => onBrowserChange?.(!browserOpen)}>Contacts (C) · {contacts.length}</button>}
   {browserOpen && <div data-ui-utility id="scanned-browser" className={compact ? "scanned-browser ui-panel ui-panel--utility" : undefined}>
   <label>Search contacts <input value={query} onChange={e => { setQuery(e.target.value); setPage(0) }}/></label>
   {group && <button className={buttonClass} onClick={() => { setGroup(null); setPage(0) }}>All contacts</button>}
   {group && !activeGroup && <p>Group no longer visible</p>}
   <ul>{entries.slice(currentPage * 8, currentPage * 8 + 8).map(body => <li key={body.id}><button className={buttonClass} aria-pressed={selected?.id === body.id} onClick={() => choose(body.id)}>{body.name}</button></li>)}</ul>
   <button className={buttonClass} disabled={currentPage === 0} onClick={() => setPage(currentPage - 1)}>Previous</button>
   <span> {currentPage + 1}/{last + 1} </span>
   <button className={buttonClass} disabled={currentPage === last} onClick={() => setPage(currentPage + 1)}>Next</button>
   </div>}
   {(!compact || selected || contactLost) && <div className={compact ? "selected-contact ui-panel ui-panel--utility" : "selected-contact"}><StatusMessage priority={contactLost ? 'error' : 'status'}>{selected ? `Selected: ${selected.name}` : contactLost ? 'Scanner contact lost · target actions unavailable.' : 'No selection'}</StatusMessage>
   {selected && availability && <>
    {availability.currentPlanet && <p>Current landing planet: {selected.name}</p>}
    <button className={buttonClass} disabled={!availability.lookAt} onClick={() => {
     const live = selectedContact(scanner, getOrigin())
     if (live) lookAtContact(scanner, getShip(), view, live.id)
    }}>Look at</button>
    {onSetDirection && <button className={buttonClass} disabled={!availability.setDirection} onClick={() => {
     const live = selectedContact(scanner, getOrigin())
     if (live && contactCommandAvailability(getShip(), live, canPilot()).setDirection) onSetDirection(live)
    }}>Set direction</button>}
   </>}
   </div>}
  </section>
 </div>
}
