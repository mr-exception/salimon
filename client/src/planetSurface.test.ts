import { test } from 'node:test'
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { definitionDigest, PLANET_DEFINITIONS, planetDefinition, validatePlanetDefinition } from './planetDefinition.ts'
import { localToSurface, neighboringRegions, surfaceRegion, surfaceToLocal, type SurfaceLocation } from './surfaceCoordinates.ts'
import { authoredObstacles, samplePlanetSurface } from './planetSurface.ts'
import { normal, scale, dot, sub } from './math.ts'
import { startingSurface, resolveSurface, siteEntities, validateSurfaceClearance } from './surfaceSite.ts'
import { allowedMove } from './topDownInteraction.ts'
import { point2 } from './topDownFrames.ts'

const location = (bodyId: string, direction: readonly [number, number, number]): SurfaceLocation => ({ bodyId, radialM: scale(normal(direction), planetDefinition(bodyId).radiusM + 5) })
const near = (a: readonly number[], b: readonly number[], tolerance = 1e-7) => assert.ok(Math.hypot(...a.map((n, i) => n - b[i]!)) < tolerance, `${a} != ${b}`)

test('AC-1/4/5 immutable packaged records and domain digests match published golden vectors', () => {
  assert.deepEqual(PLANET_DEFINITIONS.map(p => p.digest), ['f3d9f26ee5ae532a', '8a9aa7c371d8ea74', '45aac4d9d0e82a25'])
  for (const definition of PLANET_DEFINITIONS) {
    assert.equal(validatePlanetDefinition(structuredClone(definition)), definition)
    assert.ok(Object.isFrozen(definition.domains.terrain.parameters))
    const { digest, ...content } = definition
    assert.equal(definitionDigest(content), digest)
    for (const domain of Object.values(definition.domains)) {
      const { digest, ...content } = domain
      assert.equal(definitionDigest(content), digest)
      assert.match(domain.seed, /^[0-9a-f]{16}$/)
    }
    assert.equal(new Set(Object.values(definition.domains).map(d => d.seed)).size, 4)
  }
  assert.equal(definitionDigest({ z: [1, 2], a: 'µ' }), definitionDigest({ a: 'µ', z: [1, 2] }))
  assert.throws(() => definitionDigest({ value: NaN }))
  const original = planetDefinition('vesper'), visual = structuredClone(original.domains.visual)
  const terrainDigest = original.domains.terrain.digest
  const cosmetic = { ...visual, parameters: { palette: 'changed' } }
  assert.notEqual(definitionDigest(cosmetic), definitionDigest(visual))
  assert.equal(planetDefinition('vesper').domains.terrain.digest, terrainDigest)
  for (const mutate of [
    (v: any) => { v.version++ }, (v: any) => { v.digest = 'wrong' },
    (v: any) => { v.domains.terrain.parameters.heightM = 1 },
    (v: any) => { v.domains.geology.seed = '000000000000ffff' },
    (v: any) => { v.domains.resources.digest = 'wrong' },
    (v: any) => { v.extra = 1 }, (v: any) => { v.radiusM++ },
  ]) {
    const value = structuredClone(original); mutate(value)
    assert.throws(() => validatePlanetDefinition(value), /mismatch|migration/)
  }
})

test('AC-1 identical canonical answers across fresh processes, reversed queries, copies and yaw', () => {
  const vectors = ['vesper', 'vesper-moon', 'caligo'].flatMap(b => [[1, 0, 0], [0, -1, 0], [-1, 0, -0.000001], [1, 1, 1]].map(v => location(b, v as [number, number, number])))
  const expected = vectors.map(v => samplePlanetSurface(v))
  assert.deepEqual([...vectors].reverse().map(v => samplePlanetSurface(structuredClone(v))).reverse(), expected)
  assert.equal(new Set(expected.map(s => s.regionId)).size, vectors.length)
  const script = `import {samplePlanetSurface} from './src/planetSurface.ts'; console.log(JSON.stringify(${JSON.stringify(vectors)}.map(v=>samplePlanetSurface(v))))`
  const fresh = spawnSync(process.execPath, ['--experimental-strip-types', '--input-type=module', '-e', script], { cwd: new URL('..', import.meta.url), encoding: 'utf8' })
  assert.equal(fresh.status, 0, fresh.stderr); assert.deepEqual(JSON.parse(fresh.stdout), expected)
  for (const v of vectors) assert.equal(resolveSurface({ ...v, yaw: .9 }).id, resolveSurface({ ...v, yaw: -.5 }).id)
})

