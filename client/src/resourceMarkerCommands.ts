import { MARKER_KIT_ID } from './scoutFabricator.ts'
import { createFieldMark, recordFieldMark } from './exploration.ts'
import { survivalPolicy } from './survival.ts'
import { removeCountItem } from './resourceInventory.ts'
import { RESOURCE_CATALOG, type ResourceSave } from './resourceCatalog.ts'
import type { ResourceSession } from './resourceSession.ts'
type CommitResourceSave = (next: ResourceSave) => boolean
type SessionState = Pick<ResourceSession, 'error' | 'feedback' | 'fieldMarks' | 'hazards' | 'itemCatalog' | 'snapshot' | 'survival' | 'world'>

export function markField(session: SessionState, commit: CommitResourceSave, name: string, note: string, landmarkId?: string): boolean {
  if (session.error || session.survival.health === 0) return false
  try {
    const marks = recordFieldMark(session.fieldMarks, createFieldMark(session.world, name, note, landmarkId))
    if (!commit({ ...session.snapshot, survival: session.survival, hazards: session.hazards, journal: { ...session.snapshot.journal, marks } })) return false
    session.feedback = 'Field mark saved.'
    return true
  } catch (error) { session.feedback = (error as Error).message; return false }
}

export function markerDeploymentDenial(session: SessionState): string | null {
  if (session.error) return session.error
  if (!survivalPolicy(session.survival).canWork || session.world.scoutLocation.ship.mode === 'destroyed') return 'Marker deployment requires a living expedition able to work.'
  if (session.world.cockpit.view !== 'world' || session.world.player.mode === 'seated') return 'Leave the cockpit before deploying.'
  try {
    const mark = createFieldMark(session.world, '', '')
    if (session.snapshot.deployedMarkers.some(m => m.id === mark.id)) return 'A marker is already deployed here.'
    if (session.snapshot.deployedMarkers.length >= 256) return 'Deployed marker capacity full (256).'
    if (!(session.snapshot.inventory.items[MARKER_KIT_ID] ?? 0)) return 'Carry a fabricated field marker kit first.'
    return null
  } catch { return 'Deploy only at your current outdoor position.' }
}

export function deployMarker(session: SessionState, commit: CommitResourceSave, name: string, revision: number, positionId: string): boolean {
  try {
    if (markerDeploymentDenial(session)) throw Error(markerDeploymentDenial(session)!)
    if (!Number.isSafeInteger(revision) || revision !== session.snapshot.revision) throw Error('Stale marker revision')
    const mark = createFieldMark(session.world, name || 'Field marker', '')
    if (mark.id !== positionId) throw Error('Deployment position changed; review again.')
    const inventory = removeCountItem(session.snapshot.inventory, MARKER_KIT_ID, 1, RESOURCE_CATALOG, session.itemCatalog)
    if (!commit({ ...session.snapshot, inventory, deployedMarkers: [...session.snapshot.deployedMarkers, mark] })) return false
    session.feedback = 'Field marker deployed. One kit placed on the surface.'
    return true
  } catch (error) { session.feedback = (error as Error).message; return false }
}

export function renameMarker(session: SessionState, commit: CommitResourceSave, id: string, name: string, revision: number): boolean {
  if (session.error || session.survival.health === 0) return false
  try {
    if (!Number.isSafeInteger(revision) || revision !== session.snapshot.revision) throw Error('Stale marker revision')
    if (typeof name !== 'string' || !name.trim() || name.length > 48 || /[\u0000-\u001f]/.test(name)) throw Error('Use a marker name of 1–48 characters.')
    if (!session.snapshot.deployedMarkers.some(m => m.id === id)) throw Error('Unknown deployed marker')
    return commit({ ...session.snapshot, deployedMarkers: session.snapshot.deployedMarkers.map(m => m.id === id ? { ...m, name: name.trim() } : m) })
  } catch (error) { session.feedback = (error as Error).message; return false }
}
