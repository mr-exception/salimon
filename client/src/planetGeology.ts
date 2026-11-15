import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { add, dot, normal, scale, sub, type Vec3 } from './math.ts'
import { planetDefinition, type PlanetReference } from './planetDefinition.ts'
import { localToSurface, surfaceRegion, validateSurfaceLocation, type SurfaceLocation, type SurfaceRegion } from './surfaceCoordinates.ts'

export const WALK_MAX_SLOPE_RADIANS = 32 * Math.PI / 180
export const LANDING_MAX_SLOPE_RADIANS = 8 * Math.PI / 180
export const MAX_TERRAIN_BATCH = 2048
export const MAX_TERRAIN_MOTION_M = 64
// Global upper bound of the height Hessian (m/m²), including province and
// authored blends. A footprint uses this bound rather than point-only contact.
export const TERRAIN_CURVATURE_BOUND = .2
const smooth = (x: number) => { const u = Math.max(0, Math.min(1, x)); return u*u*u*(10+u*(-15+6*u)) }
const phase = (seed: string) => Number(BigInt(`0x${seed}`) % 65521n) / 65521 * Math.PI * 2
/** Conservative local grade for a trajectory contained in the authored disk.
 * Raw relief is <600 m and raw grade <2; quintic blend derivative is bounded
 * analytically, retaining safe original lateral approaches without point sweeps. */
export function authoredGradeBound(radiusM: number): number {
  const u = Math.max(0, Math.min(1, (radiusM-200)/1800)), peak = Math.min(u,.5)
  return 2*smooth(u) + 600*30*peak*peak*(1-peak)*(1-peak)/1800
}
export type GroundType = 'basin-regolith' | 'highland-rock' | 'fractured-rock' | 'polar-ice' | 'authored-ground'

/** Global Cartesian waves: no region key, UV face, LOD, cache or query order
 * enters the field. The shortest wavelength is 48 m; provinces span kilometres.
 * The function and its first two derivatives are continuous at every blend. */
function field(p: Vec3, bodyId: string, reference: PlanetReference) {
  const d = planetDefinition(bodyId, reference), t = d.domains.terrain.parameters, g = d.domains.geology.parameters
  const a = phase(d.domains.terrain.seed), b = phase(d.domains.geology.seed)
  const macro = Math.sin(dot(p, [.000032, .000019, -.000023]) + b)
  // Intersecting waves form closed impact bowls and raised rings.
  const u = Math.sin(dot(p, [-.000071, .000043, .000029]) + a)
  const v = Math.sin(dot(p, [.000023, .000061, -.000049]) + b)
  const distance2 = u*u + v*v
  const impact = 1-smooth(distance2)
  const rim = smooth(distance2/.3)*(1-smooth((distance2-.5)/.5))
  const basin = .5 + .35*rim - .45*impact
  const fracture = smooth((macro - .15) / .65)
  const ridge = .5 + .5*Math.sin(dot(p, [.105, .065, -.045]) + a)
  const icy = g.ice === 1 ? smooth((Math.abs(p[1]) / d.radiusM - .65) / .2) : 0
  const authoredDistance = bodyId === AUTHORED_SURFACE.bodyId && p[1] > 0 ? Math.hypot(p[0], p[2]) : Infinity
  const blend = smooth((authoredDistance - 200) / 1800)
  const heightM = blend * (Number(t.reliefM) * (.4 + .3*macro + .3*basin) + Number(t.fractureM)*fracture*ridge*(1-icy))
  const weights = { impact, basin: (1-macro)/2, highland: (1+macro)/2, fracture, ice: icy }
  const groundType: GroundType = blend === 0 ? 'authored-ground' : icy > .5 ? 'polar-ice' : fracture > .5 ? 'fractured-rock' : macro > 0 ? 'highland-rock' : 'basin-regolith'
  return { heightM, groundType, weights }
}
/** Validated immutable publication, single physical position; no allocations
 * proportional to planetary area and no persistent terrain cache. */
