import { test, expect, type Page } from '@playwright/test'
import { PNG } from 'pngjs'
import { opticalRayHit, type OpticalSphere } from '../src/opticalSky.ts'
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
 // A close view of one polar face must have broad flat regions, not rounding
 // speckles. This row crosses the two adjacent octants only once.
 let runs = 1
 for (let x = 101; x < 1180; x++) {
  const i = (200 * down.width + x) * 4
  if (!down.data.subarray(i, i + 3).equals(down.data.subarray(i - 4, i - 1))) runs++
 }
 await info.attach('below.png', { body: below, contentType: 'image/png' })
 expect(runs, 'POLAR_FACET_SPECKLING').toBeLessThanOrEqual(4)
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

test('ART-906 faceted catalog materials, physical near/far LOD, repeat and scanner isolation', async ({ page }, info) => {
 test.setTimeout(240000)
 const errors: string[] = []
 page.on('pageerror', e => errors.push(e.message))
 page.on('console', m => { if (m.type() === 'error') errors.push(m.text()) })
 const measurements = []
 let near: Buffer | undefined
 for (const distance of [6000, 65000, 6500000]) {
  await boot(page, `art&distance=${distance}`)
  const state = await snapshot(page), actual = await capture(page), png = PNG.sync.read(actual)
  expect(state.calls).toBe(1); expect(state.triangles).toBe(2)
  expect(state.contacts).toBe(0); expect(state.selected).toBeNull()
  expect(state.renderer).toContain('SwiftShader')
  const centers = [-.54, -.18, .18, .54].map(x => Math.round(640 + x / Math.tan(75 * Math.PI / 360) * 400))
  const colors = centers.map(x => [...png.data.subarray((400 * png.width + x) * 4, (400 * png.width + x) * 4 + 3)])
  if (distance === 6000) {
   near = actual
   expect(new Set(colors.map(c => c.join(','))).size).toBe(4)
   // Flat triangle interiors produce repeated colors, not per-pixel noise.
   const histogram = new Map<string, number>()
   for (let y = 380; y < 420; y++) for (let x = centers[0] - 20; x < centers[0] + 20; x++) {
    const i = (y * png.width + x) * 4, key = png.data.subarray(i, i + 3).toString('hex')
    histogram.set(key, (histogram.get(key) ?? 0) + 1)
   }
   expect(histogram.size).toBeGreaterThan(3); expect(histogram.size).toBeLessThan(60)
   expect(Math.max(...histogram.values())).toBeGreaterThan(100)
   await page.getByRole('button', { name: 'Pulse' }).click()
   expect((await snapshot(page)).contacts).toBe(4)
   expect((await capture(page)).equals(actual)).toBe(true)
  }
  // Beyond the physical radius, only the restrained background remains. At
  // unresolved size the body cannot become a fixed-size solid contact disk.
  // Off-axis disks project to conics: their right tangent is farther than
  // center + angular radius. Solve the physical ray/sphere tangent directly.
  const r = 650 / distance
  for (const t of [-.54, -.18, .18, .54]) {
   const tangent = (t + r * Math.sqrt(1 + t * t - r * r)) / (1 - r * r)
   const x = Math.ceil(640 + tangent / Math.tan(75 * Math.PI / 360) * 400) + 3
   expect(brightness(png, x, 400)).toBeLessThan(70)
  }
  expect((await capture(page)).equals(actual), `repeat at ${distance}m`).toBe(true)
  await info.attach(`materials-${distance}m.png`, { body: actual, contentType: 'image/png' })
  measurements.push({ distance, colors, state })
 }
 await boot(page, 'art-translated&distance=6000')
 expect((await capture(page)).equals(near!)).toBe(true)
 expect(errors).toEqual([])
 await info.attach('art-measurements.json', { body: json({ measurements, errors, repeat: 'byte-identical', translated: 'byte-identical' }), contentType: 'application/json' })
})

test('ART-906 canonical polar ice differs from the same legacy moon without moving its limb', async ({ page }, info) => {
 const measurements = []
 await boot(page, 'ice-v1')
 const legacy = await capture(page), a = PNG.sync.read(legacy)
 measurements.push(await snapshot(page))
 await boot(page, 'ice-v2')
 const icy = await capture(page), b = PNG.sync.read(icy)
 measurements.push(await snapshot(page))
 expect(icy.equals(legacy)).toBe(false)
 expect(brightness(b, 640, 360)).toBeGreaterThan(brightness(a, 640, 360))
 // Equatorial material, physical limb and background retain their pixels.
 for (const [x, y] of [[640, 400], [695, 400], [640, 340]]) {
  const i = (y * a.width + x) * 4
  expect(b.data.subarray(i, i + 4).equals(a.data.subarray(i, i + 4))).toBe(true)
 }
 expect((await capture(page)).equals(icy)).toBe(true)
 await info.attach('moon-legacy.png', { body: legacy, contentType: 'image/png' })
 await info.attach('moon-polar-ice.png', { body: icy, contentType: 'image/png' })
 await info.attach('ice-publication.json', { body: json(measurements), contentType: 'application/json' })
})

test('ART-906 unresolved point cutoff stays continuous outside the physical disk', async ({ page }, info) => {
 test.setTimeout(180000)
 const pixelAngle = 2 * Math.tan(75 * Math.PI / 360) / 800
 const thresholdDistance = 650 / (.8 * pixelAngle * Math.hypot(1, .54))
 const measurements = [], images: Buffer[] = []
 // This ray misses the star's physical disk while sampling its optical PSF.
 const x = 922, y = 400, ray = [(x + .5 - 640) * pixelAngle, -(y + .5 - 400) * pixelAngle, -1] as const
 for (const factor of [1 - 1e-5, 1 + 1e-5]) {
  const distance = thresholdDistance * factor
  await boot(page, `art&distance=${distance}`)
  const state = await snapshot(page), sphere = (state.spheres as OpticalSphere[])[3]
  expect(opticalRayHit(sphere, ray)).toBeNull()
  expect(state.calls).toBe(1); expect(state.triangles).toBe(2)
  const actual = await capture(page), png = PNG.sync.read(actual), i = (y * png.width + x) * 4
  measurements.push({ distance, radiusPixels: sphere.radiusRatio / pixelAngle, pixel: [...png.data.subarray(i, i + 3)], state })
  images.push(actual)
 }
 expect(measurements[0].radiusPixels).toBeGreaterThan(.8)
 expect(measurements[1].radiusPixels).toBeLessThan(.8)
 const delta = Math.max(...measurements[0].pixel.map((v, i) => Math.abs(v - measurements[1].pixel[i])))
 await info.attach('point-before-cutoff.png', { body: images[0], contentType: 'image/png' })
 await info.attach('point-after-cutoff.png', { body: images[1], contentType: 'image/png' })
 await info.attach('point-transition.json', { body: json({ measurements, delta }), contentType: 'application/json' })
 expect(delta, 'UNRESOLVED_POINT_POP').toBeLessThanOrEqual(2)
 expect((await capture(page)).equals(images[1])).toBe(true)
})
