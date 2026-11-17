import { definitionDigest, planetDefinition, type PlanetReference } from './planetDefinition.ts'
import { surfaceRegion, validateSurfaceLocation, type SurfaceLocation } from './surfaceCoordinates.ts'
import { MAX_FIELD_MARKS, playerSurface, regionLandmarks, type FieldMark } from './exploration.ts'
import { AUTHORED_SURFACE } from './authoredSurface.ts'
import type { TopDownState } from './topDownRuntime.ts'
import type { SiteStates } from './surfaceSite.ts'

export const MAX_VISITS = 128
export interface RegionVisit {
  readonly regionId: string
  readonly location: SurfaceLocation
  readonly definition: PlanetReference
}
export interface ExpeditionJournal {
  readonly marks: readonly FieldMark[]
  /** Oldest to newest physical visit; region residency/rendering never records one. */
  readonly visits: readonly RegionVisit[]
}
export const emptyJournal = (): ExpeditionJournal => ({ marks: [], visits: [] })

export function recordVisit(journal: ExpeditionJournal, world: TopDownState): ExpeditionJournal {
  const location = playerSurface(world)
  if (!location) return journal
  const region = surfaceRegion(location, world.scoutLocation.surface.definition)
  if (journal.visits.at(-1)?.regionId === region.id) return journal
  return { ...journal, visits: [...journal.visits.filter(v => v.regionId !== region.id),
    { regionId: region.id, location, definition: region.definition }].slice(-MAX_VISITS) }
}
function fields(value: unknown, expected: string): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).sort().join() !== expected) throw Error('Invalid expedition journal fields')
  return value as Record<string, unknown>
}
/** Validate physical identities against the save's bound publications before any
 * state is published. Unknown definitions are never silently regenerated. */
export function validateJournal(value: unknown, sites: SiteStates): ExpeditionJournal {
  const raw = fields(value, 'marks,visits')
  if (!Array.isArray(raw.marks) || raw.marks.length > MAX_FIELD_MARKS || !Array.isArray(raw.visits) || raw.visits.length > MAX_VISITS) throw Error('Expedition journal limit exceeded')
  const known = new Map(Object.values(sites).map(s => [s.address.bodyId, planetDefinition(s.address.bodyId, s.address.definition).digest]))
  const physical = (location: SurfaceLocation, reference: PlanetReference) => {
    fields(location, 'bodyId,radialM'); fields(reference, 'bodyId,digest,version')
    const d = planetDefinition(location.bodyId, reference)
    validateSurfaceLocation(location, reference)
    if (known.get(d.bodyId) !== d.digest) throw Error('Journal body/publication has not been visited')
    return surfaceRegion(location, reference)
  }
  const marks = raw.marks.map(value => {
    const mark = fields(value, 'definition,id,landmarkId,location,name,note,provenance,version') as unknown as FieldMark
    const region = physical(mark.location, mark.definition)
    if (mark.version !== 1 || typeof mark.name !== 'string' || !mark.name.trim() || mark.name.length > 48 || typeof mark.note !== 'string' || mark.note.length > 160 || /[\u0000-\u001f]/.test(mark.name + mark.note)) throw Error('Invalid field note')
    let identity: string
    if (mark.landmarkId === null) {
      identity = definitionDigest({ definition: mark.definition.digest, bodyId: mark.location.bodyId, coordinateDm: mark.location.radialM.map(n => Math.round(n * 10)) })
      if (mark.provenance !== (mark.note.trim() ? 'user-note' : 'personally-observed-estimate')) throw Error('Invalid field provenance')
    } else {
      if (typeof mark.landmarkId !== 'string' || mark.provenance !== 'known-physical-source') throw Error('Invalid landmark observation')
      const authored = mark.location.bodyId === AUTHORED_SURFACE.bodyId && region.id === AUTHORED_SURFACE.siteId && AUTHORED_SURFACE.objects.some(o => `landmark:authored:v1:${AUTHORED_SURFACE.bodyId}:${o[0]}` === mark.landmarkId && o[0].startsWith('rock:'))
      const generated = regionLandmarks(region).some(l => l.id === mark.landmarkId && l.location.radialM.every((n, i) => n === mark.location.radialM[i]))
      if (!authored && !generated) throw Error('Unknown observed landmark')
      identity = mark.landmarkId
    }
    if (mark.id !== `mark:v1:${identity}`) throw Error('Invalid field mark identity')
    return structuredClone(mark)
  }).sort((a, b) => a.id.localeCompare(b.id))
  const visits = raw.visits.map(value => {
    const visit = fields(value, 'definition,location,regionId') as unknown as RegionVisit
    if (physical(visit.location, visit.definition).id !== visit.regionId) throw Error('Invalid visited region identity')
    return structuredClone(visit)
  })
  if (new Set(marks.map(m => m.id)).size !== marks.length || new Set(visits.map(v => v.regionId)).size !== visits.length) throw Error('Duplicate expedition record')
  return { marks, visits }
}
