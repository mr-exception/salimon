import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { add, cross, dot, normal, scale, sub, type Vec3 } from './math.ts'
import { planetDefinition, type PlanetReference } from './planetDefinition.ts'

/** Legacy #842 shell: body-fixed metres at ground radius + conservative 5 m
 * Scout bound. It is separate from split astronomical Position and local metres. */
export interface SurfaceLocation { readonly bodyId: string; readonly radialM: Vec3 }
export interface SurfaceRegion {
  readonly id: string; readonly bodyId: string; readonly definition: PlanetReference
  readonly centre: Vec3; readonly east: Vec3; readonly north: Vec3; readonly up: Vec3
  readonly cell: { readonly axis: number; readonly sign: number; readonly i: number; readonly j: number; readonly steps: number }
}
export const SURFACE_SHELL_MARGIN_M = 5
export function validateSurfaceLocation(location: SurfaceLocation, reference?: PlanetReference): number {
  const definition = planetDefinition(location?.bodyId, reference), radius = definition.radiusM + SURFACE_SHELL_MARGIN_M
  const radial = location.radialM
  if (!Array.isArray(radial) || radial.length !== 3 || !radial.every(n => typeof n === 'number' && Number.isFinite(n))
    || Math.abs(Math.hypot(...radial) - radius) > .001) throw Error('Invalid body-fixed surface coordinates')
  return radius
}
/** Axis ties use X, then Y, then Z. Quantization is the shipped surface:v1
 * ratio grid, including its IDs. Changing it requires a save migration. */
function classifyRegion(location: SurfaceLocation, reference?: PlanetReference, authored = true): SurfaceRegion {
  const radius = validateSurfaceLocation(location, reference), definition = planetDefinition(location.bodyId, reference)
  const radial = location.radialM
  const axis = [0, 1, 2].reduce((a, b) => Math.abs(radial[b]!) > Math.abs(radial[a]!) ? b : a)
  const u = (axis + 1) % 3, v = (axis + 2) % 3, sign = radial[axis]! < 0 ? -1 : 1
  const steps = Math.ceil(radius / 64), i = Math.round(radial[u]! / Math.abs(radial[axis]!) * steps), j = Math.round(radial[v]! / Math.abs(radial[axis]!) * steps)
  const cell = { axis, sign, i: i || 0, j: j || 0, steps }
  let id: string, centre: Vec3, east: Vec3, north: Vec3
  if (authored && location.bodyId === AUTHORED_SURFACE.bodyId && radial[1] > 0 && Math.abs(radial[0]) <= AUTHORED_SURFACE.bounds[0] && Math.abs(radial[2]) <= AUTHORED_SURFACE.bounds[1]) {
    id = AUTHORED_SURFACE.siteId; centre = [0, radius, 0]; east = [1, 0, 0]; north = [0, 0, 1]
  } else {
    const cube: [number, number, number] = [0, 0, 0]; cube[axis] = sign; cube[u] = i / steps; cube[v] = j / steps
    const up = normal(cube)
    centre = scale(up, radius)
    const referenceAxis: Vec3 = axis === 0 ? [0, 0, 1] : [1, 0, 0]
    east = normal(sub(referenceAxis, scale(up, dot(referenceAxis, up))))
    north = normal(cross(east, up))
    id = `site:surface:v1:${location.bodyId}:${axis}:${sign}:${i}:${j}`
  }
  return { id, bodyId: location.bodyId, definition: { bodyId: definition.bodyId, version: definition.version, digest: definition.digest }, centre, east, north, up: normal(centre), cell }
}
export function surfaceRegion(location: SurfaceLocation, reference?: PlanetReference): SurfaceRegion {
  return classifyRegion(location, reference)
}
function checkedRegion(region: SurfaceRegion): number {
  const radius = validateSurfaceLocation({ bodyId: region.bodyId, radialM: region.centre }, region.definition)
  // Reconstruct instead of accepting caller-supplied axes/cache data as truth.
  const { axis, sign, i, j, steps } = region.cell ?? {}
  if (![0, 1, 2].includes(axis) || ![-1, 1].includes(sign) || steps !== Math.ceil(radius / 64) || !Number.isInteger(i) || !Number.isInteger(j) || Math.abs(i) > steps || Math.abs(j) > steps) throw Error('Invalid canonical tangent frame cell')
  const cube: [number, number, number] = [0, 0, 0]
  cube[axis] = sign; cube[(axis + 1) % 3] = i / steps; cube[(axis + 2) % 3] = j / steps
  const authored = region.id === AUTHORED_SURFACE.siteId
  if (authored && (region.bodyId !== AUTHORED_SURFACE.bodyId || axis !== 1 || sign !== 1 || i !== 0 || j !== 0)) throw Error('Invalid authored tangent frame')
  const up = normal(cube), centre = authored ? [0, radius, 0] as const : scale(up, radius)
  const reference: Vec3 = axis === 0 ? [0, 0, 1] : [1, 0, 0]
  const east = authored ? [1, 0, 0] as const : normal(sub(reference, scale(up, dot(reference, up))))
  const north = authored ? [0, 0, 1] as const : normal(cross(east, up))
  const id = authored ? AUTHORED_SURFACE.siteId : `site:surface:v1:${region.bodyId}:${axis}:${sign}:${i}:${j}`
  const expected = [centre, east, north, normal(centre)]
  if (region.id !== id || ![region.centre, region.east, region.north, region.up].every((v, index) => Array.isArray(v) && v.length === 3 && v.every((n, k) => n === expected[index]![k]))) throw Error('Invalid canonical tangent frame')
  return radius
}
/** Orthographic tangent chart; valid only on its front hemisphere, so the inverse
 * cannot select a second physical location. Region changes never change location. */
