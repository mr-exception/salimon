import { useEffect, useState } from 'react'
import { createFieldMark } from './exploration.ts'
import type { ResourceSession } from './resourceSession.ts'
import { MARKER_KIT_ID } from './scoutFabricator.ts'
import { Button, EquipmentDialog } from './ui/primitives.tsx'

export function MarkerDeployment({ session, blocked, onChange }: { session: ResourceSession; blocked: boolean; onChange: () => void }) {
  const [pending, setPending] = useState<{ positionId: string; count: number } | null>(null)
  const [name, setName] = useState('Field marker')
  const denial = session.markerDeploymentDenial()
  // Time continues. Bind the review to its position and kit count; the domain
  // uses the current revision and the existing storage compare-and-replace gate.
  useEffect(() => {
    if (!pending) return
    const cancel = () => setPending(null)
    window.addEventListener('blur', cancel)
    if (blocked || denial) cancel()
    else if (createFieldMark(session.world, '', '').id !== pending.positionId || (session.snapshot.inventory.items[MARKER_KIT_ID] ?? 0) !== pending.count) cancel()
    return () => window.removeEventListener('blur', cancel)
  }, [pending, blocked, denial, session, session.world, session.snapshot])
  return <>
    <Button disabledReason={blocked ? 'Deployment unavailable now.' : denial ?? undefined} onClick={() => {
      session.cancel(); setPending({ positionId: createFieldMark(session.world, '', '').id, count: session.snapshot.inventory.items[MARKER_KIT_ID] ?? 0 })
    }}>Deploy field marker</Button>
    {denial && <p>{denial}</p>}
    <EquipmentDialog open={!!pending} title="Deploy field marker" onDismiss={() => setPending(null)}>
      <p>Place one 1 kg kit at your current outdoor position. The persistent marker gives local direction and range within 384 m on this surface. It cannot be reclaimed. Notebook marks remain free.</p>
      <label>Marker name<input maxLength={48} value={name} onChange={e => setName(e.target.value)} /></label>
      <Button disabledReason={blocked ? 'Deployment unavailable now.' : denial ?? (!name.trim() ? 'Enter a marker name.' : undefined)} onClick={() => {
        if (!pending) return
        session.deployMarker(name, session.snapshot.revision, pending.positionId); setPending(null); onChange()
      }}>Confirm deployment</Button>
      <Button variant="secondary" onClick={() => setPending(null)}>Cancel deployment</Button>
    </EquipmentDialog>
  </>
}
