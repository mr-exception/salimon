import { createHash } from 'node:crypto'
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { PNG } from 'pngjs'
import { SaxesParser } from 'saxes'
import sharp from 'sharp'
import { authoringSources, readAssetRegistry } from './asset-sources.ts'
import type { AssetPart, AssetRegistry } from '../src/rendering/assetRegistry.ts'

const defaultLimit = 2048
const extrusion = 1
const inset = extrusion + 1
const sourceLimit = 4096
const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0

interface Raster {
  key: string
  pixels: PNG
  content: NonNullable<AssetPart['content']>
}
interface Placement extends Raster {
  x: number
  y: number
}
interface Page {
  parts: Placement[]
  width: number
  height: number
}

function pack(rasters: Raster[], limit: number): Page[] {
  const pages: Page[] = []
  const newPage = () => {
    const page: Page = { parts: [], width: 0, height: 0 }
    pages.push(page)
    return page
  }
  if (!rasters.length) return pages
  let page = newPage(), x = 0, y = 0, rowHeight = 0
  const ordered = [...rasters].sort((a, b) => b.content[3] - a.content[3] || compare(a.key, b.key))
  for (const raster of ordered) {
    const width = raster.content[2] + 2 * inset, height = raster.content[3] + 2 * inset
    if (width > limit || height > limit) throw new Error(`Atlas input ${raster.key} exceeds ${limit}px including gutters`)
    if (x + width > limit) {
      x = 0
      y += rowHeight
      rowHeight = 0
    }
    if (y + height > limit) {
      page = newPage()
      x = y = rowHeight = 0
    }
    page.parts.push({ ...raster, x: x + inset, y: y + inset })
    x += width
    rowHeight = Math.max(rowHeight, height)
    page.width = Math.max(page.width, x)
    page.height = Math.max(page.height, y + height)
  }
  return pages
}

function rasterizePage(page: Page): Buffer {
  const rgba = Buffer.alloc(page.width * page.height * 4)
  for (const { x, y, pixels, content: [sx, sy, w, h] } of page.parts) {
    for (let row = -extrusion; row < h + extrusion; row++) {
      for (let col = -extrusion; col < w + extrusion; col++) {
        const source = ((sy + Math.max(0, Math.min(row, h - 1))) * pixels.width + sx + Math.max(0, Math.min(col, w - 1))) * 4
        const target = ((y + row) * page.width + x + col) * 4
        pixels.data.copy(rgba, target, source, source + 4)
      }
    }
  }
  return rgba
}

const svgElements = new Set(['svg', 'g', 'path', 'rect', 'circle', 'ellipse', 'polygon', 'polyline', 'line', 'title', 'desc'])
const svgAttributes = new Set(['xmlns', 'viewBox', 'width', 'height', 'fill', 'stroke', 'stroke-width', 'stroke-linejoin',
  'stroke-linecap', 'fill-rule', 'opacity', 'fill-opacity', 'stroke-opacity', 'transform', 'd', 'x', 'y', 'rx', 'ry',
  'cx', 'cy', 'r', 'points', 'x1', 'y1', 'x2', 'y2'])

