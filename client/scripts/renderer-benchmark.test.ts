import assert from 'node:assert/strict'
import test from 'node:test'
import { ATLAS_LIMIT, CLIP, distribution, hit, packAtlases, pose, scene, texelsPerMetre } from '../tests/renderer/model.ts'

test('both scenes generate deterministic geometry, distinct layers and bounded shared input', () => {
  assert.deepEqual(scene('surface'), scene('surface'))
  assert.notDeepEqual(scene('surface'), scene('surface', 123))
  for (const scenario of ['scout', 'surface'] as const) {
    const objects = scene(scenario)
    assert.equal(new Set(objects.map(object => object.id)).size, objects.length)
    assert.equal(objects.filter(object => object.motion === 'stride').length, 6)
    assert.equal(objects.filter(object => object.motion === 'pulse').length, 4)
    const particle = objects.find(object => object.motion === 'drift')!
    assert.notDeepEqual(pose(particle, 30), pose(particle, 0))
    assert.ok(pose(particle, 30).alpha > 0)
    assert.equal(hit(objects, 26, 65), 'edge-probe')
    assert.equal(hit(objects, CLIP.x - 1, 65), null)
  }
  assert.ok(scene('surface').length > 400)
  assert.equal(texelsPerMetre('terrain', 'current'), 32)
  assert.equal(texelsPerMetre('deck', 'current'), 64)
  assert.equal(texelsPerMetre('terrain', '128'), 128)
})

test('atlas packing keeps deterministic, disjoint padded textures and rejects invalid sizes', () => {
  const large = [
    { key: 'a', width: ATLAS_LIMIT - 4, height: ATLAS_LIMIT - 4 },
    { key: 'b', width: 900, height: 950 }, { key: 'c', width: 900, height: 950 },
  ]
  const a = packAtlases(large), b = packAtlases([...large].reverse())
  assert.deepEqual(a, b)
  assert.equal(a.pages.length, 2)
  assert.equal(a.entries.get('a')?.page, 0)
  assert.ok(a.entries.get('b')!.x + a.entries.get('b')!.width + 4 <= a.entries.get('c')!.x)
  assert.ok(a.pages.every(({ width, height }) => width <= ATLAS_LIMIT && height <= ATLAS_LIMIT))
  assert.throws(() => packAtlases([{ key: 'bad', width: ATLAS_LIMIT - 3, height: 50 }]), /Invalid atlas bitmap/)
  assert.throws(() => packAtlases([{ key: 'bad', width: 0, height: 50 }]), /Invalid atlas bitmap/)
  assert.throws(() => packAtlases([large[0]!, large[0]!]), /Invalid atlas bitmap/)
})

test('frame distributions preserve samples, outliers and empty input', () => {
  const samples = [30, 16, 12, 18, 15]
  assert.deepEqual(distribution(samples), { p50: 16, p95: 30, p99: 30, max: 30 })
  assert.deepEqual(samples, [30, 16, 12, 18, 15])
  assert.deepEqual(distribution([]), { p50: 0, p95: 0, p99: 0, max: 0 })
})