export function surfaceToLocal(location: SurfaceLocation, region: SurfaceRegion): readonly [number, number] {
  checkedRegion(region); validateSurfaceLocation(location, region.definition)
  if (location.bodyId !== region.bodyId || dot(location.radialM, region.up) <= 0) throw Error('Location outside tangent chart')
  const delta = sub(location.radialM, region.centre)
  return [dot(delta, region.east), dot(delta, region.north)]
}
export function localToSurface(region: SurfaceRegion, local: readonly [number, number]): SurfaceLocation {
  const radius = checkedRegion(region)
  if (!Array.isArray(local) || local.length !== 2 || !local.every(Number.isFinite) || Math.hypot(...local) >= radius) throw Error('Invalid local surface coordinate')
  const [x, y] = local, h = Math.sqrt((radius - Math.hypot(x, y)) * (radius + Math.hypot(x, y)))
  return { bodyId: region.bodyId, radialM: add(add(scale(region.up, h), scale(region.east, x)), scale(region.north, y)) }
}
/** Deterministic cardinal adjacent-cell probes. Cube-face boundaries are
 * reclassified through the same physical query; no longitude wrapping or pole.
 * Authored Vesper is a bounded inset in the original grid cell. */
export function neighboringRegions(region: SurfaceRegion): readonly SurfaceRegion[] {
  const radius = checkedRegion(region), { axis, sign, i, j, steps } = region.cell
  const result = new Map<string, SurfaceRegion>()
  const include = (r: SurfaceRegion) => { if (r.id !== region.id) result.set(r.id, r) }
  if (region.id === AUTHORED_SURFACE.siteId) {
    include(classifyRegion({ bodyId: region.bodyId, radialM: region.centre }, region.definition, false))
  } else {
    for (const [di, dj] of [[-1, 0], [1, 0], [0, -1], [0, 1]]) {
      const cube: [number, number, number] = [0, 0, 0]
      cube[axis] = sign
      // Edge cells cover half a ratio cell on each face. Probe just across
      // the seam, rather than jumping a full cell and skipping that strip.
      const coordinate = (value: number) => Math.abs(value) > steps
        ? Math.sign(value) * (1 + .25 / steps)
        : Math.abs(value) === steps ? Math.sign(value) * (1 - .125 / steps) : value / steps
      cube[(axis + 1) % 3] = coordinate(i + di!); cube[(axis + 2) % 3] = coordinate(j + dj!)
      include(surfaceRegion({ bodyId: region.bodyId, radialM: scale(normal(cube), radius) }, region.definition))
    }
    if (region.bodyId === AUTHORED_SURFACE.bodyId && axis === 1 && sign === 1 && i === 0 && j === 0) include(surfaceRegion({ bodyId: region.bodyId, radialM: region.centre }, region.definition))
  }
  return [...result.values()].sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0)
}
