export const VIEWPORT = { width: 960, height: 600 } as const
export const CLIP = { x: 22, y: 22, width: 916, height: 556 } as const
export const BACKDROP = '#15222c'
export const SEED = 939026
export const ATLAS_LIMIT = 2048
export const MEGABYTE = 1024 * 1024

export type Scenario = 'scout' | 'surface'
export type Density = 'current' | '128'
export type Renderer = 'svg' | 'pixi'
export type Asset = 'deck' | 'hull' | 'chair' | 'core' | 'bench' | 'storage' | 'bed' | 'terrain'
  | 'stone' | 'mineral' | 'flora' | 'helmet' | 'torso' | 'arm' | 'boot' | 'light' | 'particle'

export interface DrawObject {
  id: string
  asset: Asset
  x: number
  y: number
  zoom: number
  rotation: number
  phase: number
  motion?: 'stride' | 'pulse' | 'drift'
  hitRadius?: number
}
export type FrameObject = DrawObject & { alpha: number }

export const texelsPerMetre = (asset: Asset, density: Density) => density === '128' ? 128 : asset === 'terrain' ? 32 : 64

export function scene(scenario: Scenario, seed = SEED): DrawObject[] {
  let state = seed >>> 0
  const random = () => ((state = (Math.imul(1664525, state) + 1013904223) >>> 0) / 2 ** 32)
  const objects: DrawObject[] = []
  const zoom = scenario === 'scout' ? 32 : 20
  const place = (id: string, asset: Asset, x: number, y: number, options: Partial<DrawObject> = {}) => {
    objects.push({ id, asset, x, y, zoom, rotation: 0, phase: 0, ...options })
  }
  if (scenario === 'scout') {
    place('hull', 'hull', 480, 301)
    place('deck', 'deck', 480, 301)
    place('chair', 'chair', 480, 185)
    place('core', 'core', 531, 358)
    place('bench', 'bench', 536, 326, { hitRadius: 24 })
    place('fabricator', 'bench', 429, 428)
    place('storage', 'storage', 527, 434)
    place('bunk-port', 'bed', 414, 293)
    place('bunk-starboard', 'bed', 550, 293)
  } else {
    for (let y = 60; y < 650; y += 160) {
      for (let x = 70; x < 1040; x += 160) {
        place('terrain-' + x + '-' + y, 'terrain', x, y, { rotation: Math.round(random() * 3) * 90 })
      }
    }
    const assets = ['stone', 'mineral', 'flora'] as const
    for (let index = 0; index < 180; index++) {
      place('surface-detail-' + index, assets[Math.floor(random() * assets.length)]!,
        20 + random() * 920, 20 + random() * 540, { rotation: random() * 360, zoom: 8 + random() * 12 })
    }
    place('hull', 'hull', 480, 300, { rotation: -12 })
    place('deck', 'deck', 480, 300, { rotation: -12 })
    place('bench', 'bench', 515, 310, { hitRadius: 18 })
    place('sample-outcrop', 'mineral', 680, 420, { hitRadius: 22 })
  }
  for (const [index, [asset, dx, dy]] of ([
    ['boot', -8, 11], ['boot', 8, 11], ['torso', 0, 0],
    ['arm', -11, 1], ['arm', 11, 1], ['helmet', 0, -9],
  ] as const).entries()) {
    place('character-' + index, asset, (scenario === 'scout' ? 480 : 603) + dx, 377 + dy,
      { zoom: 34, motion: 'stride', phase: index % 2 ? Math.PI : 0 })
  }
  for (const [index, [x, y]] of [[459, 218], [532, 342], [444, 432], [600, 374]].entries()) {
    place('light-' + index, 'light', x!, y!, { zoom: 36, phase: index * 1.2, motion: 'pulse' })
  }
  const particleCount = scenario === 'scout' ? 56 : 220
  for (let index = 0; index < particleCount; index++) {
    place('particle-' + index, 'particle', 18 + random() * 924, 20 + random() * 550,
      { zoom: 12 + random() * 18, phase: random() * 6.28, motion: 'drift' })
  }
  // This edge sprite provides a visible clipping and input boundary on both adapters.
  place('edge-probe', 'mineral', 26, 65, { zoom: 24, hitRadius: 24 })
  return objects
}

export function pose(object: DrawObject, frame: number): FrameObject {
  const wave = Math.sin(frame * .09 + object.phase)
  if (object.motion === 'stride') return { ...object, y: object.y + wave * 2, rotation: object.rotation + wave * 9, alpha: 1 }
  if (object.motion === 'pulse') return { ...object, alpha: .54 + wave * .12 }
  if (object.motion === 'drift') return {
    ...object, x: object.x + wave * 11, y: object.y + Math.cos(frame * .045 + object.phase) * 7, alpha: .33 + wave * .18,
  }
  return { ...object, alpha: 1 }
}

export function hit(objects: readonly DrawObject[], x: number, y: number): string | null {
  if (x < CLIP.x || y < CLIP.y || x >= CLIP.x + CLIP.width || y >= CLIP.y + CLIP.height) return null
  for (let index = objects.length - 1; index >= 0; index--) {
    const object = objects[index]!
    if (object.hitRadius !== undefined && Math.hypot(x - object.x, y - object.y) <= object.hitRadius) return object.id
  }
  return null
}

export interface BitmapSize { width: number; height: number }
export interface AtlasEntry extends BitmapSize { page: number; x: number; y: number }
export function packAtlases<T extends string>(sizes: readonly (BitmapSize & { key: T })[]) {
  const entries = new Map<T, AtlasEntry>()
  const pages: BitmapSize[] = [{ width: 0, height: 0 }]
  const padding = 2
  let x = 0, y = 0, rowHeight = 0
  for (const bitmap of [...sizes].sort((a, b) => b.height - a.height || a.key.localeCompare(b.key))) {
    const { key, width, height } = bitmap
    const paddedWidth = width + padding * 2, paddedHeight = height + padding * 2
    if (!Number.isInteger(width) || !Number.isInteger(height) || width <= 0 || height <= 0
      || paddedWidth > ATLAS_LIMIT || paddedHeight > ATLAS_LIMIT || entries.has(key)) throw Error('Invalid atlas bitmap: ' + key)
    if (x + paddedWidth > ATLAS_LIMIT) { x = 0; y += rowHeight; rowHeight = 0 }
    if (y + paddedHeight > ATLAS_LIMIT) { pages.push({ width: 0, height: 0 }); x = 0; y = 0; rowHeight = 0 }
    const page = pages.at(-1)!
    entries.set(key, { page: pages.length - 1, x: x + padding, y: y + padding, width, height })
    x += paddedWidth
    rowHeight = Math.max(rowHeight, paddedHeight)
    page.width = Math.max(page.width, x)
    page.height = Math.max(page.height, y + rowHeight)
  }
  return { entries, pages: sizes.length ? pages : [] }
}

export function distribution(values: readonly number[]) {
  const sorted = [...values].sort((a, b) => a - b)
  const quantile = (q: number) => sorted[Math.ceil(q * sorted.length) - 1] ?? 0
  return { p50: quantile(.5), p95: quantile(.95), p99: quantile(.99), max: sorted.at(-1) ?? 0 }
}
