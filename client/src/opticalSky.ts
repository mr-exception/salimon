import { relativeM, type Position } from './coordinates.ts'
import { dot, length, normal, scale, sub, type Vec3 } from './math.ts'
import type { Body } from './sampleUniverse.ts'
import { bodyPalette } from './bodyPalette.ts'
import { PLANET_DEFINITIONS, planetDefinition, type PlanetReference } from './planetDefinition.ts'

export const MAX_OPTICAL_BODIES = 32
export interface OpticalSphere {
  readonly direction: Vec3
  readonly distanceM: number
  readonly radiusRatio: number
  /** Factored on CPU to retain metres of clearance above million-metre globes. */
  readonly surfaceTerm: number
  readonly angularRadius: number
  readonly lightDirection: Vec3
  readonly color: Vec3
  readonly accent: Vec3
  readonly polarIce: boolean
  readonly emissive: boolean
}
/** Catalog geometry only: no discovery/contact/selection data crosses this API.
 * GPU receives bounded directions and ratios, never absolute split positions.
 * Inside a solid sphere the sky is fully occluded; no invented underground sky.
 */
export function opticalSpheres(bodies: readonly Body[], origin: Position, definitionForBody?: (bodyId: string) => PlanetReference | undefined): OpticalSphere[] {
  if (bodies.length > MAX_OPTICAL_BODIES) throw new RangeError('Optical catalog budget exceeded')
  const star = bodies.find(body => body.kind === 'star')
  return bodies.map(body => {
    const vector = relativeM(body.position, origin), distanceM = length(vector)
    const radius = body.radiusKm * 1000
    if (!Number.isFinite(distanceM) || !Number.isFinite(radius) || radius <= 0) throw new RangeError('Invalid optical sphere')
    const distance = Math.max(distanceM, radius * 1e-12)
    if (distance <= 0) throw new RangeError('Unrepresentable optical sphere')
    // The production session owns the publication, including legacy v1 saves.
    // Generic optical/scanner fixtures have no surface definition to invent.
    const knownSurface = (body.kind === 'planet' || body.kind === 'moon') && PLANET_DEFINITIONS.some(d => d.bodyId === body.id && d.radiusM === radius)
    const reference = knownSurface ? definitionForBody?.(body.id) : undefined
    const definition = reference ? planetDefinition(body.id, reference) : undefined
    return {
      direction: normal(vector), distanceM: distance, radiusRatio: radius / distance,
      surfaceTerm: ((distanceM - radius) / distance) * ((distanceM + radius) / distance),
      angularRadius: Math.asin(Math.min(1, radius / distance)),
      lightDirection: star && star !== body ? normal(relativeM(star.position, body.position)) : [0, 1, 0],
      ...bodyPalette(body.kind), emissive: body.kind === 'star',
      polarIce: definition?.domains.geology.parameters.ice === 1,
    }
  })
}
/** Stable near-root calculation shared with the optical shader. Physical metres. */
export function opticalRayHit(sphere: OpticalSphere, ray: Vec3): number | null {
  if (sphere.surfaceTerm < 0) return 0
  const along = dot(normal(ray), sphere.direction)
  if (along <= 0) return null
  // Cross-product form avoids cancellation for tiny telescope disks.
  const perpendicular = sub(normal(ray), scale(sphere.direction, along))
  const discriminant = sphere.radiusRatio ** 2 - dot(perpendicular, perpendicular)
  if (discriminant < 0) return null
  return sphere.surfaceTerm / (along + Math.sqrt(discriminant)) * sphere.distanceM
}
