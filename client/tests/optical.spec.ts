import { test, expect, type Page } from '@playwright/test'
import { PNG } from 'pngjs'
const json = (value: unknown) => JSON.stringify(value, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2)
type Snapshot = { contacts: number; selected: string | null; spheres: unknown; calls: number; triangles: number; renderer: string; camera: number[] }
const capture = (page: Page) => page.locator('canvas').screenshot({ mask: [page.getByRole('button', { name: 'Pulse' })] })
const snapshot = (page: Page) => page.evaluate(() => (window as unknown as { opticalSnapshot(): Snapshot }).opticalSnapshot())
async function boot(page: Page, scenario: string) {
 await page.goto(`/tests/optical.html?scenario=${scenario}`)
 await expect.poll(async () => (await snapshot(page))?.calls).toBe(1)
 await page.waitForTimeout(150)
}
const brightness = (png: PNG, x: number, y: number) => { const i = (y * png.width + x) * 4; return png.data[i] + png.data[i + 1] + png.data[i + 2] }
test('OPT-1 optical disks before scanning, repeat, occultation and far origin invariance', async ({ page }, info) => {
 test.setTimeout(240000) // Four full-resolution software-WebGL scene boots.
 const errors: string[] = []; page.on('pageerror', e => errors.push(e.message)); page.on('console', m => { if(m.type() === 'error') errors.push(m.text()) })
 await boot(page, 'wide')
 const initial = await snapshot(page)
 expect(initial.contacts).toBe(0); expect(initial.selected).toBeNull()
 expect(initial.renderer).toContain('SwiftShader'); expect(initial.triangles).toBe(2)
 expect(initial.camera).toEqual([0, 0, 0])
 expect(await page.locator('body').innerText()).not.toContain('Secret')
 const wide = await capture(page), png = PNG.sync.read(wide)
 expect(brightness(png, 640, 400)).toBeGreaterThan(10)
 expect(brightness(png, 690, 400)).toBeGreaterThan(10)
 expect(brightness(png, 696, 400)).toBeLessThan(10)
 await page.locator('canvas').click({ position: { x: 640, y: 400 } })
 expect((await snapshot(page)).selected).toBeNull()
 await page.getByRole('button', { name: 'Pulse' }).click()
 await expect.poll(async () => (await snapshot(page)).contacts).toBe(4)
 expect((await capture(page)).equals(wide)).toBe(true)
 await info.attach('wide.png', { body: wide, contentType: 'image/png' })
 for (const scenario of ['clear', 'translated', 'wide']) {
  await boot(page, scenario)
  expect((await capture(page)).equals(wide)).toBe(true)
 }
 expect((await snapshot(page)).spheres).toEqual(initial.spheres)
 expect(errors).toEqual([])
 await info.attach('measurements.json', { body: json({ initial, errors, repeat: 'byte-identical', occultedRemoval: 'byte-identical', translated: 'byte-identical' }), contentType: 'application/json' })
})
test('OPT-2 telescope tiny moon and grounded horizon with below-surface occlusion', async ({ page }, info) => {
 const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
 await boot(page, 'telescope')
 const telescope = await capture(page), png = PNG.sync.read(telescope)
 expect(brightness(png, 640, 400)).toBeGreaterThan(10)
 expect(brightness(png, 820, 400)).toBeGreaterThan(10)
 expect(brightness(png, 830, 400)).toBeLessThan(10)
 expect(brightness(png, 915, 400)).toBeGreaterThan(10)
 expect(brightness(png, 930, 400)).toBeLessThan(10)
 await info.attach('telescope.png', { body: telescope, contentType: 'image/png' })
 const state = await snapshot(page)
 await boot(page, 'surface')
 const surface = await capture(page), ground = PNG.sync.read(surface)
 expect(brightness(ground, 640, 200)).toBeLessThan(10)
 expect(brightness(ground, 640, 600)).toBeGreaterThan(10)
 await info.attach('surface.png', { body: surface, contentType: 'image/png' })
 await boot(page, 'below')
 const below = await capture(page), down = PNG.sync.read(below)
 expect(brightness(down, 640, 200)).toBeGreaterThan(10)
 expect(brightness(down, 640, 600)).toBeGreaterThan(10)
 await info.attach('below.png', { body: below, contentType: 'image/png' })
 expect(below.equals(telescope)).toBe(false)
 expect(errors).toEqual([])
 await info.attach('measurements.json', { body: json({ telescope: state, below: await snapshot(page), errors }), contentType: 'application/json' })
})

test('OPT-3 near/far physical depths and huge-origin telescope preserve optical pixels', async ({ page }, info) => {
 test.setTimeout(240000) // Five full-resolution software-WebGL scene boots.
 const measurements = []
 await boot(page, 'wide')
 const wide = await capture(page)
 for (const scenario of ['near-depth', 'far-depth']) {
  await boot(page, scenario)
  const actual = await capture(page), state = await snapshot(page)
  expect(actual.equals(wide), `DEPTH_CLIPPING ${scenario}`).toBe(true)
  expect(state.camera).toEqual([0, 0, 0]); expect(state.calls).toBe(1)
  await info.attach(`${scenario}.png`, { body: actual, contentType: 'image/png' })
  measurements.push(state)
 }
 await boot(page, 'telescope'); const telescope = await capture(page)
 await boot(page, 'telescope-translated')
 expect((await capture(page)).equals(telescope), 'TELESCOPE_SPLIT_PRECISION').toBe(true)
 measurements.push(await snapshot(page))
 await info.attach('depth-precision.json', { body: json({ measurements, nearScale: 1e-6, farScale: 1e16, depthRangeM: [0.01, 1e24] }), contentType: 'application/json' })
})
