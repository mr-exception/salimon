import { test, expect } from '@playwright/test'
import { PNG } from 'pngjs'
import { readdir, readFile } from 'node:fs/promises'
import { gzipSync } from 'node:zlib'
import { compareImages } from '../scripts/visual-comparison.ts'
import { ATLAS_LIMIT, BACKDROP, CLIP, MEGABYTE, VIEWPORT, type Renderer } from './renderer/model.ts'
import type { BenchmarkConfig, rendererBenchmark } from './renderer/main.tsx'

declare global { interface Window { rendererBenchmark: typeof rendererBenchmark } }
const url = 'http://127.0.0.1:4375/tests/renderer/index.html'

async function bundleSizes() {
  const root = new URL('../.renderer-fixture/assets/', import.meta.url)
  return Promise.all((await readdir(root)).sort().map(async file => {
    const data = await readFile(new URL(file, root))
    return { file, bytes: data.byteLength, gzipBytes: gzipSync(data).byteLength }
  }))
}

for (const scenario of ['scout', 'surface'] as const) {
  for (const density of ['current', '128'] as const) {
    test(`RENDER-939 ${scenario} ${density} frame timing, masks and interaction`, async ({ page, browser }, info) => {
      await page.goto(url)
      const region = page.getByRole('main')
      const results = []
      const screenshots = new Map<Renderer, Buffer>()
      const errors: string[] = []
      page.on('pageerror', error => errors.push(error.message))
      // Reverse order between densities and across each pair of trials to
      // distribute shader/font/cache warmup without overlapping other runs.
      const order = density === 'current' ? ['svg', 'pixi', 'pixi', 'svg'] as const : ['pixi', 'svg', 'svg', 'pixi'] as const
      for (const [index, renderer] of order.entries()) {
        const config: BenchmarkConfig = { renderer, scenario, density }
        const estimate = await page.evaluate(config => window.rendererBenchmark.mount(config), config)
        expect(estimate.dpr).toBe(1)
        expect(estimate.viewport).toEqual(VIEWPORT)
        expect(estimate.nodeCount).toBe(estimate.objectCount)
        // Playwright generates real pointer coordinates for logical scene selection.
        const [targetX, targetY] = scenario === 'scout' ? [536, 326] : [680, 420]
        await page.mouse.click(targetX!, targetY!)
        await expect(region).toHaveAttribute('data-pick', scenario === 'scout' ? 'bench' : 'sample-outcrop')
        await page.mouse.click(26, 65)
        await expect(region).toHaveAttribute('data-pick', 'edge-probe')
        await page.mouse.click(CLIP.x - 1, 65)
        await expect(region).toHaveAttribute('data-pick', 'none')
        const result = await page.evaluate(() => window.rendererBenchmark.measure())
        expect(result.intervals).toHaveLength(90)
        expect(result.intervals.every(ms => Number.isFinite(ms) && ms > 0)).toBe(true)
        if (renderer === 'pixi') {
          expect(result.drawCalls).toHaveLength(90)
          expect(result.drawCalls.every(draws => draws > 0)).toBe(true)
          expect(estimate.atlasPages.length).toBeGreaterThan(0)
        } else expect(result.drawsPerFrame).toBeNull()

        await page.evaluate(() => window.rendererBenchmark.renderAt(72))
        const original = await region.screenshot({ path: info.outputPath(renderer + '-' + (index + 1) + '.png') })
        const pixels = PNG.sync.read(original)
        const outside = (65 * pixels.width + CLIP.x - 2) * 4
        expect(pixels.data.readUIntBE(outside, 3).toString(16).padStart(6, '0')).toBe(BACKDROP.slice(1))
        // The repeat uses the same clock frame across a full adapter remount.
        const before = screenshots.get(renderer)
        if (before) expect(compareImages(before, original).pass).toBe(true)
        else screenshots.set(renderer, original)
        const bytes = estimate.rgbaWithMipsUpperBound + (estimate.estimatedFrameBufferBytes ?? 0)
        results.push({
          ...result,
          targets: {
            frameP95Within16_7ms: result.frameMs.p95 <= 16.7,
            estimatedMemoryWithin128MiB: bytes <= 128 * MEGABYTE,
            atlasWithin2048: estimate.atlasPages.every(page => Math.max(page.width, page.height) <= ATLAS_LIMIT),
            sourceDensity: density === '128' ? 128 : '32 terrain / 64 sprites',
          },
        })
        await page.evaluate(() => window.rendererBenchmark.dispose())
        await expect(region.locator('canvas,svg')).toHaveCount(0)
        // A disposed adapter no longer accepts pointer input.
        await page.mouse.click(26, 65)
        await expect(region).toHaveAttribute('data-pick', 'none')
      }
      const comparison = compareImages(screenshots.get('svg')!, screenshots.get('pixi')!)
      await info.attach('renderer-parity-diff.png', { body: comparison.diff, contentType: 'image/png' })
      const bundles = await bundleSizes()
      const gzipBudget = bundles.reduce((sum, file) => sum + file.gzipBytes, 0)
      await info.attach('renderer-measurements.json', {
        body: JSON.stringify({
          browser: browser.version(), scenario, density, results, bundles,
          compressedFixtureUpperBound: gzipBudget + Math.max(...results.map(x => x.estimate.compressedTextures)),
          under15MB: gzipBudget + Math.max(...results.map(x => x.estimate.compressedTextures)) <= 15_000_000,
          comparison: { ...comparison, diff: undefined },
        }),
        contentType: 'application/json',
      })
      expect(errors).toEqual([])
    })
  }
}

test('RENDER-939 context loss reports failure and remount restores controls', async ({ page }) => {
  await page.goto(url)
  const config = { renderer: 'pixi', scenario: 'scout', density: 'current' } as const
  await page.evaluate(config => window.rendererBenchmark.mount(config), config)
  expect(await page.evaluate(() => window.rendererBenchmark.loseContext())).toBe(true)
  await expect(page.getByRole('status')).toContainText('WebGL context lost')
  await expect(page.evaluate(() => window.rendererBenchmark.renderAt(1))).rejects.toThrow('WebGL context lost')
  await page.evaluate(config => window.rendererBenchmark.mount(config), config)
  await expect(page.getByRole('status')).toBeEmpty()
  await page.mouse.click(536, 326)
  await expect(page.getByRole('main')).toHaveAttribute('data-pick', 'bench')
})
