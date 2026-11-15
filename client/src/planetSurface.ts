import { probeGeology } from './planetGeology.ts'
import { dot, add, scale } from './math.ts'
import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { planetDefinition, type PlanetReference } from './planetDefinition.ts'
import { surfaceRegion, surfaceToLocal, type SurfaceLocation } from './surfaceCoordinates.ts'
import { samplePhysicalSurface, type SurfaceProbe } from './surfacePhysics.ts'

/** Per-save overlay: reference binds immutable truth; only authored local
 * anchors/removals change. Mining depletion stays in the original source ledger. */
export interface SurfaceDelta {
  readonly definition: PlanetReference
  readonly anchors: Readonly<Record<string, readonly [number, number]>>
}
export function authoredObstacles(delta?: SurfaceDelta) {
  if (delta) {
    planetDefinition(AUTHORED_SURFACE.bodyId, delta.definition)
    if (!delta.anchors || typeof delta.anchors !== 'object' || Array.isArray(delta.anchors)) throw Error('Invalid surface delta')
    for (const [id, anchor] of Object.entries(delta.anchors)) {
      if (!AUTHORED_SURFACE.objects.some(o => o[0] === id) || !Array.isArray(anchor) || anchor.length !== 2 || !anchor.every(n => typeof n === 'number' && Number.isFinite(n) && Math.abs(n) <= 128)) throw Error('Invalid surface delta anchor')
    }
    for (const id of ['source:landing:mineral:01', 'source:landing:biology:01']) if (!Object.hasOwn(delta.anchors, id)) throw Error('Missing original source identity')
  }
  return AUTHORED_SURFACE.objects.filter(o => !delta || Object.hasOwn(delta.anchors, o[0])).map(([id, x, y, halfWidth, halfHeight]) => {
    const point = delta?.anchors[id] ?? [x, y]
    return { id, x: point[0], y: point[1], halfWidth, halfHeight }
  })
}
/** One surface truth for renderer/collision/gathering/landing consumers. The
 * legacy flat terrain has no generated deposits away from authored Vesper.
 * Probe positions/deltas use this body's canonical region, never screen space. */
export function samplePlanetSurface(location: SurfaceLocation, reference?: PlanetReference, probe: Omit<SurfaceProbe, 'x' | 'y'> = {}, delta?: SurfaceDelta) {
  if (probe.motion !== undefined && (!Array.isArray(probe.motion) || probe.motion.length !== 2 || !probe.motion.every(Number.isFinite))) throw Error('Invalid physical surface motion')
  const region = surfaceRegion(location, reference)
  if (delta) {
    planetDefinition(location.bodyId, delta.definition)
    if (delta.definition.version !== region.definition.version || delta.definition.digest !== region.definition.digest) throw Error('Surface delta revision mismatch')
  }
  // Authored obstacles also affect probes just beyond the inset boundary. Use
  // its physical tangent chart, not region membership, to avoid a collision seam.
  let physicalProbe = probe
  let obstacles: ReturnType<typeof authoredObstacles> = [], local = surfaceToLocal(location, region)
  if (location.bodyId === AUTHORED_SURFACE.bodyId && location.radialM[1] > 0) {
    const definition = planetDefinition(location.bodyId, reference)
    const authored = surfaceRegion({ bodyId: location.bodyId, radialM: [0, definition.radiusM + 5, 0] }, reference)
    local = surfaceToLocal(location, authored); obstacles = authoredObstacles(delta)
    // Project canonical-region directions into the authored chart as well.
    const project = (v: readonly [number, number]) => {
      const direction = add(scale(region.east, v[0]), scale(region.north, v[1]))
      return [dot(direction, authored.east), dot(direction, authored.north)] as const
    }
    const heading = project([Math.cos(probe.landingYaw ?? 0), Math.sin(probe.landingYaw ?? 0)])
    physicalProbe = { ...probe, motion: probe.motion ? project(probe.motion) : undefined,
      landingYaw: probe.landingYaw === undefined ? undefined : Math.atan2(heading[1], heading[0]) }
  } else if (delta) throw Error('Surface delta does not belong to this surface')
  const result = samplePhysicalSurface({ ...physicalProbe, x: local[0], y: local[1] }, obstacles)
  const definition = planetDefinition(location.bodyId, reference)
  const geology = definition.version === 2 ? probeGeology(location, definition, probe.radiusM, probe.motion, probe.landingYaw) : undefined
  return { ...result, ...geology, collision: result.collision || !!geology?.collision,
    moveFraction: Math.min(result.moveFraction, geology?.moveFraction ?? 1),
    landingClear: result.landingClear === null ? null : result.landingClear && (geology?.landingClear ?? true), bodyId: location.bodyId, definition: region.definition, regionId: region.id, siteId: region.id,
    terrainId: `${location.bodyId}:terrain:v${definition.version}:${definition.domains.terrain.digest}`,
    sourceIds: obstacles.filter(o => o.id.startsWith('source:') && Math.abs(o.x - local[0]) <= o.halfWidth && Math.abs(o.y - local[1]) <= o.halfHeight).map(o => o.id),
  }
}
