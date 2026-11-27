import assert from 'node:assert/strict'
import test from 'node:test'
import { createTopDownState, drawList, LAYERS, PLAYER_ID } from './topDownRuntime.ts'
import { svgArtIntersects, svgLayers, svgSpriteTransform } from './rendering/svgWorld.ts'

test('retained SVG stages preserve domain layer order, with terrain art behind generated ground', () => {
  const world = createTopDownState('exterior', true)
  const items = drawList(world)
  const passes = svgLayers(items)
  assert.deepEqual(passes.map(pass => pass.id), LAYERS)
  assert.deepEqual(passes.flatMap(pass => [...pass.beforeTerrain, ...pass.sprites]), items)
  const terrain = passes[0]
  assert.ok(terrain.beforeTerrain.some(item => item.entity.appearance === 'ground'))
  assert.ok(terrain.sprites.every(item => item.entity.appearance !== 'ground' && item.entity.appearance !== 'surface'))
  assert.ok(passes.find(pass => pass.id === 'people')?.sprites.some(item => item.entity.id === PLAYER_ID))
  const seated = svgLayers(drawList({ ...world, player: { ...world.player, mode: 'seated' } }))
  assert.equal(seated.find(pass => pass.id === 'cues')?.sprites.filter(item => item.entity.id === PLAYER_ID).length, 1)
})

test('sprite and selection share signed camera-space translation, rotation and scale', () => {
  const item = drawList(createTopDownState())[0]
  assert.equal(svgSpriteTransform({ ...item, positionM: [-1.25, 2], angle: Math.PI / 2 }, 32),
    'translate(-40 64) rotate(90) scale(32)')
  assert.equal(svgSpriteTransform({ ...item, positionM: [0, 0], angle: -Math.PI / 2 }, 64),
    'translate(0 0) rotate(-90) scale(64)')
})

test('art culling retains rotated overhang and the exact viewport boundary', () => {
  const item = drawList(createTopDownState())[0]
  const bounds = [-1, -4, 2, 8] as const
  assert.equal(svgArtIntersects({ ...item, positionM: [14, 0], angle: Math.PI / 2 }, bounds, 20, 20), true)
  assert.equal(svgArtIntersects({ ...item, positionM: [14.01, 0], angle: Math.PI / 2 }, bounds, 20, 20), false)
  assert.equal(svgArtIntersects({ ...item, positionM: [15, 0], angle: Math.PI / 2 }, bounds, 20, 20, 2), true)
})
