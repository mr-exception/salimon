import { planetDefinition } from './planetDefinition.ts'
import { validateJournal } from './expeditionJournal.ts'
import { MAX_FIELD_MARKS, playerSurface, surfaceAnchor, surfaceDistance, type FieldMark } from './exploration.ts'
import { relativeAnchor, worldToView } from './topDownFrames.ts'
import { PLAYER_ID, type TopDownState } from './topDownRuntime.ts'
import type { SiteStates } from './surfaceSite.ts'

export const MARKER_GUIDANCE_RANGE_M = 384
/** Deployed kits are separate from free notebook observations. Their count is
 * the durable manufactured-item receipt; they cannot be reclaimed or deleted. */
export function validateDeployedMarkers(value: unknown, sites: SiteStates): readonly FieldMark[] {
  if (!Array.isArray(value) || value.length > MAX_FIELD_MARKS) throw Error('Invalid deployed marker capacity')
  const marks = validateJournal({ marks: value, visits: [] }, sites).marks
  if (marks.some(m => m.landmarkId !== null || m.note !== '')) throw Error('Invalid deployed marker')
  return marks
}
export function markerGuidance(world: TopDownState, marker: FieldMark): string {
  const player = playerSurface(world)
  if (!player || world.scoutLocation.ship.mode === 'destroyed' || marker.definition.digest !== planetDefinition(world.scoutLocation.surface.bodyId, world.scoutLocation.surface.definition).digest || surfaceDistance(player, marker.location) > MARKER_GUIDANCE_RANGE_M) return `${marker.name}: guidance unavailable (same surface, within 384 m only).`
  const anchor = world.entities.find(e => e.id === PLAYER_ID)!.anchor
  const vector = worldToView(relativeAnchor(surfaceAnchor(marker.location, world), anchor, world.frames), world.frames)
  const distance = Math.hypot(...vector)
  if (distance < 1) return `${marker.name}: marker here (within 1 m).`
  const sectors = ['↑ ahead', '↗ ahead-right', '→ right', '↘ behind-right', '↓ behind', '↙ behind-left', '← left', '↖ ahead-left']
  const sector = (Math.round(Math.atan2(vector[0], -vector[1]) / (Math.PI / 4)) + 8) % 8
  return `${marker.name}: ${sectors[sector]} · ${Math.round(distance)} m · local marker guidance.`
}
