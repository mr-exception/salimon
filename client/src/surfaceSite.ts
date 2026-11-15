import { samplePlanetSurface } from './planetSurface.ts'
import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { physicalRadiusM } from './contact.ts'
import { ORIGIN, relativeM, translate, type Position } from './coordinates.ts'
import { add, normal, scale, rotateVector, type Vec3 } from './math.ts'
import { planetaryEntities, STARTING_SITE } from './planetarySite.ts'
import { SAMPLE_BODIES, type Body } from './sampleUniverse.ts'
import { point2, type WorldFrames } from './topDownFrames.ts'
import { surfaceRegion, surfaceToLocal } from './surfaceCoordinates.ts'
import { planetDefinition, geologicalDefinition, type PlanetReference } from './planetDefinition.ts'
import { STEP_SECONDS } from './simulation.ts'
import type { Entity2D } from './topDownRuntime.ts'

/** Body-fixed metres from the body's centre, at physicalRadiusM (includes 5 m hull bound).
 * This is an explicit approved landing target, never inferred from a collision. */
export interface SurfaceAddress { readonly bodyId: string; readonly radialM: Vec3; readonly yaw: number; readonly definition?: PlanetReference }
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
  const { bodyId, version, digest } = geologicalDefinition(body.id)
  return { bodyId: body.id, radialM: [2, Math.sqrt(radius * radius - 5), -1], yaw: STARTING_SITE.shipAngle, definition: { bodyId, version, digest } }
}
export function resolveSurface(address: SurfaceAddress, bodies: readonly Body[] = SAMPLE_BODIES, tick = 0): SurfaceSite {
  if (!address || Object.keys(address).filter(k => k !== 'definition').sort().join() !== 'bodyId,radialM,yaw') throw Error('Incomplete surface mapping.')
  const body = bodies.find(b => b.id === address.bodyId)
  if (!body || (body.kind !== 'planet' && body.kind !== 'moon')) throw Error('Unsupported surface: choose a sampled planet or moon; stars and asteroids have no walking site.')
  const radial = address.radialM, radius = physicalRadiusM(body)
  if (!Array.isArray(radial) || radial.length !== 3 || !radial.every(Number.isFinite) || !Number.isFinite(address.yaw) || Math.abs(address.yaw) > Math.PI
    || Math.abs(Math.hypot(...radial) - radius) > .001) throw Error('Invalid body-fixed surface coordinates or orientation.')
  if (Object.hasOwn(address, 'definition') && address.definition !== undefined && (!address.definition || Object.keys(address.definition).sort().join() !== 'bodyId,digest,version')) throw Error('Invalid surface publication reference')
  const definition = planetDefinition(body.id, address.definition)
  if (body.radiusKm * 1000 !== definition.radiusM) throw Error('Planet geometry/version mismatch')
  const region = surfaceRegion(address, address.definition)
  const { id, centre, east, north, up } = region
  const local = surfaceToLocal(address, region)
  const project = (v: Vec3) => rotateVector(v, body.angularVelocityRadps ?? [0, 0, 0], tick * STEP_SECONDS)
  const frames: WorldFrames = {
    planet: { id: 'planet:fixture', universeOrigin: translate(body.position, project(centre)), angle: 0, basis: { east: project(east), north: project(north), up: project(up) } },
    ship: { id: 'ship:scout', parent: 'planet:fixture', origin: point2(...local), angle: address.yaw },
  }
  return { id, body, address: structuredClone(address), frames }
}
/** Navigation shell stays canonical; only the physical ship pose receives height. */
export function surfaceContactRadial(address: SurfaceAddress): Vec3 {
  const height = address.definition?.version === 2 ? samplePlanetSurface(address, address.definition).heightM : 0
  return add(address.radialM, scale(normal(address.radialM), height))
}
export function surfacePosition(site: SurfaceSite, tick = 0): Position { return translate(site.body.position, rotateVector(surfaceContactRadial(site.address), site.body.angularVelocityRadps ?? [0, 0, 0], tick * STEP_SECONDS)) }
export function siteEntities(site: SurfaceSite, saved?: SiteState): Entity2D[] {
  const template: Entity2D[] = site.id === STARTING_SITE.id ? planetaryEntities() : [
    { id: `terrain:${site.id}`, anchor: { frame: 'planet:fixture', point: ORIGIN }, layer: 'terrain', appearance: 'surface' },
  ]
  if (!saved) return template
  if (resolveSurface(saved.address).id !== site.id || !saved.anchors || typeof saved.anchors !== 'object' || Array.isArray(saved.anchors)) throw Error('Invalid saved site identity.')
  if (planetDefinition(site.body.id, saved.address.definition).digest !== planetDefinition(site.body.id, site.address.definition).digest) throw Error('Saved surface revision mismatch; approved migration required')
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
  const definition = planetDefinition(site.body.id, site.address.definition)
  const delta = site.id === STARTING_SITE.id ? {
    definition,
    anchors: Object.fromEntries(entities.filter(e => AUTHORED_SURFACE.objects.some(o => o[0] === e.id)).map(e => {
      const [x, y] = relativeM(e.anchor.point, ORIGIN)
      return [e.id, [x, y] as const]
    })),
  } : undefined
  const sample = samplePlanetSurface(site.address, definition, { landingYaw: site.frames.ship.angle }, delta)
  if (!sample.solid || !Number.isFinite(sample.slopeRadians) || !sample.landingClear) throw Error('Unsafe landing footprint: hull or aft access clearance is obstructed.')
}
