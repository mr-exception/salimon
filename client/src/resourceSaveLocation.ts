import { depositFromId, isDepositId } from './planetDeposits.ts'
import { probeGeology } from './planetGeology.ts'
import { surfaceRegion, localToSurface } from './surfaceCoordinates.ts'
import { planetDefinition } from './planetDefinition.ts'
import { captureSite, resolveSurface, siteEntities, validateSurfaceClearance, type SiteStates, type SiteState, type SurfaceAddress } from './surfaceSite.ts'
import { synchronizeDoor } from './scoutDoor.ts'
import { CELL_SIZE_M, ORIGIN, relativeM } from './coordinates.ts'
import { leaveCockpit } from './cockpit.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { type ResourceSource } from './resourceInventory.ts'
import { walkingFrames, relativeAnchor, rotate2, type Anchor } from './topDownFrames.ts'
import { PLAYER_RADIUS_M } from './topDownInteraction.ts'
import { capturePlayerLocation, type TopDownState } from './topDownRuntime.ts'
import type { PlayerLocation } from './topDownTraversal.ts'
import { RESOURCE_SOURCES } from './resourceCatalog.ts'
export const serializeForDecode = (value: unknown) => JSON.parse(serialize(value))

export const serialize = (value: unknown) => JSON.stringify(value, (_, v) => typeof v === 'bigint' ? v.toString() : v)

export function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Expected object')
  return value as Record<string, unknown>
}

export function integer(value: unknown, min = 0): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < min) throw new Error('Invalid integer')
  return value
}

export function parseLocation(value: unknown, world: TopDownState): PlayerLocation {
  const location = object(value), anchor = object(location.anchor), point = object(anchor.point)
  if (anchor.frame !== world.frames.planet.id && anchor.frame !== world.frames.ship.id) throw new Error('Unknown frame')
  if (!Array.isArray(point.cell) || point.cell.length !== 3 || !Array.isArray(point.offsetM) || point.offsetM.length !== 3) throw new Error('Invalid position')
  const cell = point.cell.map((v: unknown) => {
    if (typeof v !== 'string' || !/^-?(0|[1-9][0-9]{0,127})$/.test(v)) throw new Error('Invalid cell')
    return BigInt(v)
  }) as [bigint, bigint, bigint]
  const offsetM = point.offsetM.map((v: unknown) => {
    if (typeof v !== 'number' || !Number.isFinite(v) || v < -CELL_SIZE_M / 2 || v >= CELL_SIZE_M / 2) throw new Error('Invalid offset')
    return v
  }) as [number, number, number]
  if (cell[2] !== 0n || offsetM[2] !== 0 || typeof location.facing !== 'number' || !Number.isFinite(location.facing)) throw new Error('Invalid planar location')
  const chart = anchor.chart as import('./surfaceCoordinates.ts').SurfaceRegion | undefined
  if (anchor.chart !== undefined && (!chart || typeof chart !== 'object' || Array.isArray(chart))) throw Error('Invalid walking chart')
  if (chart) {
    if (!world.scoutLocation.binding || anchor.frame !== 'planet:fixture') throw Error('Invalid walking chart')
    const landing = surfaceRegion(world.scoutLocation.surface,world.scoutLocation.surface.definition)
    if (chart.bodyId !== landing.bodyId || chart.definition.digest !== landing.definition.digest) throw Error('Walking chart publication mismatch')
    localToSurface(chart,[offsetM[0],offsetM[1]])
    world = {...world,frames:walkingFrames(world.scoutLocation.frames,landing,chart)}
  }
  const parsed: Anchor = { frame: anchor.frame, point: { cell, offsetM }, ...(chart ? {chart} : {}) }
  if (world.frames.ship.parent === null && (parsed.frame !== 'ship:scout' || cell.some(v=>v!==0n) || offsetM[1]>5.5 || offsetM[1]<-5.5 || Math.abs(offsetM[0])>2.2)) throw new Error('Unsafe airborne location')
  for (const entity of world.entities) {
    if (!entity.footprint || world.frames.ship.parent === null && entity.anchor.frame !== parsed.frame) continue
    const [x, y] = rotate2(relativeAnchor(parsed, entity.anchor, world.frames), entity.anchor.frame === world.frames.ship.id ? -world.frames.ship.angle : 0)
    if (![x, y].every(Number.isFinite) || (Math.abs(x) < entity.footprint.halfWidth + PLAYER_RADIUS_M - 1e-8 && Math.abs(y) < entity.footprint.halfHeight + PLAYER_RADIUS_M - 1e-8)) throw new Error('Unsafe saved location')
  }
  const surface = world.scoutLocation.surface
  if (parsed.frame === 'planet:fixture') {
    const [x,y] = relativeM(parsed.point, ORIGIN)
    localToSurface(chart ?? surfaceRegion(surface,surface.definition),[x,y])
  }
  if (parsed.frame === 'planet:fixture' && surface.definition?.version === 2) {
    const region = chart ?? surfaceRegion(surface, surface.definition), [x,y] = relativeM(parsed.point, ORIGIN)
    if (probeGeology(localToSurface(region,[x,y]), surface.definition, PLAYER_RADIUS_M).collision) throw Error('Unsafe saved terrain location')
  }
  return { anchor: parsed, facing: location.facing }
}

