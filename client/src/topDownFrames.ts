import { ORIGIN, relativeM, translate, type Position } from './coordinates.ts'

export type Point2 = readonly [number, number]
export type FrameId = 'planet:fixture' | 'ship:scout'
export interface Anchor { readonly frame: FrameId; readonly point: Position }
export interface WorldFrames {
  readonly planet: { readonly id: 'planet:fixture'; readonly universeOrigin: Position; readonly angle: number }
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
  if (anchor.frame === frames.planet.id) return anchor.point
  if (frames.ship.parent === null) throw new Error('Planetary frame unavailable in flight')
  const [x, y] = rotate2(planar(anchor.point), frames.ship.angle)
  return translate(frames.ship.origin, [x, y, 0])
}
export function convertAnchor(anchor: Anchor, frame: FrameId, frames: WorldFrames): Anchor {
  if (anchor.frame === frame) return anchor
  if (frames.ship.parent === null) throw new Error('Planetary frame unavailable in flight')
  const planetPoint = inPlanet(anchor, frames)
  if (frame === frames.planet.id) return { frame, point: planetPoint }
  const [x, y] = relativeM(planetPoint, frames.ship.origin)
  const [localX, localY] = rotate2([x, y], -frames.ship.angle)
  return { frame, point: point2(localX, localY) }
}
// Returns planet-axis metres, including when both points are ship-local.
export function relativeAnchor(target: Anchor, origin: Anchor, frames: WorldFrames): Point2 {
  if (target.frame === origin.frame) {
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
