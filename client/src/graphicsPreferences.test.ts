import assert from 'node:assert/strict'
import test from 'node:test'
import { GRAPHICS_KEY, QUALITY, readGraphics, renderDpr, storeGraphics } from './rendering/graphicsPreferences.ts'

test('graphics presets and custom values persist independently of gameplay', () => {
  const saved = new Map([['expedition', 'original']])
  const storage = { getItem: (key: string) => saved.get(key) ?? null, setItem: (key: string, value: string) => { saved.set(key, value) } }
  assert.deepEqual(readGraphics(storage), { preset: 'Medium', custom: QUALITY.Medium })
  const preference = { preset: 'Custom', custom: { ...QUALITY.High, dprCap: 1.5, scale: 80 } } as const
  storeGraphics(storage, preference)
  assert.deepEqual(readGraphics(storage), preference)
  assert.equal(saved.get('expedition'), 'original')
  assert.equal(renderDpr(3, preference.custom), 1.2)
  assert.equal(renderDpr(2, QUALITY.Low), .75)
  assert.equal(renderDpr(1, QUALITY.Medium), 1)
  assert.ok(saved.has(GRAPHICS_KEY))
})

test('graphics values clamp malformed storage and recover inaccessible preferences', () => {
  const corrupt = { getItem: () => '{', setItem: () => { throw new Error('Blocked') } }
  assert.equal(readGraphics(corrupt).preset, 'Medium')
  assert.doesNotThrow(() => storeGraphics(corrupt, readGraphics(corrupt)))
  const invalid = { getItem: () => JSON.stringify({ preset: 'Unknown', custom: {
    scale: -5, dprCap: 600, particles: '100', environment: -10, lighting: 'Wrong', shadows: null,
  } }), setItem: () => {} }
  assert.deepEqual(readGraphics(invalid), { preset: 'Medium', custom: {
    scale: 50, dprCap: 2, particles: 60, environment: 0, lighting: 'standard', shadows: 'contact',
  } })
})