export function decodeSites(value: unknown): SiteStates {
  const bodyRevisions = new Map<string, string>()
  const states = object(value), sites: Record<string, SiteState> = {}
  if (!Object.hasOwn(states, STARTING_SITE.id)) throw Error('Missing original world state')
  for (const [id, raw] of Object.entries(states)) {
    const state = object(raw)
    if (Object.keys(state).sort().join() !== 'address,anchors') throw Error('Invalid site fields')
    const site = resolveSurface(state.address as SurfaceAddress)
    if (site.id !== id) throw Error('Saved site/body mismatch')
    const saved = state as unknown as SiteState
    const digest = planetDefinition(site.body.id, saved.address.definition).digest
    if (bodyRevisions.has(site.body.id) && bodyRevisions.get(site.body.id) !== digest) throw Error('Conflicting saved body revisions')
    bodyRevisions.set(site.body.id, digest)
    siteEntities(site, saved)
    sites[id] = saved
  }
  return sites
}

export function materializeSite(world: TopDownState, sites: SiteStates): TopDownState {
  const location = world.scoutLocation
  const terrain = location.binding ? siteEntities(resolveSurface(location.surface), sites[location.siteId]) : []
  if (location.binding) validateSurfaceClearance(resolveSurface(location.surface), terrain)
  return synchronizeDoor({ ...world, entities: [...world.entities.filter(e => e.anchor.frame !== 'planet:fixture' || e.layer !== 'terrain'), ...terrain] })
}

export /** Work is source-bound, canonical and strictly below one 960-unit extraction. */
function validateMiningWork(value: unknown, sources: Readonly<Record<string, ResourceSource>> = RESOURCE_SOURCES): Record<string, number> {
  const work: Record<string, number> = {}
  for (const [id, units] of Object.entries(object(value))) {
    if (!Object.hasOwn(sources, id) || integer(units, 1) >= 960) throw new Error('Invalid mining work')
    work[id] = units as number
  }
  return work
}

export function sourceDefinitions(sources: Readonly<Record<string, ResourceSource>>): Readonly<Record<string, ResourceSource>> {
  const definitions = { ...RESOURCE_SOURCES }
  for (const id of Object.keys(sources)) if (!Object.hasOwn(definitions, id)) definitions[id] = depositFromId(id).initial
  return definitions
}

export function safeLocation(world: TopDownState): PlayerLocation {
  // A reload returns standing beside the chair, never inside its collider.
  return capturePlayerLocation(world.player.mode === 'seated' ? leaveCockpit(world) : world)
}

export function captureResourceSites(world: TopDownState, sites: SiteStates): SiteStates {
  const location = world.scoutLocation
  return location.binding ? { ...sites, [location.siteId]: captureSite(resolveSurface(location.surface), world.entities.filter(e => !isDepositId(e.id))) } : sites
}
