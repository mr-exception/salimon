import { dot, type Vec3 } from './math.ts'
import { samplePhysicalSurface, WALK_RADIUS_M } from './surfacePhysics.ts'
import { relativeAnchor, rotate2, type Anchor, type Point2, type WorldFrames } from './topDownFrames.ts'

// Footprints are in metres in the entity's own frame and rendered from this data.
export interface Footprint { readonly halfWidth: number; readonly halfHeight: number; readonly surfaceEast?: Vec3 }
export interface Interaction { readonly label: string; readonly action: string; readonly reachM: number; readonly enabled?: boolean }
export interface WorldObject {
  readonly id: string
  readonly anchor: Anchor
  readonly footprint?: Footprint
  readonly interaction?: Interaction
}
export const PLAYER_RADIUS_M = WALK_RADIUS_M
/** Fixed body-space direction keeps physical footprint and sprite aligned when
 * the walking chart changes, including differently oriented cube faces. */
export function objectAngle(object: WorldObject, frames: WorldFrames): number {
  if (object.anchor.frame === frames.ship.id) return frames.ship.angle
  const east = object.footprint?.surfaceEast
  if (!east) return 0
  const chart = frames.surface?.chart ?? object.anchor.chart
  if (!chart) throw Error('Missing physical source chart')
  return Math.atan2(dot(east, chart.north), dot(east, chart.east))
}

export function allowedMove(anchor: Anchor, deltaPlanet: Point2, objects: readonly WorldObject[], frames: WorldFrames): number {
  let fraction = 1
  for (const object of objects) {
    if (!object.footprint || frames.ship.parent === null && object.anchor.frame !== anchor.frame) continue
    const angle = -objectAngle(object, frames)
    const origin = rotate2(relativeAnchor(anchor, object.anchor, frames), angle)
    const result = samplePhysicalSurface({ x: origin[0], y: origin[1], radiusM: PLAYER_RADIUS_M, motion: rotate2(deltaPlanet, angle) }, [
      { id: object.id, x: 0, y: 0, ...object.footprint },
    ])
    fraction = Math.min(fraction, result.moveFraction)
  }
  return fraction
}
export function interactionInReach(player: Anchor, object: WorldObject, frames: WorldFrames): boolean {
  return !(frames.ship.parent === null && player.frame !== object.anchor.frame) && !!object.interaction && object.interaction.enabled !== false
    && Math.hypot(...relativeAnchor(object.anchor, player, frames)) <= object.interaction.reachM + 1e-9
}
export function nearbyObject(player: Anchor, objects: readonly WorldObject[], frames: WorldFrames): WorldObject | undefined {
  return objects.filter(o => interactionInReach(player, o, frames)).sort((a, b) =>
    Math.hypot(...relativeAnchor(a.anchor, player, frames)) - Math.hypot(...relativeAnchor(b.anchor, player, frames))
    || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))[0]
}