test('AC-2 reversible tangent charts across poles, antimeridian, all face seams/corners and large bodies', () => {
  for (const definition of PLANET_DEFINITIONS) {
    for (const direction of [
      [0, 1, 0], [0, -1, 0], [1e-12, 1, -1e-12], [1e-12, -1, 1e-12],
      [-1, 0, 1e-12], [-1, 0, -1e-12],
      ...[-1, 1].flatMap(x => [-1, 1].flatMap(y => [[x, y, 0], [x, 0, y], [0, x, y], [x, y, 1], [x, y, -1]])),
    ] as [number, number, number][]) {
      const v = location(definition.bodyId, direction), region = surfaceRegion(v)
      near(localToSurface(region, surfaceToLocal(v, region)).radialM, v.radialM)
      for (const local of [[0, 0], [.0001, -.0002], [31.9999, -32.0001], [128, -128]] as const) {
        const physical = localToSurface(region, local)
        near(surfaceToLocal(physical, region), local, 1e-8)
        const next = surfaceRegion(physical)
        near(localToSurface(next, surfaceToLocal(physical, next)).radialM, physical.radialM)
        for (const basis of [region.east, region.north, region.up]) near([dot(basis, basis)], [1], 1e-12)
        near([dot(region.east, region.north), dot(region.east, region.up), dot(region.north, region.up)], [0, 0, 0], 1e-12)
      }
      const neighbors = neighboringRegions(region)
      assert.ok(neighbors.length > 0 && neighbors.length <= 5)
      assert.equal(new Set(neighbors.map(n => n.id)).size, neighbors.length)
      assert.deepEqual(neighbors.map(n => n.id), neighboringRegions(structuredClone(region)).map(n => n.id))
      for (const neighbor of neighbors) {
        assert.notEqual(neighbor.id, region.id)
        assert.ok(neighboringRegions(neighbor).some(back => back.id === region.id), 'seam and corner adjacency reciprocal')
        const physical = localToSurface(neighbor, [0.125, -.25])
        near(localToSurface(region, surfaceToLocal(physical, region)).radialM, physical.radialM)
      }
    }
  }
  const a = surfaceRegion(location('caligo', [-1, 0, 1e-12])), b = surfaceRegion(location('caligo', [-1, 0, -1e-12]))
  assert.equal(a.id, b.id, 'no antimeridian duplicate world')
  const r = surfaceRegion(location('caligo', [1, 0, 0]))
  for (const n of neighboringRegions(r)) assert.ok(neighboringRegions(n).some(back => back.id === r.id), 'interior adjacency reciprocal')
  const edge = surfaceRegion(location('caligo', [1, 1, 0]))
  assert.ok(neighboringRegions(edge).some(n => n.cell.axis !== edge.cell.axis), 'probe crosses cube face')
})

test('AC-2/4 malformed coordinates, unsupported bodies, chart back-side and forged frames fail explicitly', () => {
  const region = surfaceRegion(startingSurface())
  for (const radialM of [[0, 0, 0], [NaN, 1, 0], [Infinity, 0, 1], [1, 2], ['1', 2, 3]]) assert.throws(() => surfaceRegion({ bodyId: 'vesper', radialM } as any))
  for (const bodyId of ['nacre', 'relay', 'unknown']) assert.throws(() => samplePlanetSurface({ bodyId, radialM: [1, 0, 0] }), /Unsupported/)
  assert.throws(() => samplePlanetSurface(startingSurface(), { bodyId: 'vesper', version: 2, digest: 'wrong' }), /mismatch/)
  assert.throws(() => surfaceToLocal(location('vesper', [0, -1, 0]), region), /outside/)
  assert.throws(() => surfaceToLocal(location('caligo', [0, 1, 0]), region), /mismatch/)
  for (const local of [[Infinity, 0], [NaN, 0], [1], [1e9, 1e9]]) assert.throws(() => localToSurface(region, local as any))
  assert.throws(() => localToSurface({ ...region, east: [0, 0, 0] }, [0, 0]), /frame/)
  assert.throws(() => neighboringRegions({ ...region, cell: { ...region.cell, i: 9 } }), /frame/)
})