function iconSize(source: Buffer, limit: number): [number, number] {
  let size: [number, number] | undefined
  const parser = new SaxesParser()
  parser.on('doctype', () => { throw new Error('SVG icon contains a doctype') })
  parser.on('processinginstruction', () => { throw new Error('SVG icon contains a processing instruction') })
  parser.on('opentag', ({ name, attributes }) => {
    if (!svgElements.has(name) || Object.entries(attributes).some(([key, value]) =>
      !svgAttributes.has(key) || /url\s*\(/i.test(value) ||
      (key === 'xmlns' && value !== 'http://www.w3.org/2000/svg'))) throw new Error('SVG icon contains unsupported markup')
    if (!size) {
      if (name !== 'svg' || attributes.xmlns !== 'http://www.w3.org/2000/svg') throw new Error('SVG icon needs an SVG root')
      const width = Number(attributes.width), height = Number(attributes.height)
      if (![width, height].every(n => Number.isInteger(n) && n > 0 && n <= limit)) throw new Error('SVG icon dimensions exceed limit or are invalid')
      size = [width, height]
    }
  })
  parser.write(new TextDecoder('utf-8', { fatal: true }).decode(source)).close()
  if (!size) throw new Error('Missing SVG icon root')
  return size
}

const shippingStatuses = new Set(['placeholder', 'approved', 'production'])
const fileName = (data: Buffer, extension: string) => `${createHash('sha256').update(data).digest('hex')}.${extension}`

export async function buildAssetPackage(
  registry: AssetRegistry,
  load: (path: string) => Promise<Buffer>,
  limit = defaultLimit,
) {
  if (!Number.isInteger(limit) || limit < 2 * inset + 1 || limit > defaultLimit) throw new Error('Invalid atlas page limit')
  const files = new Map<string, Buffer>()
  const pages: { category: string; url: string; width: number; height: number }[] = []
  const placements = new Map<string, { url: string; rect: readonly [number, number, number, number]; offset: readonly [number, number] }>()
  const groups = new Map<string, Raster[]>()
  let decodedBytes = 0
  const entries = registry.assets.filter(entry => shippingStatuses.has(entry.status)).sort((a, b) => compare(a.id, b.id))
  function add(bytes: Buffer, extension: string, rgbaBytes: number) {
    const name = fileName(bytes, extension)
    if (!files.has(name)) {
      files.set(name, bytes)
      decodedBytes += rgbaBytes
    }
    return name
  }
  for (const entry of entries) {
    const rights = entry.provenance
    if (!rights.redistributable || /^(unknown|unlicensed|noassertion|none)$/i.test(rights.license.trim())) {
      throw new Error(`Missing redistribution rights for ${entry.id}`)
    }
    for (const part of entry.parts) {
      const key = `${entry.id}/${part.id}`
      const source = await load(part.source)
      if (part.source.endsWith('.svg')) {
        const [width, height] = iconSize(source, limit)
        placements.set(key, { url: add(source, 'svg', width * height * 4), rect: [0, 0, width, height], offset: [0, 0] })
        continue
      }
      if (!part.content) throw new Error(`Missing raster bounds for ${key}`)
      const pixels = PNG.sync.read(source)
      const group = groups.get(entry.category) ?? []
      group.push({ key, pixels, content: part.content })
      groups.set(entry.category, group)
    }
  }
  for (const [category, rasters] of [...groups].sort(([a], [b]) => compare(a, b))) {
    for (const page of pack(rasters, limit)) {
      const { width, height } = page
      const raw = rasterizePage(page)
      const bytes = await sharp(raw, { raw: { width, height, channels: 4 } }).webp({ lossless: true }).toBuffer()
      const url = add(bytes, 'webp', raw.length)
      pages.push({ category, url, width, height })
      for (const { key, x, y, content: [sx, sy, w, h] } of page.parts) {
        placements.set(key, { url, rect: [x, y, w, h], offset: [sx, sy] })
      }
    }
  }
  const manifest = {
    schemaVersion: registry.schemaVersion,
    atlas: { limit, extrusion, gutter: inset - extrusion, pages },
    assets: entries.map(entry => ({
      ...entry,
      parts: entry.parts.map(({ id, layer }) => ({ id, layer, ...placements.get(`${entry.id}/${id}`)! })),
    })),
    sizes: { compressedBytes: [...files.values()].reduce((sum, bytes) => sum + bytes.length, 0), decodedRgbaBytes: decodedBytes },
  }
  return { manifest, files }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const destination = process.argv[2] ?? fileURLToPath(new URL('../dist/assets/art/', import.meta.url))
  const registry = readAssetRegistry(sourceLimit)
  const { manifest, files } = await buildAssetPackage(registry, path => readFile(new URL(path, authoringSources)))
  await mkdir(destination, { recursive: true })
  for (const [name, bytes] of files) await writeFile(join(destination, name), bytes)
  const manifestTemp = join(destination, `manifest-${process.pid}.tmp`)
  await writeFile(manifestTemp, JSON.stringify(manifest, null, 2) + '\n')
  await rename(manifestTemp, join(destination, 'manifest.json'))
  console.log(`Packaged ${files.size} files: ${manifest.sizes.compressedBytes} compressed bytes, ${manifest.sizes.decodedRgbaBytes} decoded RGBA bytes`)
}
