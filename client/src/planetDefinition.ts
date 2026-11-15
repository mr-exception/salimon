import { AUTHORED_SURFACE } from './authoredSurface.ts'

/** Packaged fictional fixtures; no observed scientific data or runtime seed selection. */
export type SurfaceBodyId = 'vesper' | 'vesper-moon' | 'caligo'
export type DomainName = 'terrain' | 'geology' | 'resources' | 'visual'
export interface PlanetReference { readonly bodyId: string; readonly version: number; readonly digest: string }
export interface PlanetDefinition extends PlanetReference {
  readonly schema: 1
  readonly provenance: 'illustrative-packaged-fixture'
  readonly radiusM: number
  readonly geometry: 'sphere'
  readonly coordinates: 'surface:v1'
  readonly authored: 'illustrated-landing:v1' | null
  readonly domains: Readonly<Record<DomainName, {
    readonly version: 1 | 2; readonly seed: string; readonly algorithm: string
    readonly parameters: Readonly<Record<string, number | string>>; readonly digest: string
  }>>
}
/** Sorted-key JSON, UTF-8, FNV-1a-64, unsigned 16-digit lower-case hex.
 * Content identifier, not authentication. Full records are also matched against the pack. */
export function definitionDigest(value: unknown): string {
  function canonical(v: unknown): string {
    if (v === null || typeof v === 'string' || typeof v === 'boolean') return JSON.stringify(v)
    if (typeof v === 'number' && Number.isFinite(v)) return JSON.stringify(v)
    if (Array.isArray(v)) return `[${v.map(canonical).join(',')}]`
    if (typeof v === 'object' && v && Object.getPrototypeOf(v) === Object.prototype) return `{${Object.keys(v).sort().map(k => `${JSON.stringify(k)}:${canonical((v as Record<string, unknown>)[k])}`).join(',')}}`
    throw Error('Invalid canonical definition value')
  }
  let hash = 0xcbf29ce484222325n
  for (const byte of new TextEncoder().encode(canonical(value))) hash = BigInt.asUintN(64, (hash ^ BigInt(byte)) * 0x100000001b3n)
  return hash.toString(16).padStart(16, '0')
}
function freeze<T>(value: T): T {
  if (value && typeof value === 'object') { Object.values(value).forEach(freeze); Object.freeze(value) }
  return value
}
function fixture(bodyId: SurfaceBodyId, radiusM: number, seeds: readonly string[], version = 1): PlanetDefinition {
  const domain = (index: number, algorithm: string, parameters: Record<string, number | string>) => {
    const content = { version: (index < 2 && version === 2 ? 2 : 1) as 1 | 2, seed: seeds[index]!, algorithm, parameters }
    return { ...content, digest: definitionDigest(content) }
  }
  const content = {
    bodyId, version, schema: 1 as const, provenance: 'illustrative-packaged-fixture' as const,
    radiusM, geometry: 'sphere' as const, coordinates: 'surface:v1' as const,
    authored: bodyId === 'vesper' ? 'illustrated-landing:v1' as const : null,
    domains: {
      terrain: domain(0, version === 1 ? 'flat-sphere:v1' : 'geological-sphere:v2', { heightM: 0, regionRatioMetres: 64, authoredDigest: bodyId === 'vesper' ? definitionDigest(AUTHORED_SURFACE) : 'none', ...(version === 2 ? { reliefM: bodyId === 'vesper-moon' ? 180 : bodyId === 'caligo' ? 360 : 240, fractureM: bodyId === 'caligo' ? 16 : 12, geologyAlgorithm: 'provinces:v2' } : {}) }),
      geology: domain(1, version === 1 ? 'unclassified:v1' : 'provinces:v2', version === 1 ? { substrate: 'unspecified' } : { scaleM: 18000, ice: bodyId === 'vesper-moon' ? 1 : 0 }),
      resources: domain(2, 'authored-only:v1', { sources: bodyId === 'vesper' ? 'landing:mineral:01,landing:biology:01' : 'none' }),
      visual: domain(3, 'illustrated-fixture:v1', { palette: bodyId }),
    },
  }
  return freeze({ ...content, digest: definitionDigest(content) })
}
export const PLANET_DEFINITIONS: readonly PlanetDefinition[] = Object.freeze([
  fixture('vesper', 6200000, ['0000000000000101', '0000000000000102', '0000000000000103', '0000000000000104']),
  fixture('vesper-moon', 1400000, ['0000000000000201', '0000000000000202', '0000000000000203', '0000000000000204']),
  fixture('caligo', 9000000, ['0000000000000301', '0000000000000302', '0000000000000303', '0000000000000304']),
])
export const GEOLOGICAL_PLANET_DEFINITIONS: readonly PlanetDefinition[] = Object.freeze(PLANET_DEFINITIONS.map(p => fixture(p.bodyId as SurfaceBodyId, p.radiusM, Object.values(p.domains).map(d => d.seed), 2)))
// Published values are deliberately independent of the builder above. A changed
// parameter/algorithm needs a new publication and an approved save migration.
const PUBLISHED: Readonly<Record<string, string>> = {
  vesper: 'f3d9f26ee5ae532a', 'vesper-moon': '8a9aa7c371d8ea74', caligo: '45aac4d9d0e82a25',
}
const PUBLISHED_V2: Readonly<Record<string, string>> = { vesper: '263916104a5852c0', 'vesper-moon': 'ee38e47c056e3a66', caligo: '71194ec053cb6da7' }
/** New destinations explicitly bind this revision. Missing references remain v1. */
export function geologicalDefinition(bodyId: string): PlanetDefinition {
  const definition = GEOLOGICAL_PLANET_DEFINITIONS.find(p => p.bodyId === bodyId)
  if (!definition) throw Error('Unsupported canonical planet definition')
  return planetDefinition(bodyId, definition)
}
export function planetDefinition(bodyId: string, reference?: PlanetReference): PlanetDefinition {
  const definition = (reference?.version === 2 ? GEOLOGICAL_PLANET_DEFINITIONS : PLANET_DEFINITIONS).find(p => p.bodyId === bodyId)
  if (!definition) throw Error('Unsupported canonical planet definition')
  if (definition.digest !== (definition.version === 2 ? PUBLISHED_V2 : PUBLISHED)[bodyId]) throw Error('Packaged planet definition hash mismatch')
  if (reference && (reference.bodyId !== bodyId || reference.version !== definition.version || reference.digest !== definition.digest)) throw Error('Planet definition version/hash mismatch; approved migration required')
  return definition
}
/** Validate untrusted complete records, including domain digests and unknown fields. */
export function validatePlanetDefinition(value: PlanetDefinition): PlanetDefinition {
  const expected = planetDefinition(value?.bodyId, value)
  if (definitionDigest(value) !== definitionDigest(expected)) throw Error('Planet definition content mismatch')
  // A digest is not a security boundary: compare canonical records via sorted keys.
  const equal = (a: unknown, b: unknown): boolean => {
    if (a === b) return true
    if (!a || !b || typeof a !== 'object' || typeof b !== 'object') return false
    const aa = a as Record<string, unknown>, bb = b as Record<string, unknown>
    return Object.keys(aa).length === Object.keys(bb).length && Object.keys(aa).every(k => Object.hasOwn(bb, k) && equal(aa[k], bb[k]))
  }
  if (!equal(value, expected)) throw Error('Planet definition content mismatch')
  return expected
}
