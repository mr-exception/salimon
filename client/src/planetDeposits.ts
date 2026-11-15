import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { definitionDigest, planetDefinition } from './planetDefinition.ts'
import { probeGeology } from './planetGeology.ts'
import { samplePlanetSurface } from './planetSurface.ts'
import { localToSurface, neighboringRegions, surfaceRegion, type SurfaceRegion } from './surfaceCoordinates.ts'
import { cross, dot, normal, scale, sub, type Vec3 } from './math.ts'
import { point2 } from './topDownFrames.ts'
import type { Entity2D } from './topDownRuntime.ts'
import type { ResourceSource } from './resourceInventory.ts'

/** Supplemental resource publication. Existing terrain/publication hashes and
 * legacy authored-only bodies are never rewritten. No biology is generated. */
export const DEPOSIT_RULES = Object.freeze({ algorithm: 'geological-deposits:v1', resourceId: 'mineral', candidates: 4,
  baseProbability: .12, highlandProbability: .18, fractureProbability: .48,
  commonMinGrams: 2000, commonRangeGrams: 10001, rareProbability: .04, rareMinGrams: 30000,
  rareRangeGrams: 30001, halfExtentM: .65, reachM: 2.3, authoredExclusionM: 200 })
export const DEPOSIT_PUBLICATION = definitionDigest(DEPOSIT_RULES)
export const isDepositId = (id: string) => id.startsWith('source:deposit:')
export interface PlanetDeposit { readonly entity: Entity2D; readonly initial: ResourceSource; readonly rare: boolean }
const random = (key: string) => {
  let value = BigInt('0x' + definitionDigest(key))
  value = BigInt.asUintN(64, (value ^ (value >> 30n)) * 0xbf58476d1ce4e5b9n)
  value = BigInt.asUintN(64, (value ^ (value >> 27n)) * 0x94d049bb133111ebn)
  return Number((value ^ (value >> 31n)) >> 11n) / 9007199254740992
}

/** Exactly four candidates per canonical cell; ownership is checked at edges.
 * Independent of callers, neighboring requests, renderer, ship and players. */
export function regionDeposits(region: SurfaceRegion): readonly PlanetDeposit[] {
  localToSurface(region, [0, 0]) // reject forged charts, even for legacy regions
  const d = planetDefinition(region.bodyId, region.definition)
  if (d.version !== 2 || region.id === AUTHORED_SURFACE.siteId) return []
  const result: PlanetDeposit[] = []
  for (let slot = 0; slot < DEPOSIT_RULES.candidates; slot++) {
    const id = `source:deposit:v1:${d.bodyId}:${d.digest}:${region.cell.axis}:${region.cell.sign}:${region.cell.i}:${region.cell.j}:${slot}`
    const key = `${DEPOSIT_PUBLICATION}:${d.domains.resources.seed}:${id}`
    const x = (slot % 2 ? 1 : -1) * (5 + random(key + ':x') * 5)
    const y = (slot < 2 ? 1 : -1) * (5 + random(key + ':y') * 5)
    const location = localToSurface(region, [x, y])
    if (surfaceRegion(location, d).id !== region.id) continue
    if (d.bodyId === AUTHORED_SURFACE.bodyId && location.radialM[1] > 0
      && Math.hypot(location.radialM[0], location.radialM[2]) < DEPOSIT_RULES.authoredExclusionM) continue
    // The entire source and a reachable walking ring must be safe, not just its centre.
    const geology = probeGeology(location, d, DEPOSIT_RULES.halfExtentM + .46)
    if (geology.collision || geology.groundType === 'polar-ice' || samplePlanetSurface(location, d, { radiusM: DEPOSIT_RULES.halfExtentM + .46 }).collision) continue
    const probability = DEPOSIT_RULES.baseProbability + geology.weights.highland * DEPOSIT_RULES.highlandProbability + geology.weights.fracture * DEPOSIT_RULES.fractureProbability
    if (random(key + ':presence') >= probability) continue
    const rare = random(key + ':rarity') < DEPOSIT_RULES.rareProbability
    const remainingGrams = (rare ? DEPOSIT_RULES.rareMinGrams : DEPOSIT_RULES.commonMinGrams)
      + Math.floor(random(key + ':grams') * (rare ? DEPOSIT_RULES.rareRangeGrams : DEPOSIT_RULES.commonRangeGrams))
    result.push({ initial: { id, resourceId: DEPOSIT_RULES.resourceId, remainingGrams }, rare,
      entity: { id, anchor: { frame: 'planet:fixture', chart: region, point: point2(x, y) }, layer: 'terrain', appearance: 'mineral',
        footprint: { halfWidth: DEPOSIT_RULES.halfExtentM, halfHeight: DEPOSIT_RULES.halfExtentM },
        source: { siteId: region.id, kind: 'mineral', archetype: 'banded-outcrop' },
        interaction: { label: rare ? 'Rich banded mineral outcrop' : 'Banded mineral outcrop', action: 'Inspect', reachM: DEPOSIT_RULES.reachM } } })
  }
  return result
}

/** Regenerate trusted initial mass from an ID; saved amounts never define it. */
export function depositFromId(id: string): PlanetDeposit {
  const parts = id.split(':')
  if (parts.length !== 10 || parts.slice(0, 3).join(':') !== 'source:deposit:v1') throw Error('Invalid deposit identity')
  const [, , , bodyId, digest, a, s, i, j] = parts
  const d = planetDefinition(bodyId!, { bodyId: bodyId!, version: 2, digest: digest! })
  const axis = Number(a), sign = Number(s), steps = Math.ceil((d.radiusM + 5) / 64)
  if (![0, 1, 2].includes(axis) || ![-1, 1].includes(sign) || ![Number(i), Number(j)].every(v => Number.isInteger(v) && Math.abs(v) <= steps)) throw Error('Invalid deposit cell')
  const cube: [number, number, number] = [0, 0, 0]
  cube[axis] = sign; cube[(axis + 1) % 3] = Number(i) / steps; cube[(axis + 2) % 3] = Number(j) / steps
  // A face-edge cell's centre can tie with the adjacent face. Rebuild its
  // validated chart directly instead of reclassifying that shared centre.
  const up = normal(cube), referenceAxis: Vec3 = axis === 0 ? [0, 0, 1] : [1, 0, 0]
  const east = normal(sub(referenceAxis, scale(up, dot(referenceAxis, up))))
  const centre = scale(up, d.radiusM + 5)
  const region: SurfaceRegion = { id: `site:surface:v1:${d.bodyId}:${axis}:${sign}:${Number(i)}:${Number(j)}`,
    bodyId: d.bodyId, definition: { bodyId: d.bodyId, version: d.version, digest: d.digest },
    centre, up: normal(centre), east, north: normal(cross(east, up)), cell: { axis, sign, i: Number(i), j: Number(j), steps } }
  const deposit = regionDeposits(region).find(p => p.initial.id === id)
  if (!deposit) throw Error('Unknown generated deposit')
  return deposit
}
export function depositNeighborhood(centre: SurfaceRegion): readonly SurfaceRegion[] {
  const regions = new Map([[centre.id, centre]])
  let frontier = [centre]
  for (let ring = 0; ring < 2; ring++) {
    const next: SurfaceRegion[] = []
    for (const region of frontier) for (const n of neighboringRegions(region)) if (!regions.has(n.id)) { regions.set(n.id, n); next.push(n) }
    frontier = next
  }
  return [...regions.values()].sort((a, b) => a.id.localeCompare(b.id))
}
