import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { ORIGIN, relativeM } from './coordinates.ts'
import { definitionDigest, planetDefinition, type PlanetReference } from './planetDefinition.ts'
import { sampleGeology } from './planetGeology.ts'
import { depositNeighborhood } from './planetDeposits.ts'
import { localToSurface, surfaceRegion, surfaceToLocal, validateSurfaceLocation, type SurfaceLocation, type SurfaceRegion } from './surfaceCoordinates.ts'
import { inPlanet, point2, relativeAnchor, worldToView, type Anchor } from './topDownFrames.ts'
import { PLAYER_ID, type TopDownState } from './topDownRuntime.ts'

export interface Landmark {
  readonly id: string
  readonly label: string
  readonly location: SurfaceLocation
  readonly definition: PlanetReference
  readonly footprintM: readonly [number, number]
  readonly physicalSource: string
}
export interface FieldMark {
  readonly version: 1
  readonly id: string
  readonly location: SurfaceLocation
  readonly definition: PlanetReference
  readonly landmarkId: string | null
  readonly provenance: 'known-physical-source' | 'personally-observed-estimate' | 'user-note'
  readonly name: string
  readonly note: string
}
export const MAX_FIELD_MARKS = 256
export const INSPECTION_RANGE_M = 12
export const LOCAL_FIX_RANGE_M = 384

/** A sparse interpretation of existing physical relief, never extra obstacles or
 * loot. One canonical candidate is admitted only on a pronounced rocky grade.
 * No quota: flat/icy/legacy regions remain empty. Ownership rejects face ties. */
export function regionLandmarks(region: SurfaceRegion): readonly Landmark[] {
  const location = localToSurface(region, [0, 0]), d = planetDefinition(region.bodyId, region.definition)
  if (d.version !== 2 || surfaceRegion(location, d).id !== region.id) return []
  const sample = sampleGeology(location, d)
  if (sample.groundType !== 'fractured-rock' || sample.grade < .7) return []
  // Admit isolated pronounced grade crests, rather than labeling every rocky
  // cell in a province. The four probes use the same immutable physical field.
  if ([[12,0],[-12,0],[0,12],[0,-12]].some(([x,y]) => sampleGeology(localToSurface(region,[x!,y!]),d).grade >= sample.grade)) return []
  const id = `landmark:relief:v1:${d.digest}:${region.id}`
  return [{ id, label: 'Fractured ridge', location, definition: region.definition,
    footprintM: [3, 3], physicalSource: sample.geologyId }]
}
export function playerSurface(state: TopDownState): SurfaceLocation | null {
  const player = state.entities.find(e => e.id === PLAYER_ID)!
  if (!state.scoutLocation.binding || player.anchor.frame !== 'planet:fixture') return null
  const address = state.scoutLocation.surface
  const chart = state.frames.surface?.chart ?? surfaceRegion(address, address.definition)
  const [x, y] = relativeM(inPlanet(player.anchor, state.frames), ORIGIN)
  return localToSurface(chart, [x, y])
}
export function localLandmarks(state: TopDownState): readonly Landmark[] {
  const player = playerSurface(state)
  if (!player) return []
  const address = state.scoutLocation.surface, definition = planetDefinition(player.bodyId, address.definition)
  const generated = depositNeighborhood(surfaceRegion(player, definition)).flatMap(regionLandmarks)
  // Saved authored positions/removals are authoritative, not a duplicate map.
  const authored = state.entities.filter(e => e.id.startsWith('rock:') && e.anchor.frame === 'planet:fixture').map(e => {
    const chart = e.anchor.chart ?? surfaceRegion(address, definition)
    const [x, y] = relativeM(e.anchor.point, ORIGIN)
    return { id: `landmark:authored:v1:${AUTHORED_SURFACE.bodyId}:${e.id}`, label: `${e.id.slice(5)} boulder`,
      location: localToSurface(chart, [x, y]), definition, footprintM: [e.footprint!.halfWidth, e.footprint!.halfHeight] as const, physicalSource: e.id }
  })
  return [...authored, ...generated].filter(l => surfaceDistance(player, l.location) <= INSPECTION_RANGE_M)
}
export function surfaceDistance(a: SurfaceLocation, b: SurfaceLocation): number {
  if (a.bodyId !== b.bodyId) return Infinity
  return Math.hypot(...a.radialM.map((n, i) => n - b.radialM[i]!))
}
export function surfaceAnchor(location: SurfaceLocation, state: TopDownState): Anchor {
  const chart = state.frames.surface?.chart ?? surfaceRegion(state.scoutLocation.surface, state.scoutLocation.surface.definition)
  return { frame: 'planet:fixture', chart, point: point2(...surfaceToLocal(location, chart)) }
}
/** Explicit commands only. This bounded session payload is P07's durable-write
 * input; loading/rendering a region never calls it or creates known places. */
