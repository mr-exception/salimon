import assert from 'node:assert/strict'
import test from 'node:test'
import { PNG } from 'pngjs'
import { readAssetRegistry, alphaBounds } from './asset-sources.ts'
import { resolveAsset, validateAssetRegistry } from '../src/rendering/assetRegistry.ts'

const lookup = {
  source: (path: string) => path === 'scout/chair.png' ? { alphaBounds: [2, 2, 45, 45] as const } : undefined,
  footprint: (ref: string) => ref === 'scout:chair' ? [.7, .7] as const : undefined,
  anchor: (ref: string) => ref === 'chair',
}

test('validates actual source alpha and domain dimensions for the checked-in registry', () => {
  const registry = readAssetRegistry()
  assert.equal(registry.schemaVersion, 1)
  assert.ok(registry.assets.some(entry => entry.id === 'scout.chair' && entry.status === 'placeholder'))
  const pixels = new PNG({ width: 4, height: 5 })
  assert.equal(alphaBounds(pixels), undefined)
  pixels.data[(2 * 4 + 1) * 4 + 3] = 1
  pixels.data[(3 * 4 + 3) * 4 + 3] = 255
  assert.deepEqual(alphaBounds(pixels), [1, 2, 3, 2])
})

test('rejects invalid schema, duplicate IDs, missing references, padding and conflicting dimensions', () => {
  const { assets: [asset] } = readAssetRegistry()
  const check = (entry: unknown, expected: RegExp) =>
    assert.throws(() => validateAssetRegistry({ schemaVersion: 1, assets: [entry] }, lookup), expected)
  assert.throws(() => validateAssetRegistry({ schemaVersion: 2, assets: [] }, lookup), /version/)
  assert.throws(() => validateAssetRegistry({ schemaVersion: 1, assets: [asset, asset] }, lookup), /duplicate asset/)
  check({ ...asset, revision: 0 }, /revision/)
  check({ ...asset, pivot: [0, 1.01] }, /pivot/)
  check({ ...asset, parts: [asset.parts[0], asset.parts[0]] }, /duplicate part/)
  check({ ...asset, parts: [{ ...asset.parts[0], source: '../chair.png' }] }, /source path/)
  check({ ...asset, parts: [{ ...asset.parts[0], source: 'missing.png' }] }, /missing source/)
  check({ ...asset, parts: [{ ...asset.parts[0], content: [0, 0, 49, 49] }] }, /alpha bounds/)
  check({ ...asset, density: 128 }, /unpadded pixels/)
  check({ ...asset, footprintRef: 'scout:missing' }, /domain footprint/)
  check({ ...asset, physicalSize: [1, 1] }, /domain footprint/)
  check({ ...asset, anchors: { seat: 'missing' } }, /unknown anchor/)
  check({ ...asset, states: { idle: ['missing'] } }, /invalid state parts/)
  check({ ...asset, status: 'production', provenance: { ...asset.provenance, redistributable: false } }, /redistribution/)
})

test('missing decorative assets fall back and essential assets can retry the same stable ID', async () => {
  const [entry] = readAssetRegistry().assets
  const failure = new Error('missing art')
  const load = async () => { throw failure }
  assert.deepEqual(await resolveAsset({ ...entry, essential: false }, load), { kind: 'placeholder', id: entry.id })
  assert.deepEqual(await resolveAsset(entry, load), { kind: 'retry', id: entry.id, cause: failure })
  assert.deepEqual(await resolveAsset(entry, async () => 'loaded'), { kind: 'available', value: 'loaded' })
})
