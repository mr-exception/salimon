import { physicalRadiusM } from './contact.ts'
import { ORIGIN, relativeM, translate, type Position } from './coordinates.ts'
import { cross, dot, normal, rotateVector, scale, sub, type Vec3 } from './math.ts'
import { planetaryEntities, STARTING_SITE } from './planetarySite.ts'
import { SAMPLE_BODIES, type Body } from './sampleUniverse.ts'
import { point2, rotate2, type WorldFrames } from './topDownFrames.ts'
import { STEP_SECONDS } from './simulation.ts'
import type { Entity2D } from './topDownRuntime.ts'

/** Body-fixed metres from the body's centre, at physicalRadiusM (includes 5 m hull bound).
 * This is an explicit approved landing target, never inferred from a collision. */
export interface SurfaceAddress { readonly bodyId: string; readonly radialM: Vec3; readonly yaw: number }
export interface SurfaceSite {
  readonly id: string
  readonly address: SurfaceAddress
  readonly body: Body
  readonly frames: WorldFrames
}
// No mutable artwork/source definitions enter saves. Only existing local anchors
// and removal of non-source decorations can change; source depletion is the ledger.
export interface SiteState { readonly address: SurfaceAddress; readonly anchors: Readonly<Record<string, readonly [number, number]>> }
export type SiteStates = Readonly<Record<string, SiteState>>
export function startingSurface(): SurfaceAddress {
  const body = SAMPLE_BODIES.find(b => b.id === 'vesper')!, radius = physicalRadiusM(body)
  return { bodyId: body.id, radialM: [2, Math.sqrt(radius * radius - 5), -1], yaw: STARTING_SITE.shipAngle }
}
export function resolveSurface(address: SurfaceAddress, bodies: readonly Body[] = SAMPLE_BODIES, tick = 0): SurfaceSite {
  if (!address || Object.keys(address).sort().join() !== 'bodyId,radialM,yaw') throw Error('Incomplete surface mapping.')
  const body = bodies.find(b => b.id === address.bodyId)
  if (!body || (body.kind !== 'planet' && body.kind !== 'moon')) throw Error('Unsupported surface: choose a sampled planet or moon; stars and asteroids have no walking site.')
  const radial = address.radialM, radius = physicalRadiusM(body)
  if (!Array.isArray(radial) || radial.length !== 3 || !radial.every(Number.isFinite) || !Number.isFinite(address.yaw) || Math.abs(address.yaw) > Math.PI
    || Math.abs(Math.hypot(...radial) - radius) > .001) throw Error('Invalid body-fixed surface coordinates or orientation.')
  let id: string, centre: Vec3, east: Vec3, north: Vec3
  // Only this bounded physical region contains the authored resources. Do not
  // classify all +Y landings (or all Vesper landings) as the original patch.
  if (body.id === 'vesper' && radial[1] > 0 && Math.abs(radial[0]) <= 18 && Math.abs(radial[2]) <= 14) {
    id = STARTING_SITE.id; centre = [0, radius, 0]; east = [1, 0, 0]; north = [0, 0, 1]
  } else {
    // Cubed-sphere 64 m ratio cells, with stable axis tie-breaks and pole handling.
    // Identity belongs to physical region, independent of visit, camera or yaw.
    const axis = [0, 1, 2].reduce((a, b) => Math.abs(radial[b]!) > Math.abs(radial[a]!) ? b : a)
    const u = (axis + 1) % 3, v = (axis + 2) % 3, sign = radial[axis]! < 0 ? -1 : 1
    const steps = Math.ceil(radius / 64), i = Math.round(radial[u]! / Math.abs(radial[axis]!) * steps), j = Math.round(radial[v]! / Math.abs(radial[axis]!) * steps)
    const cube: [number, number, number] = [0, 0, 0]; cube[axis] = sign; cube[u] = i / steps; cube[v] = j / steps
    const up = normal(cube)
    centre = scale(up, radius)
    const reference: Vec3 = axis === 0 ? [0, 0, 1] : [1, 0, 0]
    east = normal(sub(reference, scale(up, dot(reference, up))))
    north = normal(cross(east, up))
    id = `site:surface:v1:${body.id}:${axis}:${sign}:${i}:${j}`
  }
  const delta = sub(radial, centre)
  const project = (v: Vec3) => rotateVector(v, body.angularVelocityRadps ?? [0, 0, 0], tick * STEP_SECONDS)
  const frames: WorldFrames = {
    planet: { id: 'planet:fixture', universeOrigin: translate(body.position, project(centre)), angle: 0, basis: { east: project(east), north: project(north), up: project(normal(centre)) } },
    ship: { id: 'ship:scout', parent: 'planet:fixture', origin: point2(dot(delta, east), dot(delta, north)), angle: address.yaw },
  }
  return { id, body, address: structuredClone(address), frames }
}
export function surfacePosition(site: SurfaceSite, tick = 0): Position { return translate(site.body.position, rotateVector(site.address.radialM, site.body.angularVelocityRadps ?? [0, 0, 0], tick * STEP_SECONDS)) }
export function siteEntities(site: SurfaceSite, saved?: SiteState): Entity2D[] {
  const template: Entity2D[] = site.id === STARTING_SITE.id ? planetaryEntities() : [
    { id: `terrain:${site.id}`, anchor: { frame: 'planet:fixture', point: ORIGIN }, layer: 'terrain', appearance: 'surface' },
  ]
  if (!saved) return template
  if (resolveSurface(saved.address).id !== site.id || !saved.anchors || typeof saved.anchors !== 'object' || Array.isArray(saved.anchors)) throw Error('Invalid saved site identity.')
  const entries = Object.entries(saved.anchors)
  for (const [id, p] of entries) {
    if (!template.some(e => e.id === id) || !Array.isArray(p) || p.length !== 2 || !p.every(n => typeof n === 'number' && Number.isFinite(n) && Math.abs(n) <= 128)) throw Error('Invalid local site anchor.')
  }
  if (template.some(e => (e.source || e.appearance === 'ground' || e.appearance === 'surface') && !Object.hasOwn(saved.anchors, e.id))) throw Error('Missing persistent site entity.')
  return template.filter(e => Object.hasOwn(saved.anchors, e.id)).map(e => ({ ...e, anchor: { frame: 'planet:fixture', point: point2(...saved.anchors[e.id]!) } }))
}
export function captureSite(site: SurfaceSite, entities: readonly Entity2D[]): SiteState {
  const anchors = Object.fromEntries(entities.filter(e => e.anchor.frame === 'planet:fixture' && e.layer === 'terrain').map(e => {
    const [x, y] = relativeM(e.anchor.point, ORIGIN)
    return [e.id, [x, y] as const]
  }))
  const state = { address: site.address, anchors }
  siteEntities(site, state)
  return state
}
/** Physical 5 m bound plus illustrated hull/player margin and aft walking apron. */
export function validateSurfaceClearance(site: SurfaceSite, entities = siteEntities(site)): void {
  const [x, y] = relativeM(site.frames.ship.origin, ORIGIN), angle = site.frames.ship.angle
  const c = Math.abs(Math.cos(angle)), s = Math.abs(Math.sin(angle))
  for (const e of entities) {
    if (!e.footprint) continue
    const [ex, ey] = relativeM(e.anchor.point, ORIGIN), dx = ex - x, dy = ey - y
    const { halfWidth: w, halfHeight: h } = e.footprint
    const [lx, ly] = rotate2([dx, dy], -angle)
    const physical = Math.hypot(Math.max(0, Math.abs(dx) - w), Math.max(0, Math.abs(dy) - h)) < 5
    // Separating-axis test: conservative illustrated rectangle includes 0.46 m
    // walking margin. The aft apron extends beyond the doorway to local y=8.5.
    const overlap = (cx: number, cy: number, hw: number, hh: number) => {
      const [wx, wy] = rotate2([cx, cy], angle)
      return Math.abs(lx - cx) < hw + c*w + s*h && Math.abs(ly - cy) < hh + s*w + c*h
        && Math.abs(dx - wx) < w + c*hw + s*hh && Math.abs(dy - wy) < h + s*hw + c*hh
    }
    if (physical || overlap(0, 0, 4.2, 7.5) || overlap(0, 7, 1.5, 1.5)) throw Error('Unsafe landing footprint: hull or aft access clearance is obstructed.')
  }
}
