import { definitionDigest, planetDefinition, type PlanetReference } from './planetDefinition.ts'
import { surfaceRegion, type SurfaceLocation } from './surfaceCoordinates.ts'

/** Supplemental immutable publication; terrain/catalog hashes are never rewritten. */
export const ENVIRONMENT_RULES = Object.freeze({
  version: 1, algorithm: 'fnv1a64-modulo:v1', seedPolicy: 'canonical-body-v1-terrain-seed',
  regionGrid: 'surface:v1:cell', temperatureMinC: -180, temperatureMaxC: 180,
  rateMinHpPerSecond: 0, rateMaxHpPerSecond: 20,
  safeBodyId: 'vesper', safePositiveY: true, safeHalfExtentM: 128,
  safeTemperatureC: 24, safeRadiationHpPerSecond: 0, safeToxicityHpPerSecond: 0,
})
export const ENVIRONMENT_PUBLICATION = 'f9512d32ccbcfd7e'
export const SCOUT_SAFE_ENVIRONMENT = Object.freeze({ temperatureC: 24, radiationHpPerSecond: 0, toxicityHpPerSecond: 0 })
export function validateEnvironmentPublication(value: unknown): typeof ENVIRONMENT_PUBLICATION {
  if (value !== ENVIRONMENT_PUBLICATION || definitionDigest(ENVIRONMENT_RULES) !== ENVIRONMENT_PUBLICATION) throw Error('Unsupported environmental publication; approved migration required')
  return ENVIRONMENT_PUBLICATION
}
function bucket(bodyId: string, field: 'temperature' | 'radiation' | 'toxicity', size: number, cell?: object): number {
  // The frozen v1 body seed is canonical world identity, independent of visual,
  // geological or catalog publication changes and of mutable random state.
  const seed = planetDefinition(bodyId).domains.terrain.seed
  const key = { publication: ENVIRONMENT_PUBLICATION, seed, bodyId, field, ...(cell ? { cell } : {}) }
  return Number(BigInt('0x' + definitionDigest(key)) % BigInt(size))
}
export interface EnvironmentSample {
  readonly publication: typeof ENVIRONMENT_PUBLICATION
  readonly bodyId: string; readonly regionId: string
  readonly temperatureC: number; readonly radiationHpPerSecond: number; readonly toxicityHpPerSecond: number
  readonly safeStartingRegion: boolean
}
/** Bounded scanner input; the range is the publication's supported range, not an inferred atmosphere. */
export function bodyEnvironmentSummary(bodyId: string, reference?: PlanetReference, publication: string = ENVIRONMENT_PUBLICATION) {
  validateEnvironmentPublication(publication); planetDefinition(bodyId, reference)
  return Object.freeze({ publication: ENVIRONMENT_PUBLICATION, bodyId,
    temperatureRangeC: Object.freeze([-180, 180] as const),
    radiationHpPerSecond: bucket(bodyId, 'radiation', 21), toxicityHpPerSecond: bucket(bodyId, 'toxicity', 21),
    safeStartingRegionOverride: bodyId === ENVIRONMENT_RULES.safeBodyId,
  })
}
/** No cache or random resampling. One canonical cell query and three fixed-size hashes. */
export function samplePlanetEnvironment(location: SurfaceLocation, reference?: PlanetReference, publication: string = ENVIRONMENT_PUBLICATION): EnvironmentSample {
  const summary = bodyEnvironmentSummary(location?.bodyId, reference, publication)
  const region = surfaceRegion(location, reference) // validates finite body-fixed coordinates and publication
  const [x, y, z] = location.radialM
  // Orthographic +Y starting tangent chart is exactly x/z; never the current
  // landing/chart/cache origin. Inclusive square, front hemisphere only.
  const safeStartingRegion = location.bodyId === ENVIRONMENT_RULES.safeBodyId && y > 0 && Math.abs(x) <= ENVIRONMENT_RULES.safeHalfExtentM && Math.abs(z) <= ENVIRONMENT_RULES.safeHalfExtentM
  return Object.freeze({ publication: ENVIRONMENT_PUBLICATION, bodyId: location.bodyId, regionId: region.id, safeStartingRegion,
    ...(safeStartingRegion ? SCOUT_SAFE_ENVIRONMENT : {
      temperatureC: bucket(location.bodyId, 'temperature', ENVIRONMENT_RULES.temperatureMaxC - ENVIRONMENT_RULES.temperatureMinC + 1, region.cell) + ENVIRONMENT_RULES.temperatureMinC,
      radiationHpPerSecond: summary.radiationHpPerSecond, toxicityHpPerSecond: summary.toxicityHpPerSecond,
    }),
  })
}
