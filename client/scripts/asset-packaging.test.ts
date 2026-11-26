import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { readFile } from 'node:fs/promises'
import test from 'node:test'
import sharp from 'sharp'
import { authoringSources, readAssetRegistry } from './asset-sources.ts'
import { buildAssetPackage } from './asset-packaging.ts'

const load = (path: string) => readFile(new URL(path, authoringSources))

test('repeat and reversed-order packaging preserve bytes and straight-alpha extruded edges', async () => {
  const registry = readAssetRegistry()
  const a = await buildAssetPackage(registry, load)
  const b = await buildAssetPackage({ ...registry, assets: [...registry.assets].reverse() }, load)
  assert.deepEqual(a, b)
  assert.equal(a.manifest.atlas.limit, 2048)
  const chair = a.manifest.assets.find(asset => asset.id === 'scout.chair')!.parts[0]
  const [x, y, w, h] = chair.rect
  const { data, info } = await sharp(a.files.get(chair.url)!).raw().toBuffer({ resolveWithObject: true })
  assert.equal(info.channels, 4)
  const pixel = (col: number, row: number) => [...data.subarray((row * info.width + col) * 4, (row * info.width + col + 1) * 4)]
  assert.deepEqual(pixel(x - 1, y + 22), pixel(x, y + 22))
  assert.deepEqual(pixel(x + w, y + 22), pixel(x + w - 1, y + 22))
  assert.deepEqual(pixel(x - 2, y + 22), [0, 0, 0, 0])
  assert.deepEqual(chair.offset, [2, 2])
  assert.deepEqual([w, h], [45, 45])
  for (const [name, bytes] of a.files) assert.equal(name.split('.')[0], createHash('sha256').update(bytes).digest('hex'))
  assert.equal(a.manifest.sizes.compressedBytes, [...a.files.values()].reduce((sum, file) => sum + file.length, 0))
  assert.equal(a.manifest.sizes.decodedRgbaBytes, 4 * (info.width * info.height + 256 * 256))
})

test('packs an exact-fit rectangle and spills equal-size assets onto separate bounded pages', async () => {
  const registry = readAssetRegistry()
  const first = registry.assets[0]
  const result = await buildAssetPackage({ ...registry, assets: [first, { ...first, id: 'scout.chair.second' }] }, load, 49)
  assert.deepEqual(result.manifest.atlas.pages.map(({ width, height }) => [width, height]), [[49, 49], [49, 49]])
  assert.equal(result.files.size, 1)
  await assert.rejects(buildAssetPackage({ ...registry, assets: [first] }, load, 48), /exceeds/)
  await assert.rejects(buildAssetPackage(registry, load, 2049), /limit/)
})

test('rejects bad rights, missing files, corrupt raster data and unsafe or oversize SVG', async () => {
  const registry = readAssetRegistry()
  const [chair, icon] = registry.assets
  await assert.rejects(buildAssetPackage({ ...registry, assets: [{ ...chair, provenance: { ...chair.provenance, redistributable: false } }] }, load), /rights/)
  await assert.rejects(buildAssetPackage(registry, path => load('missing-' + path)), /ENOENT/)
  await assert.rejects(buildAssetPackage(registry, async () => Buffer.from('invalid PNG')))
  const svg = (file: string) => buildAssetPackage({ ...registry, assets: [icon] }, async () => Buffer.from(file))
  await assert.rejects(svg('<svg'), /root element|unclosed|unexpected/i)
  await assert.rejects(svg('<svg xmlns="http://www.w3.org/2000/svg" width="2049" height="1"/>'), /dimensions/)
  await assert.rejects(svg('<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><image href="https://example.com/icon.svg"/></svg>'), /unsupported/)
})
