import { relativeM } from './coordinates.ts'
import { SCOUT_ANCHORS, SCOUT_ENTRY_HALF_WIDTH_M } from './scoutLayout.ts'
import { convertAnchor, type Anchor, type WorldFrames } from './topDownFrames.ts'
import { PLAYER_RADIUS_M } from './topDownInteraction.ts'

// Ownership changes only across the open aft threshold, after collision sweeps.
// Merely approaching another hull edge must never board the ship.
export function crossScoutEntry(before: Anchor, after: Anchor, frames: WorldFrames): Anchor {
  if (frames.ship.parent === null) return after
  const localBefore = convertAnchor(before, frames.ship.id, frames)
  const localAfter = convertAnchor(after, frames.ship.id, frames)
  const [ax, rawAy] = relativeM(localBefore.point, SCOUT_ANCHORS.entry!.point)
  const [bx, rawBy] = relativeM(localAfter.point, SCOUT_ANCHORS.entry!.point)
  // One nanometre avoids round-trip transform noise at the exact plane.
  const ay = Math.abs(rawAy) < 1e-9 ? 0 : rawAy, by = Math.abs(rawBy) < 1e-9 ? 0 : rawBy
  const entering = after.frame === frames.planet.id && ay >= 0 && by < 0
  const leaving = after.frame === frames.ship.id && ay <= 0 && by > 0
  if (!entering && !leaving) return after
  const crossingX = ax + (bx - ax) * (-ay / (by - ay))
  if (Math.abs(crossingX) > SCOUT_ENTRY_HALF_WIDTH_M - PLAYER_RADIUS_M) return after
  return convertAnchor(after, entering ? frames.ship.id : frames.planet.id, frames)
}

// In-memory hooks for a future authoritative reconnect adapter. Keep the split
// Position (including bigint cells) and frame identity, never a flattened world
// Number or camera position. Collision/spawn validation belongs to that adapter.
export interface PlayerLocation { readonly anchor: Anchor; readonly facing: number }
export function copyPlayerLocation(location: PlayerLocation): PlayerLocation {
  return { anchor: { frame: location.anchor.frame, point: {
    cell: [...location.anchor.point.cell], offsetM: [...location.anchor.point.offsetM],
  } }, facing: location.facing }
}
