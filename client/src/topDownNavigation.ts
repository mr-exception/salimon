import { relativeM, translate, ORIGIN } from './coordinates.ts'
import { probeGeology } from './planetGeology.ts'
import { localToSurface, surfaceRegion } from './surfaceCoordinates.ts'
import { allowedMove, objectAngle, PLAYER_RADIUS_M } from './topDownInteraction.ts'
import { convertAnchor, inPlanet, relativeAnchor, rotate2, viewToWorld, type Anchor, type Point2 } from './topDownFrames.ts'
import type { TopDownState } from './topDownRuntime.ts'

export interface GroundCommand {
  readonly destination: Anchor
  readonly waypoints: readonly Anchor[]
  readonly status: 'moving' | 'arrived' | 'failed' | 'cancelled'
}
export const ROUTE_RANGE_M = 48
const MAX_NODES = 512, MAX_EDGES = 4096, MAX_PROBES = 16384

/** Inverse of the live SVG projection. Caller first applies inverse screen CTM. */
export function groundFromView(state: TopDownState, svg: Point2): Anchor {
  const delta = viewToWorld([svg[0] / state.camera.pixelsPerM, svg[1] / state.camera.pixelsPerM], state.frames)
  const local = rotate2(delta, state.camera.anchor.frame === 'ship:scout' ? -state.frames.ship.angle : 0)
  return { ...state.camera.anchor, point: translate(state.camera.anchor.point, [...local, 0]) }
}
export function interactiveAt(state: TopDownState, anchor: Anchor): string | null {
  return [...state.entities].reverse().find(object => {
    if (!object.interaction || state.frames.ship.parent === null && object.anchor.frame !== anchor.frame) return false
    const [x, y] = rotate2(relativeAnchor(anchor, object.anchor, state.frames), -objectAngle(object, state.frames))
    return Math.abs(x) <= (object.footprint?.halfWidth ?? .65) && Math.abs(y) <= (object.footprint?.halfHeight ?? .3)
  })?.id ?? null
}
/** Shared walking authority, also used to validate every planned edge. */
export function terrainMovementFraction(state: TopDownState, anchor: Anchor, delta: Point2): number {
  if (anchor.frame === 'planet:fixture' && state.scoutLocation.binding && state.scoutLocation.surface.definition?.version === 2) {
    const address = state.scoutLocation.surface, region = state.frames.surface?.chart ?? surfaceRegion(address, address.definition)
    const [x, y] = relativeM(inPlanet(anchor, state.frames), ORIGIN)
    return probeGeology(localToSurface(region, [x, y]), address.definition!, PLAYER_RADIUS_M, delta, undefined, region).moveFraction
  }
  return 1
}
export function movementFraction(state: TopDownState, anchor: Anchor, delta: Point2): number {
  return Math.min(allowedMove(anchor, delta, state.entities, state.frames), terrainMovementFraction(state, anchor, delta))
}
export function groundPositionClear(state: TopDownState, anchor: Anchor, margin = 0): boolean {
  for (const object of state.entities) {
    if (!object.footprint || state.frames.ship.parent === null && object.anchor.frame !== anchor.frame) continue
    const [x, y] = rotate2(relativeAnchor(anchor, object.anchor, state.frames), -objectAngle(object, state.frames))
    if (Math.abs(x) < object.footprint.halfWidth + PLAYER_RADIUS_M + margin && Math.abs(y) < object.footprint.halfHeight + PLAYER_RADIUS_M + margin) return false
  }
  // A short sweep also tests the conservative terrain walking margin.
  return movementFraction(state, anchor, [.001, 0]) === 1
}
const clearPoint = (state: TopDownState, anchor: Anchor) => groundPositionClear(state, anchor, .025)
function edgeClear(state: TopDownState, from: Anchor, to: Anchor, budget: { probes: number }): boolean {
  // Probe exterior terrain even when a candidate starts inside the Scout.
  const probe = state.frames.ship.parent === null ? from : convertAnchor(from, 'planet:fixture', state.frames)
  const delta = relativeAnchor(to, from, state.frames), length = Math.hypot(...delta)
  // Keep geology's bounded physical probe contract, including long direct edges.
  const count = Math.max(1, Math.ceil(length / 1))
  for (let i = 0; i < count; i++) {
    if (++budget.probes > MAX_PROBES) return false
    const d: Point2 = [delta[0] / count, delta[1] / count]
    const local = rotate2([d[0] * i, d[1] * i], probe.frame === 'ship:scout' ? -state.frames.ship.angle : 0)
    if (movementFraction(state, { ...probe, point: translate(probe.point, [...local, 0]) }, d) < 1) return false
  }
  return true
}

/** Bounded visibility graph over conservative physical footprint corners. Unlike
 * a coarse grid this preserves narrow legal passages between interior fixtures.
 * Stable object/corner order and Dijkstra ties make plans deterministic. */
