import { markerGuidance } from './fieldMarker.ts'
import { useMemo, useState } from 'react'
import { localLandmarks, scoutGuidance, surfaceAnchor } from './exploration.ts'
import { planetDefinition } from './planetDefinition.ts'
import { relativeAnchor, worldToView } from './topDownFrames.ts'
import type { ResourceSession } from './resourceSession.ts'
import type { TopDownState } from './topDownRuntime.ts'
import { Dialog } from './ui/primitives.tsx'

/** Only deliberate records get projected cues; no discovery by rendering. */
export function FieldMarkCues({state,session}: {state:TopDownState;session?:ResourceSession|null}) {
  if (!state.scoutLocation.binding) return null
  const definition = planetDefinition(state.scoutLocation.surface.bodyId,state.scoutLocation.surface.definition)
  return <g pointerEvents="none" fill="none" stroke="#f4ebd4" strokeWidth="2">
    {[...(session?.fieldMarks ?? []), ...(session?.snapshot.deployedMarkers ?? [])].filter(m => m.location.bodyId === definition.bodyId && m.definition.digest === definition.digest).map(mark => {
      try {
        const [x,y] = worldToView(relativeAnchor(surfaceAnchor(mark.location,state),state.camera.anchor,state.frames),state.frames)
        if (Math.hypot(x,y) > 40) return null
        return <g key={`${session?.snapshot.deployedMarkers.includes(mark) ? 'kit' : 'note'}:${mark.id}`} data-field-mark={mark.id} data-deployed-marker={session?.snapshot.deployedMarkers.includes(mark) ? mark.id : undefined} transform={`translate(${x*state.camera.pixelsPerM} ${y*state.camera.pixelsPerM})`}>
          {session?.snapshot.deployedMarkers.includes(mark) && <path d="M0 0V-22L15 -17L0 -12 M-6 5H6" stroke="#e9b76c" strokeWidth="3" />}
          <path d="M0 -13 L10 0 L0 13 L-10 0 Z M-4 0 L4 0 M0 -4 L0 4" stroke="#242c29" strokeWidth="5" />
          <path d="M0 -13 L10 0 L0 13 L-10 0 Z M-4 0 L4 0 M0 -4 L0 4" />
          <title>{mark.name} · {mark.provenance}</title>
        </g>
      } catch { return null }
    })}
  </g>
}
export function ExplorationHud({state,session,blocked,onChange,open,guiding,onDismiss,onGuidanceChange}: {state:TopDownState;session:ResourceSession;blocked:boolean;onChange:()=>void;open:boolean;guiding:boolean;onDismiss:()=>void;onGuidanceChange:()=>void}) {
  const [name,setName] = useState(''), [note,setNote] = useState('')
  const [guidedMarker, setGuidedMarker] = useState<string | null>(null)
  const deployed = session.snapshot.deployedMarkers
  const target = deployed.find(m => m.id === guidedMarker)
  const landmarks = useMemo(() => open ? localLandmarks(state) : [], [open,state])
  const guidance = guiding ? scoutGuidance(state,session.walkingFix) : null
  const mark = (id?:string) => {
    if (blocked) return
    session.world = state; session.markField(name,note,id); onChange()
  }
  return <aside className="exploration-hud" aria-label="Field exploration" style={!guiding && !target ? {width:0} : undefined}>
    {target && <><p data-marker-guidance role="status">{markerGuidance(state, target)}</p><button onClick={() => setGuidedMarker(null)}>Hide marker guidance</button></>}
    {guiding && <button onClick={event => {onGuidanceChange();event.currentTarget.blur()}}>Hide Scout guidance</button>}
    {guidance && <p data-scout-guidance data-fix={String(guidance.available)}>{guidance.text}</p>}
    <Dialog open={open} title="Field notebook" onDismiss={onDismiss}>
      <p>Inspect a nearby physical feature or mark where you stand. Knowledge is added only by your action.</p>
      <p>Field notes save with your expedition. Up to 256 deliberate marks are retained.</p>
      <label>Name (optional)<input value={name} maxLength={48} onChange={event=>setName(event.target.value)} /></label>
      <label>Short note (optional)<input value={note} maxLength={160} onChange={event=>setNote(event.target.value)} /></label>
      <button disabled={blocked} onClick={()=>mark()}>Mark current location</button>
      <ul aria-label="Nearby geological features">{landmarks.map(l => <li key={l.id} data-landmark={l.id}>
        <button disabled={blocked} onClick={()=>mark(l.id)}>Inspect {l.label}</button>
        <small>Nearby geological feature · personally inspected when recorded</small>
      </li>)}</ul>
      {!landmarks.length && <p>No geological feature within the 12 m inspection range.</p>}
      <p role="status">{session.error || session.feedback}</p>
      <h3>Deployed markers ({deployed.length} / 256)</h3>
      <p>Each marker holds one fabricated 1 kg kit on the surface. No reclaim. Select a marker for local guidance; use the Name field above to rename it for free.</p>
      <ul aria-label="Deployed markers">{deployed.map(m => <li key={m.id}>
        <strong>{m.name}</strong> · Body: {m.location.bodyId}
        <button onClick={() => setGuidedMarker(m.id)}>Guide to {m.name}</button>
        <button disabled={blocked || !name.trim()} onClick={() => {session.renameMarker(m.id, name, session.snapshot.revision); onChange()}}>Rename {m.name}</button>
      </li>)}</ul>
      <h3>Deliberately recorded places ({session.fieldMarks.length})</h3>
      <ul aria-label="Known places">{session.fieldMarks.map(m=><li key={m.id} data-known-mark={m.id}>
        <strong>{m.name}</strong> · {m.provenance.replaceAll('-', ' ')}<p>{m.note}</p><small>Body: {m.location.bodyId}</small>
      </li>)}</ul>
    </Dialog>
  </aside>
}
