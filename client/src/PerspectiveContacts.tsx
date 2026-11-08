import { useEffect, useRef, useState } from 'react'
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
export function PerspectiveContacts({ scanner, geometry, view, getOrigin, getShip, canPilot = () => true, onSetDirection, compact = false, browserOpen = true, onBrowserChange }: {
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
 const availability = selected && contactCommandAvailability(getShip(), selected, canPilot())
 const bounds = root.current?.getBoundingClientRect()
 const markers = perspectiveMarkers(contacts, geometry, origin, view.snapshot(), bounds?.width ?? 0, bounds?.height ?? 0)
 // A grouping menu belongs to the current visible group, never to stale IDs.
 const activeGroup = group && markers.find(m => group.includes(m.id) && m.ids.length === group.length && m.ids.every(id => group.includes(id)))
 const entries = contacts.filter(b => (!group || activeGroup?.ids.includes(b.id)) && b.name.toLowerCase().includes(query.toLowerCase()))
 const last = Math.max(0, Math.ceil(entries.length / 8) - 1), currentPage = Math.min(page, last)
 const choose = (id: string) => { selectContact(scanner, getOrigin(), id); redraw(n => n + 1) }
 return <div ref={root} style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'hidden' }}>
  {markers.map(marker => <button key={marker.id} data-sensor-marker aria-label={`Sensor contact ${marker.ids.length === 1 ? contacts.find(b => b.id === marker.id)?.name : `group of ${marker.ids.length}`}`}
   style={{ position: 'absolute', left: marker.x, top: marker.y, transform: 'translate(-50%, -50%)', width: 32, height: 32, borderRadius: '50%', color: compact ? 'var(--ui-active)' : '#9feaff', background: compact ? 'var(--ui-void)' : '#102431aa', border: compact ? '1px solid var(--ui-active)' : '1px solid #9feaff', pointerEvents: 'auto' }}
   onClick={() => {
    const live = perspectiveMarkers(currentContacts(scanner, getOrigin()), geometry, getOrigin(), view.snapshot(), root.current!.clientWidth, root.current!.clientHeight).find(m => m.id === marker.id)
    if (!live) return
    if (live.ids.length === 1) choose(live.id)
    else { setGroup(live.ids); setPage(0); setQuery(''); onBrowserChange?.(true) }
   }}>{marker.ids.length === 1 ? '+' : marker.ids.length}</button>)}
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
   <div className={compact ? "selected-contact ui-panel ui-panel--utility" : "selected-contact"}><StatusMessage priority={contactLost ? 'error' : 'status'}>{selected ? `Selected: ${selected.name}` : contactLost ? 'Scanner contact lost · target actions unavailable.' : 'No selection'}</StatusMessage>
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
   </div>
  </section>
 </div>
}
