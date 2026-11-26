export const assetCategories = ['character', 'structure', 'prop', 'terrain', 'effect', 'icon'] as const
const lifecycle = ['placeholder', 'draft', 'approved', 'production', 'deprecated'] as const
type Category = typeof assetCategories[number]
type Status = typeof lifecycle[number]
type Point = readonly [number, number]

export interface AssetPart {
  readonly id: string
  readonly source: string
  readonly layer: string
  /** Pixel rectangle of nonzero alpha, excluding padding. */
  readonly content?: readonly [number, number, number, number]
}

export interface AssetEntry {
  readonly id: string
  readonly revision: number
  readonly category: Category
  readonly status: Status
  readonly essential: boolean
  readonly pivot: Point
  readonly physicalSize?: Point
  readonly density?: number
  readonly footprintRef?: string
  readonly parts: readonly AssetPart[]
  readonly anchors: Readonly<Record<string, Point | string>>
  readonly states: Readonly<Record<string, readonly string[]>>
  readonly provenance: {
    readonly origin: string
    readonly license: string
    readonly attribution: string
    readonly redistributable: boolean
    readonly references: readonly string[]
  }
}

export interface AssetRegistry {
  readonly schemaVersion: 1
  readonly assets: readonly AssetEntry[]
}

export interface AssetSource {
  readonly alphaBounds?: AssetPart['content']
}

export interface AssetValidation {
  readonly source: (path: string) => AssetSource | undefined
  readonly footprint: (ref: string) => Point | undefined
  readonly anchor: (ref: string) => boolean
}

const stableId = /^[a-z][a-z0-9]*(?:[.:-][a-z0-9]+)*$/
const sourcePath = /^(?:[a-z0-9_-]+\/)*[a-z0-9_-]+\.(png|svg)$/
const text = (value: unknown): value is string => typeof value === 'string' && value.trim().length > 0
const id = (value: unknown): value is string => text(value) && stableId.test(value)
const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value)
const tuple = (value: unknown, length: number, test: (n: number) => boolean): value is number[] =>
  Array.isArray(value) && value.length === length && value.every(n => typeof n === 'number' && Number.isFinite(n) && test(n))
const positive = (n: number) => n > 0
const normalized = (n: number) => n >= 0 && n <= 1
const integer = (n: unknown): n is number => typeof n === 'number' && Number.isSafeInteger(n) && n > 0

export function validateAssetRegistry(input: unknown, lookup: AssetValidation): AssetRegistry {
  function require(condition: unknown, message: string): asserts condition {
    if (!condition) throw new Error(`Asset registry: ${message}`)
  }
  require(record(input) && input.schemaVersion === 1 && Array.isArray(input.assets), 'expected schema version 1 and assets array')
  const ids = new Set<string>()
  for (const entry of input.assets as unknown[]) {
    require(record(entry) && id(entry.id), 'invalid asset ID')
    const name = entry.id
    require(!ids.has(name), `${name}: duplicate asset ID`)
    ids.add(name)
    require(integer(entry.revision) && assetCategories.some(category => category === entry.category) &&
      lifecycle.some(status => status === entry.status) && typeof entry.essential === 'boolean' &&
      tuple(entry.pivot, 2, normalized), `${name}: invalid revision, category, lifecycle, essential flag or pivot`)
    const physical = entry.category !== 'icon'
    if (physical) {
      require(tuple(entry.physicalSize, 2, positive) && integer(entry.density), `${name}: invalid physical size or density`)
    } else {
      require(entry.physicalSize === undefined && entry.density === undefined && entry.footprintRef === undefined,
        `${name}: icons cannot specify domain dimensions`)
    }
    if (entry.footprintRef !== undefined) {
      require(id(entry.footprintRef) && tuple(entry.physicalSize, 2, positive), `${name}: invalid footprint reference`)
      const domainSize = lookup.footprint(entry.footprintRef)
      require(domainSize && domainSize.every((metres, i) => Math.abs(metres - (entry.physicalSize as number[])[i]) < 1e-8),
        `${name}: physical size differs from domain footprint ${entry.footprintRef}`)
    }
    require(record(entry.provenance), `${name}: missing provenance`)
    const { origin, license, attribution, redistributable, references } = entry.provenance
    require(text(origin) && text(license) && text(attribution) && typeof redistributable === 'boolean' &&
      Array.isArray(references) && references.every(text), `${name}: incomplete provenance or rights`)
    require((entry.status !== 'approved' && entry.status !== 'production') || redistributable,
      `${name}: approved and production assets require redistribution rights`)
    require(Array.isArray(entry.parts) && entry.parts.length > 0, `${name}: no artwork parts`)
    const parts = new Set<string>()
    for (const part of entry.parts as unknown[]) {
      require(record(part) && id(part.id) && id(part.layer) && text(part.source) && sourcePath.test(part.source),
        `${name}: invalid part ID, layer or source path`)
      require(!parts.has(part.id), `${name}: duplicate part ${part.id}`)
      parts.add(part.id)
      const source = lookup.source(part.source)
      require(source, `${name}: missing source ${part.source}`)
      if (part.source.endsWith('.png')) {
        require(tuple(part.content, 4, n => Number.isSafeInteger(n) && n >= 0) &&
          source.alphaBounds && source.alphaBounds.every((n, i) => n === (part.content as number[])[i]),
          `${name}: invalid or mismatched alpha bounds for ${part.source}`)
        if (physical) {
          const size = entry.physicalSize as number[]
          const density = entry.density as number
          require(part.content[2] === Math.round(size[0] * density) && part.content[3] === Math.round(size[1] * density),
            `${name}: unpadded pixels do not match physical size and density`)
        }
      } else {
        require(!physical && part.content === undefined, `${name}: SVG sources are for icons`)
      }
    }
    require(record(entry.anchors) && record(entry.states), `${name}: missing anchors or states map`)
    for (const [key, ref] of Object.entries(entry.anchors)) {
      require(id(key) && (tuple(ref, 2, normalized) || (id(ref) && lookup.anchor(ref))), `${name}: unknown anchor ${key}`)
    }
    for (const [key, refs] of Object.entries(entry.states)) {
      require(id(key) && Array.isArray(refs) && refs.length > 0 && new Set(refs).size === refs.length &&
        refs.every(ref => parts.has(ref)), `${name}: invalid state parts for ${key}`)
    }
  }
  return input as unknown as AssetRegistry
}

export type AssetResolution<T> =
  | { readonly kind: 'available'; readonly value: T }
  | { readonly kind: 'placeholder'; readonly id: string }
  | { readonly kind: 'retry'; readonly id: string; readonly cause: unknown }

export async function resolveAsset<T>(entry: AssetEntry, load: () => Promise<T>): Promise<AssetResolution<T>> {
  try {
    return { kind: 'available', value: await load() }
  } catch (cause) {
    return entry.essential ? { kind: 'retry', id: entry.id, cause } : { kind: 'placeholder', id: entry.id }
  }
}