export function createFieldMark(state: TopDownState, name: string, note: string, landmarkId?: string): FieldMark {
  const location = playerSurface(state)
  if (!location || state.scoutLocation.ship.mode === 'destroyed') throw Error('Field marking requires a living surface expedition.')
  if (typeof name !== 'string' || typeof note !== 'string' || name.length > 48 || note.length > 160 || /[\u0000-\u001f]/.test(name + note)) throw Error('Use a name up to 48 characters and a note up to 160 characters.')
  const landmark = landmarkId ? localLandmarks(state).find(l => l.id === landmarkId) : undefined
  if (landmarkId && !landmark) throw Error('Landmark is outside local inspection range.')
  const definition = planetDefinition(location.bodyId, state.scoutLocation.surface.definition)
  validateSurfaceLocation(location, definition)
  const target = landmark?.location ?? location
  const identity = landmark?.id ?? definitionDigest({definition: definition.digest, bodyId: target.bodyId, coordinateDm: target.radialM.map(n => Math.round(n * 10))})
  return { version: 1, id: `mark:v1:${identity}`, location: target, definition: {bodyId:definition.bodyId,version:definition.version,digest:definition.digest},
    landmarkId: landmark?.id ?? null, provenance: landmark ? 'known-physical-source' : note.trim() ? 'user-note' : 'personally-observed-estimate',
    name: name.trim() || landmark?.label || 'Field location', note: note.trim() }
}
export function recordFieldMark(marks: readonly FieldMark[], mark: FieldMark): readonly FieldMark[] {
  if (!marks.some(m => m.id === mark.id) && marks.length >= MAX_FIELD_MARKS) throw Error('Field notebook full (256 marks).')
  return [...marks.filter(m => m.id !== mark.id), structuredClone(mark)].sort((a, b) => a.id.localeCompare(b.id))
}
export interface WalkingFix { readonly home: string; readonly travelledM: number }
const homeKey = (state: TopDownState) => `${state.scoutLocation.siteId}:${JSON.stringify(state.scoutLocation.surface)}`
/** Visual reacquisition near the hull starts a new local dead-reckoning budget.
 * Reload away from home has no fix; neither proximity nor loading distant data
 * silently restores a spent walking estimate. P07 may later persist its inputs. */
export function updateWalkingFix(before: TopDownState, after: TopDownState, fix: WalkingFix | null): WalkingFix | null {
  const current = playerSurface(after)
  if (!current) return null
  const distance = surfaceDistance(current, after.scoutLocation.surface)
  if (distance <= 24) return {home: homeKey(after), travelledM: distance}
  const previous = playerSurface(before)
  if (!previous || !fix || fix.home !== homeKey(after)) return null
  const travelledM = fix.travelledM + surfaceDistance(previous, current)
  return travelledM <= LOCAL_FIX_RANGE_M ? {home: fix.home, travelledM} : null
}
export interface ScoutGuidance { readonly available: boolean; readonly text: string; readonly direction?: readonly [number, number] }
/** A bounded walking estimate relative to the actual landed Scout. No orbital
 * sensor, distant target lookup, exact range, autopilot or all-planet fix. */
export function scoutGuidance(state: TopDownState, fix: WalkingFix | null = null): ScoutGuidance {
  const player = playerSurface(state)
  if (!player) return {available:false,text:'Scout guidance unavailable: leave the landed cabin for a local walking estimate.'}
  const distance = surfaceDistance(player, state.scoutLocation.surface)
  if (distance > LOCAL_FIX_RANGE_M || !fix || fix.home !== homeKey(state) || fix.travelledM > LOCAL_FIX_RANGE_M) return {available:false,text:'Scout guidance unavailable: no reliable local walking fix (384 m travel budget). Retrace your route to visually reacquire the hull.'}
  const anchor = state.entities.find(e => e.id === PLAYER_ID)!.anchor
  const vector = worldToView(relativeAnchor({frame:'ship:scout',point:point2(0,0)}, anchor, state.frames), state.frames)
  const range = Math.hypot(...vector), uncertainty = 5 + Math.ceil(fix.travelledM / 32)
  if (range < 8) return {available:true,text:'Scout nearby · local walking estimate (±5 m). Look for the hull and aft door.'}
  const sectors = ['↑ ahead','↗ ahead-right','→ right','↘ behind-right','↓ behind','↙ behind-left','← left','↖ ahead-left']
  const sector = (Math.round(Math.atan2(vector[0], -vector[1]) / (Math.PI / 4)) + 8) % 8
  return {available:true,direction:vector,text:`${sectors[sector]} · Scout roughly ${Math.round(range / 10) * 10} m (±${uncertainty} m). Local walking estimate; no global position fix.`}
}