export function sampleGeology(location: SurfaceLocation, reference: PlanetReference) {
  validateSurfaceLocation(location, reference)
  const d = planetDefinition(location.bodyId, reference)
  if (d.version !== 2 || d.domains.terrain.algorithm !== 'geological-sphere:v2' || d.domains.geology.algorithm !== 'provinces:v2') throw Error('Unsupported geological publication')
  const p = scale(normal(location.radialM), d.radiusM), value = field(p, location.bodyId, reference)
  // Fixed physical finite difference, independent of visual sampling density.
  const gradient = [0, 1, 2].map(axis => {
    const step: [number, number, number] = [0, 0, 0]; step[axis] = .125
    return (field(add(p, step), location.bodyId, reference).heightM - field(sub(p, step), location.bodyId, reference).heightM) / .25
  }) as unknown as Vec3
  const up = normal(p), tangentGradient = sub(gradient, scale(up, dot(gradient, up)))
  const normalBody = normal(sub(up, tangentGradient)), grade = Math.hypot(...tangentGradient)
  return { ...value, normalBody, grade, slopeRadians: Math.atan(grade), solid: true as const,
    geologyId: `${d.bodyId}:geology:v2:${d.domains.geology.digest}` }
}
export function sampleGeologyBatch(locations: readonly SurfaceLocation[], reference: PlanetReference) {
  if (!Array.isArray(locations) || locations.length > MAX_TERRAIN_BATCH) throw Error('Terrain batch exceeds bounded workload')
  return locations.map(location => sampleGeology(location, reference))
}
/** Conservative disk contact: slope + curvature × radius bounds the entire
 * footprint. Long walks are swept in fixed .25 m intervals, never endpoint only.
 * Unreasonably large probes fail explicitly instead of allocating huge patches. */
export function probeGeology(location: SurfaceLocation, reference: PlanetReference, radius = 0, motion: readonly [number, number] = [0, 0], landingYaw?: number, chart?: SurfaceRegion) {
  if (!Number.isFinite(radius) || radius < 0 || radius > 4 || !Array.isArray(motion) || motion.length !== 2 || !motion.every(Number.isFinite) || Math.hypot(...motion) > MAX_TERRAIN_MOTION_M || (landingYaw !== undefined && !Number.isFinite(landingYaw))) throw Error('Terrain probe exceeds bounded workload')
  const region = chart ?? surfaceRegion(location, reference)
  // Recenter a chart at the exact physical query. Axes remain the query's
  // canonical directions, so probe motion agrees with the public surface API.
  const origin = [dot(sub(location.radialM, region.centre), region.east), dot(sub(location.radialM, region.centre), region.north)] as const
  const at = (x: number, y: number) => sampleGeology(localToSurface(region, [origin[0]+x, origin[1]+y]), reference)
  const centre = sampleGeology(location, reference)
  // Exact authored plateau, including the complete swept/landing footprint.
  // Avoid a dense slope grid at 60 Hz for the preserved starting site.
  if (location.bodyId === AUTHORED_SURFACE.bodyId && location.radialM[1] > 0
    && Math.hypot(location.radialM[0], location.radialM[2]) + Math.hypot(...motion) + radius + (landingYaw === undefined ? 0 : 11) < 190) {
    return { ...centre, collision: false, moveFraction: 1, landingClear: landingYaw === undefined ? null : true }
  }
  const blocked = (grade: number, margin: number) => grade + TERRAIN_CURVATURE_BOUND*margin > Math.tan(WALK_MAX_SLOPE_RADIANS)
  const collision = blocked(centre.grade, radius)
  const steps = Math.max(1, Math.ceil(Math.hypot(...motion) / .25))
  let moveFraction = 1
  if (Math.hypot(...motion) > 0) for (let i = 0; i <= steps; i++) {
    if (blocked(at(motion[0]*i/steps, motion[1]*i/steps).grade, radius + .25)) { moveFraction = Math.max(0, (i-1)/steps); break }
  }
  let landingClear: boolean | null = null
  if (landingYaw !== undefined) {
    landingClear = true
    // Conservative enclosing box includes the 5 m Scout bound, full hull,
    // player margin and aft apron; .5 m grid with a half-diagonal safety bound.
    const c = Math.cos(landingYaw), s = Math.sin(landingYaw)
    for (let x = -5; x <= 5 && landingClear; x += .5) for (let y = -8.5; y <= 8.5; y += .5) {
      const sample = at(c*x-s*y, s*x+c*y)
      if (sample.grade + TERRAIN_CURVATURE_BOUND * Math.SQRT1_2 / 2 > Math.tan(LANDING_MAX_SLOPE_RADIANS) || Math.abs(sample.heightM-centre.heightM) > .5) { landingClear = false; break }
    }
  }
  return { ...centre, collision, moveFraction, landingClear }
}