test('AC-3/4 shared physical query preserves original spawn, Scout, source anchors and walking/landing decisions', () => {
  const site = resolveSurface(startingSurface()), entities = siteEntities(site), region = surfaceRegion(site.address)
  assert.equal(site.id, AUTHORED_SURFACE.siteId)
  near(surfaceToLocal(startingSurface(), region), AUTHORED_SURFACE.ship.slice(0, 2))
  assert.deepEqual(AUTHORED_SURFACE.spawn, [-6, 3])
  assert.equal(samplePlanetSurface(startingSurface(), undefined, { landingYaw: site.address.yaw }).landingClear, true)
  assert.doesNotThrow(() => validateSurfaceClearance(site))
  for (const source of authoredObstacles().filter(o => o.id.startsWith('source:'))) {
    const atSource = localToSurface(region, [source.x, source.y]), sample = samplePlanetSurface(atSource, undefined, { landingYaw: 0 })
    assert.equal(sample.collision, true); assert.equal(sample.landingClear, false); assert.deepEqual(sample.sourceIds, [source.id])
    const address = { ...atSource, yaw: 0 }
    assert.throws(() => validateSurfaceClearance(resolveSurface(address)), /Unsafe/)
    const from = [source.x, source.y - source.halfHeight - .6] as const, motion = [0, 4] as const
    const probe = samplePlanetSurface(localToSurface(region, from), undefined, { radiusM: .46, motion })
    assert.equal(probe.heightM, 0); assert.equal(probe.slopeRadians, 0); assert.equal(probe.solid, true)
    near([probe.moveFraction], [allowedMove({ frame: 'planet:fixture', point: point2(...from) }, motion, entities, site.frames)], 1e-8)
  }
  const anchors = Object.fromEntries(authoredObstacles().map(o => [o.id, [o.x, o.y] as const]))
  anchors['rock:west'] = [2, -1]
  const delta = { definition: planetDefinition('vesper'), anchors }
  assert.equal(samplePlanetSurface(startingSurface(), undefined, { landingYaw: -.12 }, delta).landingClear, false)
  assert.throws(() => authoredObstacles({ ...delta, definition: { ...delta.definition, version: 99 } }), /mismatch/)
  assert.throws(() => authoredObstacles({ ...delta, anchors: { ...anchors, unknown: [0, 0] } }), /Invalid/)
  assert.throws(() => authoredObstacles({ ...delta, anchors: {} }), /source/)
  assert.throws(() => authoredObstacles({ ...delta, anchors: { ...anchors, 'rock:west': [NaN, 0] } }), /Invalid/)
  assert.throws(() => samplePlanetSurface(startingSurface(), undefined, { radiusM: -1 }), /Invalid/)
  assert.throws(() => samplePlanetSurface(startingSurface(), undefined, { landingYaw: NaN }), /Invalid/)
  assert.throws(() => samplePlanetSurface(startingSurface(), undefined, { motion: [0, 0, 9] } as any), /Invalid/)
  assert.throws(() => resolveSurface(startingSurface(), [{ ...site.body, radiusKm: site.body.radiusKm + 1 }]), /coordinates|mismatch/)
  assert.throws(() => samplePlanetSurface(location('caligo', [1, 0, 0]), undefined, {}, delta), /mismatch/)
  for (const bodyId of ['vesper-moon', 'caligo']) {
    const sample = samplePlanetSurface(location(bodyId, [0, 1, 0]), undefined, { landingYaw: 1 })
    assert.equal(sample.collision, false); assert.equal(sample.landingClear, true); assert.deepEqual(sample.sourceIds, [])
  }
  assert.ok(Math.hypot(...sub(location('vesper', [1, 0, 0]).radialM, startingSurface().radialM)) > 1000)
})