export function planGroundRoute(state: TopDownState, destination: Anchor): readonly Anchor[] | null {
  const start = state.entities.find(e => e.id === 'person:player')!.anchor
  if (state.frames.ship.parent === null && destination.frame !== 'ship:scout') return null
  const delta = relativeAnchor(destination, start, state.frames)
  if (!delta.every(Number.isFinite) || Math.hypot(...delta) > ROUTE_RANGE_M) return null
  if (state.frames.ship.parent === null) {
    const [x, y] = relativeM(destination.point, ORIGIN)
    if (Math.abs(x) > 2.5 - PLAYER_RADIUS_M || y > 5.5 - PLAYER_RADIUS_M || y < -6 + PLAYER_RADIUS_M) return null
  }
  const frame = state.frames.ship.parent === null ? 'ship:scout' : 'planet:fixture'
  const endpoint = state.frames.ship.parent === null ? destination : convertAnchor(destination, frame, state.frames)
  if (!clearPoint(state, endpoint)) return null
  const budget = { probes: 0 }
  if (edgeClear(state, start, destination, budget)) return [destination]
  const nodes: Anchor[] = [start, destination]
  for (const object of [...state.entities].sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0)) {
    if (!object.footprint || state.frames.ship.parent === null && object.anchor.frame !== frame) continue
    const centre = state.frames.ship.parent === null ? object.anchor : convertAnchor(object.anchor, frame, state.frames)
    const offset = relativeAnchor(centre, start, state.frames)
    if (offset.some((n, i) => n < Math.min(0, delta[i]!) - 8 || n > Math.max(0, delta[i]!) + 8)) continue
    for (const x of [-1, 1]) for (const y of [-1, 1]) {
      const local = rotate2([x * (object.footprint.halfWidth + PLAYER_RADIUS_M + .04), y * (object.footprint.halfHeight + PLAYER_RADIUS_M + .04)], frame === 'ship:scout' ? 0 : objectAngle(object, state.frames))
      const anchor = { ...centre, point: translate(centre.point, [...local, 0]) }
      if (clearPoint(state, anchor)) nodes.push(anchor)
      if (nodes.length > MAX_NODES) return null
    }
  }
  // Terrain has no rectangular corner publication. Add a bounded local lattice
  // only when the direct corridor intersects an authoritative slope blocker.
  if (frame === 'planet:fixture') {
    const origin = convertAnchor(start, frame, state.frames), length = Math.hypot(...delta)
    const direction: Point2 = length ? [delta[0] / length, delta[1] / length] : [1, 0]
    let terrainBlocked = false
    for (let d = 0; d < length; d++) {
      const anchor = { ...origin, point: translate(origin.point, [direction[0] * d, direction[1] * d, 0]) }
      if (terrainMovementFraction(state, anchor, direction) < 1) { terrainBlocked = true; break }
    }
    if (terrainBlocked) for (let d = 0; d <= length + 2; d += 2) for (let side = -8; side <= 8; side += 2) {
      const anchor = { ...origin, point: translate(origin.point, [direction[0] * d - direction[1] * side, direction[1] * d + direction[0] * side, 0]) }
      if (clearPoint(state, anchor)) nodes.push(anchor)
      if (nodes.length > MAX_NODES) return null
    }
  }
  const costs = nodes.map(() => Infinity), previous = nodes.map(() => -1), closed = new Set<number>()
  costs[0] = 0
  let edges = 0
  while (closed.size < nodes.length) {
    let best = -1
    for (let i = 0; i < nodes.length; i++) if (!closed.has(i) && Number.isFinite(costs[i]) && (best < 0 || costs[i]! < costs[best]!)) best = i
    if (best < 0) return null
    if (best === 1) {
      const route: Anchor[] = []
      for (let n = 1; n !== 0; n = previous[n]!) route.unshift(nodes[n]!)
      return route
    }
    closed.add(best)
    for (let i = 1; i < nodes.length; i++) {
      if (closed.has(i)) continue
      const cost = costs[best]! + Math.hypot(...relativeAnchor(nodes[i]!, nodes[best]!, state.frames))
      if (cost >= costs[i]!) continue
      if (++edges > MAX_EDGES || budget.probes > MAX_PROBES) return null
      if (edgeClear(state, nodes[best]!, nodes[i]!, budget)) { costs[i] = cost; previous[i] = best }
    }
  }
  return null
}
export function cancelGroundMovement(state: TopDownState): TopDownState {
  return state.groundCommand?.status === 'moving' ? { ...state, groundCommand: { ...state.groundCommand, status: 'cancelled', waypoints: [] }, player: { ...state.player, mode: state.player.mode === 'walking' ? 'standing' : state.player.mode }, feedback: 'Movement cancelled.' } : state
}
export function requestGroundMovement(state: TopDownState, destination: Anchor): TopDownState {
  state = cancelGroundMovement(state)
  if (state.player.mode === 'seated' || state.cockpit.view !== 'world' || state.scoutLocation.ship.mode === 'destroyed') return state
  const waypoints = planGroundRoute(state, destination)
  return { ...state, groundCommand: { destination, waypoints: waypoints ?? [], status: waypoints ? 'moving' : 'failed' }, player: { ...state.player, mode: 'standing', targetId: null }, feedback: waypoints ? 'Moving to destination. Escape to cancel.' : 'Destination unreachable or outside the local route range.' }
}
