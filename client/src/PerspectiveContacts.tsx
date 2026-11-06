import { useEffect, useRef, useState } from 'react'
import type { Position } from './coordinates.ts'
import type { ObservationCameraState } from './observationCamera.ts'
import type { Body } from './sampleUniverse.ts'
import type { Scanner } from './scanner.ts'
import { currentContacts, perspectiveMarkers, selectContact, selectedContact } from './perspectiveContacts.ts'

/** DOM companion to ObservationCamera + OpticalSky. Parent owns scanner ticks,
 * physical origin and optional flight action. No optical raycaster/hit meshes.
 * Every handler rechecks live scanner authority, including stale DOM events.
 */
export function PerspectiveContacts({ scanner, geometry, view, getOrigin, onSetDirection }: {
 scanner: Scanner; geometry: readonly Body[]; view: ObservationCameraState; getOrigin: () => Position
 onSetDirection?: (body: Body) => void
}) {
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
 const bounds = root.current?.getBoundingClientRect()
 const markers = perspectiveMarkers(contacts, geometry, origin, view.snapshot(), bounds?.width ?? 0, bounds?.height ?? 0)
 // A grouping menu belongs to the current visible group, never to stale IDs.
 const activeGroup = group && markers.find(m => group.includes(m.id) && m.ids.length === group.length && m.ids.every(id => group.includes(id)))
 const entries = contacts.filter(b => (!group || activeGroup?.ids.includes(b.id)) && b.name.toLowerCase().includes(query.toLowerCase()))
 const last = Math.max(0, Math.ceil(entries.length / 8) - 1), currentPage = Math.min(page, last)
 const choose = (id: string) => { selectContact(scanner, getOrigin(), id); redraw(n => n + 1) }
 return <div ref={root} style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'hidden' }}>
  {markers.map(marker => <button key={marker.id} aria-label={`Sensor contact ${marker.ids.length === 1 ? contacts.find(b => b.id === marker.id)?.name : `group of ${marker.ids.length}`}`}
   style={{ position: 'absolute', left: marker.x, top: marker.y, transform: 'translate(-50%, -50%)', width: 32, height: 32, borderRadius: '50%', color: '#9feaff', background: '#102431aa', border: '1px solid #9feaff', pointerEvents: 'auto' }}
   onClick={() => {
    const live = perspectiveMarkers(currentContacts(scanner, getOrigin()), geometry, getOrigin(), view.snapshot(), root.current!.clientWidth, root.current!.clientHeight).find(m => m.id === marker.id)
    if (!live) return
    if (live.ids.length === 1) choose(live.id)
    else { setGroup(live.ids); setPage(0); setQuery('') }
   }}>{marker.ids.length === 1 ? '+' : marker.ids.length}</button>)}
  <section aria-label="Detected contacts" style={{ position: 'absolute', right: 8, top: 8, width: 240, padding: 12, color: 'white', background: '#102431ee', pointerEvents: 'auto' }}>
   <label>Search contacts <input value={query} onChange={e => { setQuery(e.target.value); setPage(0) }}/></label>
   {group && <button onClick={() => { setGroup(null); setPage(0) }}>All contacts</button>}
   {group && !activeGroup && <p>Group no longer visible</p>}
   <ul>{entries.slice(currentPage * 8, currentPage * 8 + 8).map(body => <li key={body.id}><button aria-pressed={selected?.id === body.id} onClick={() => choose(body.id)}>{body.name}</button></li>)}</ul>
   <button disabled={currentPage === 0} onClick={() => setPage(currentPage - 1)}>Previous</button>
   <span> {currentPage + 1}/{last + 1} </span>
   <button disabled={currentPage === last} onClick={() => setPage(currentPage + 1)}>Next</button>
   <p role="status">{selected ? `Selected: ${selected.name}` : 'No selection'}</p>
   {selected && onSetDirection && <button onClick={() => { const live = selectedContact(scanner, getOrigin()); if (live) onSetDirection(live) }}>Set direction</button>}
  </section>
 </div>
}
