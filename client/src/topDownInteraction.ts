import { relativeAnchor, rotate2, type Anchor, type Point2, type WorldFrames } from './topDownFrames.ts'

// Footprints are in metres in the entity's own frame and rendered from this data.
export interface Footprint { readonly halfWidth: number; readonly halfHeight: number }
export interface Interaction { readonly label: string; readonly action: string; readonly reachM: number; readonly enabled?: boolean }
export interface WorldObject {
  readonly id: string
  readonly anchor: Anchor
  readonly footprint?: Footprint
  readonly interaction?: Interaction
}
export const PLAYER_RADIUS_M = .46
const angleOf = (anchor: Anchor, frames: WorldFrames) => anchor.frame === frames.ship.id ? frames.ship.angle : 0

// Swept point versus a conservatively expanded box. It cannot tunnel even if a
// caller requests a long move. Tangent/away motion at a boundary remains legal.
function hitFraction(origin: Point2, delta: Point2, box: Footprint): number {
  let enter = -Infinity, leave = Infinity
  for (let axis = 0; axis < 2; axis++) {
    const extent = (axis === 0 ? box.halfWidth : box.halfHeight) + PLAYER_RADIUS_M
    if (Math.abs(delta[axis]) < 1e-12) {
      if (Math.abs(origin[axis]) >= extent - 1e-10) return 1
    } else {
      const a = (-extent - origin[axis]) / delta[axis], b = (extent - origin[axis]) / delta[axis]
      enter = Math.max(enter, Math.min(a, b)); leave = Math.min(leave, Math.max(a, b))
    }
  }
  return enter < leave && leave > 1e-10 && enter >= -1e-10 && enter < 1 ? Math.max(0, enter - 1e-9) : 1
}
export function allowedMove(anchor: Anchor, deltaPlanet: Point2, objects: readonly WorldObject[], frames: WorldFrames): number {
  let fraction = 1
  for (const object of objects) {
    if (!object.footprint || frames.ship.parent === null && object.anchor.frame !== anchor.frame) continue
    const angle = -angleOf(object.anchor, frames)
    const origin = rotate2(relativeAnchor(anchor, object.anchor, frames), angle)
    fraction = Math.min(fraction, hitFraction(origin, rotate2(deltaPlanet, angle), object.footprint))
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
