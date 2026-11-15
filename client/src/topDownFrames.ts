import { dot, add, scale, cross } from './math.ts'
import { localToSurface, surfaceToLocal, type SurfaceRegion } from './surfaceCoordinates.ts'
import { ORIGIN, relativeM, translate, type Position } from './coordinates.ts'

export type Point2 = readonly [number, number]
export type FrameId = 'planet:fixture' | 'ship:scout'
export interface Anchor { readonly frame: FrameId; readonly point: Position; readonly chart?: SurfaceRegion }
export interface WorldFrames {
  readonly surface?: { readonly landing: SurfaceRegion; readonly chart: SurfaceRegion; readonly source: WorldFrames }
  readonly planet: { readonly id: 'planet:fixture'; readonly universeOrigin: Position; readonly angle: number; readonly basis?: { readonly east: import('./math.ts').Vec3; readonly north: import('./math.ts').Vec3; readonly up: import('./math.ts').Vec3 } }
  readonly ship: { readonly id: 'ship:scout'; readonly parent: 'planet:fixture' | null; readonly origin: Position; readonly angle: number }
}
export const point2 = (x: number, y: number): Position => translate(ORIGIN, [x, y, 0])
export function rotate2([x, y]: Point2, angle: number): Point2 {
  if (![x, y, angle].every(Number.isFinite)) throw new RangeError('Expected finite planar transform')
  const c = Math.cos(angle), s = Math.sin(angle)
  return [c * x - s * y, s * x + c * y]
}
function planar(point: Position): Point2 {
  const [x, y] = relativeM(point, ORIGIN)
  return [x, y]
}
// Resolve only to the common planet frame. Never add the astronomical universe
// origin to small local coordinates, or rotate flattened astronomical Numbers.
export function inPlanet(anchor: Anchor, frames: WorldFrames): Position {
  if (frames.ship.parent === null && anchor.frame === frames.planet.id) throw new Error('Planetary frame unavailable in flight')
  if (anchor.frame === frames.planet.id) {
    if (!frames.surface || (anchor.chart ?? frames.surface.landing).id === frames.surface.chart.id) return anchor.point
    const location = localToSurface(anchor.chart ?? frames.surface.landing, planar(anchor.point))
    // Back-side landmarks are never folded into the visible walking chart.
    if (dot(location.radialM, frames.surface.chart.up) <= 0) return point2(4*Math.hypot(...location.radialM), 0)
    return point2(...surfaceToLocal(location, frames.surface.chart))
  }
  if (frames.ship.parent === null) throw new Error('Planetary frame unavailable in flight')
  const [x, y] = rotate2(planar(anchor.point), frames.ship.angle)
  return translate(frames.ship.origin, [x, y, 0])
}
export function convertAnchor(anchor: Anchor, frame: FrameId, frames: WorldFrames): Anchor {
  if (frames.ship.parent === null && (anchor.frame === frames.planet.id || frame === frames.planet.id)) throw new Error('Planetary frame unavailable in flight')
  if (anchor.frame === frame && (frame === 'ship:scout' || !frames.surface)) return anchor
  if (frames.ship.parent === null) throw new Error('Planetary frame unavailable in flight')
  const planetPoint = inPlanet(anchor, frames)
  if (frame === frames.planet.id) return { frame, point: planetPoint, ...(frames.surface ? {chart: frames.surface.chart} : {}) }
  const [x, y] = relativeM(planetPoint, frames.ship.origin)
  const [localX, localY] = rotate2([x, y], -frames.ship.angle)
  return { frame, point: point2(localX, localY) }
}
// Returns planet-axis metres, including when both points are ship-local.
export function relativeAnchor(target: Anchor, origin: Anchor, frames: WorldFrames): Point2 {
  if (frames.ship.parent === null && (target.frame === frames.planet.id || origin.frame === frames.planet.id)) throw new Error('Planetary frame unavailable in flight')
  if (target.frame === origin.frame && (target.frame === frames.ship.id || !frames.surface)) {
    const [x, y] = relativeM(target.point, origin.point)
    return target.frame === frames.ship.id ? rotate2([x, y], frames.ship.angle) : [x, y]
  }
  const [x, y] = relativeM(inPlanet(target, frames), inPlanet(origin, frames))
  return [x, y]
}

// Universal 2D presentation basis: the active ship's bow is always screen-up.
// Domain anchors and collision remain in planet/ship frames; only presentation
// and its inverse input mapping rotate. Ownership handoff never changes basis.
export const worldToView = (direction: Point2, frames: WorldFrames): Point2 => rotate2(direction, -frames.ship.angle)
export const viewToWorld = (direction: Point2, frames: WorldFrames): Point2 => rotate2(direction, frames.ship.angle)

/** Reproject only the small walking chart. The landed ship/site remain immutable.
 * Distant back-side landmarks stay distant; they cannot become ghost doors. */
export function walkingFrames(source: WorldFrames, landing: SurfaceRegion, chart: SurfaceRegion, previous?: WorldFrames): WorldFrames {
  if (chart.id === landing.id) return source
  const [x,y] = relativeM(source.ship.origin, ORIGIN)
  const ship = localToSurface(landing, [x,y])
  const origin = dot(ship.radialM, chart.up) > 0 ? point2(...surfaceToLocal(ship, chart)) : point2(4*Math.hypot(...ship.radialM),0)
  let heading = add(scale(landing.east, Math.cos(source.ship.angle)), scale(landing.north, Math.sin(source.ship.angle)))
  if (previous) {
    const old = previous.surface?.chart ?? landing
    heading = add(scale(old.east,Math.cos(previous.ship.angle)),scale(old.north,Math.sin(previous.ship.angle)))
    // Transport the view through adjacent charts rather than projecting a
    // remote fixed vector, whose tangent projection vanishes at two poles.
    const axis = cross(old.up,chart.up), cosine = dot(old.up,chart.up)
    if (cosine <= 0) throw Error('Walking charts must hand off locally')
    heading = add(add(heading,cross(axis,heading)),scale(cross(axis,cross(axis,heading)),1/(1+cosine)))
  }
  const angle = Math.atan2(dot(heading, chart.north), dot(heading, chart.east))
  return {...source, surface: {source,landing,chart}, ship: {...source.ship,origin,angle}}
}
