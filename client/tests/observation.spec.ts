import { test, expect, type Page } from '@playwright/test'
import { Quaternion, Vector3 } from 'three'
import { PNG } from 'pngjs'
import { compareImages } from '../scripts/visual-comparison.ts'
const json = (value: unknown) => JSON.stringify(value, (_, item) => typeof item === 'bigint' ? item.toString() : item, 2)
type Snapshot = {
 view: { quaternion: [number, number, number, number]; fov: number }
 ship: { yaw: number; pitch: number; throttle: number; position: unknown; velocityMps: number[] }
 origin: unknown
 render: { position: number[]; quaternion: number[]; fov: number; aspect: number; width: number; height: number; dpr: number; framebuffer: number[]; renderer: string }
}
const snapshot = (page: Page) => page.evaluate(() => (window as unknown as { observationSnapshot(): Snapshot }).observationSnapshot())
async function boot(page: Page) {
 await page.clock.install({ time: new Date('2026-11-06T12:00:00Z') })
 await page.clock.pauseAt(new Date('2026-11-06T12:00:01Z'))
 await page.goto('/tests/observation.html')
 await expect.poll(async () => { await page.clock.runFor(16); return (await snapshot(page))?.render.fov }).toBe(75)
 await page.clock.runFor(160)
}
async function settle(page: Page) { await page.clock.runFor(160) }
function angle(a: number[], b: number[]) { return new Quaternion(...a).angleTo(new Quaternion(...b)) }

test('OBS-1 actual perspective camera, independent ship, reset and remount', async ({ page }, info) => {
 const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
 await boot(page)
 const initial = await snapshot(page)
 expect(initial.render.renderer).toContain('SwiftShader')
 expect(initial.render.position).toEqual([0, 0, 0])
 await info.attach('wide.png', { body: await page.screenshot(), contentType: 'image/png' })
 await page.keyboard.down('ArrowLeft'); await page.clock.runFor(1000); await page.keyboard.up('ArrowLeft')
 const steered = await snapshot(page)
 expect(steered.ship.yaw).toBeGreaterThan(initial.ship.yaw)
 expect(steered.view).toEqual(initial.view)
 await page.keyboard.down('KeyD'); await page.clock.runFor(1000); await page.keyboard.up('KeyD')
 const looked = await snapshot(page)
 expect(looked.ship.yaw).toBe(steered.ship.yaw)
 expect(angle(looked.view.quaternion, initial.view.quaternion)).toBeGreaterThan(0.1)
 await page.mouse.move(640, 400); await page.mouse.wheel(0, -10000); await settle(page)
 const zoomed = await snapshot(page)
 expect(zoomed.render.fov).toBe(0.1)
 expect(zoomed.ship.position).toEqual(initial.ship.position)
 expect(zoomed.render.position).toEqual([0, 0, 0])
 await page.getByRole('button', { name: 'Toggle scene' }).click()
 await expect(page.locator('canvas')).toHaveCount(0)
 await page.getByRole('button', { name: 'Toggle scene' }).click()
 await expect(page.locator('canvas')).toBeVisible(); await page.clock.runFor(1000)
 expect((await snapshot(page)).view).toEqual(zoomed.view)
 await page.getByRole('button', { name: 'Reset view' }).click()
 await expect.poll(async () => { await settle(page); return (await snapshot(page)).view.fov }).toBe(75)
 const reset = await snapshot(page)
 expect(reset.view.fov).toBe(75)
 const direction = new Vector3(0, 0, -1).applyQuaternion(new Quaternion(...reset.view.quaternion))
 expect(direction.x).toBeCloseTo(-Math.sin(reset.ship.yaw))
 expect(reset.ship.yaw).toBe(steered.ship.yaw)
 // Reset is one-shot; subsequent ship motion never drags the view.
 await page.locator('h1').click()
 await page.keyboard.down('ArrowUp'); await page.clock.runFor(800); await page.keyboard.up('ArrowUp')
 expect((await snapshot(page)).view).toEqual(reset.view)
 expect(errors).toEqual([])
 await info.attach('states.json', { body: json({ initial, steered, looked, zoomed, reset }), contentType: 'application/json' })
})

test('OBS-2 telescope pixels, drag, resize/DPR, wheel clamp and focus cancellation', async ({ page }, info) => {
 await boot(page)
 await page.mouse.move(640, 400); await page.mouse.wheel(0, -10000); await settle(page)
 const telescope = await snapshot(page)
 await info.attach('telescope.png', { body: await page.screenshot(), contentType: 'image/png' })
 expect(telescope.render.fov).toBe(0.1)
 await page.mouse.down(); await page.mouse.move(740, 400, { steps: 5 }); await page.mouse.up(); await settle(page)
 const drag = await snapshot(page)
 expect(angle(telescope.view.quaternion, drag.view.quaternion)).toBeGreaterThan(0.0001)
 expect(angle(telescope.view.quaternion, drag.view.quaternion)).toBeLessThan(0.001)
 await page.mouse.down({ button: 'right' }); await page.mouse.move(840, 450); await page.mouse.up({ button: 'right' }); await settle(page)
 expect((await snapshot(page)).view).toEqual(drag.view)
 await page.keyboard.down('KeyW'); await page.clock.runFor(160)
 await page.getByLabel('Notes').focus(); await page.keyboard.up('KeyW'); await settle(page)
 const focused = await snapshot(page)
 await page.keyboard.type('wasd'); await page.keyboard.press('ArrowLeft')
 await page.mouse.move(640, 400); await page.mouse.wheel(0, 10000); await settle(page)
 expect((await snapshot(page)).view).toEqual(focused.view)
 expect((await snapshot(page)).ship.yaw).toBe(focused.ship.yaw)
 await page.locator('h1').click(); await page.keyboard.down('KeyD'); await page.clock.runFor(160)
 await page.evaluate(() => window.dispatchEvent(new Event('blur'))); await settle(page)
 const blurred = await snapshot(page); await page.clock.runFor(320)
 expect((await snapshot(page)).view).toEqual(blurred.view); await page.keyboard.up('KeyD')
 await page.setViewportSize({ width: 960, height: 600 }); await settle(page)
 const resized = await snapshot(page)
 expect(resized.view).toEqual(blurred.view)
 expect(resized.render.aspect).toBeCloseTo(resized.render.width / resized.render.height)
 expect(resized.render.framebuffer[0]).toBe(Math.round(resized.render.width * resized.render.dpr))
 await page.mouse.move(480, 300); await page.mouse.wheel(0, 10000); await settle(page)
 expect((await snapshot(page)).render.fov).toBe(75)
 await info.attach('states.json', { body: json({ telescope, drag, focused, blurred, resized }), contentType: 'application/json' })
})

test('OBS-3 DPR 2 preserves projection and origin', async ({ browser }, info) => {
 const context = await browser.newContext({ viewport: { width: 960, height: 600 }, deviceScaleFactor: 2 })
 const page = await context.newPage(); await boot(page)
 const state = await snapshot(page)
 expect(state.render.dpr).toBe(2)
 expect(state.render.framebuffer).toEqual([state.render.width * 2, state.render.height * 2])
 expect(state.render.aspect).toBeCloseTo(state.render.width / state.render.height)
 expect(state.render.position).toEqual([0, 0, 0]); expect(state.render.fov).toBe(75)
 await info.attach('dpr2.png', { body: await page.screenshot(), contentType: 'image/png' })
 await context.close()
})

test('OBS-4 repeated wide/telescope pixels and negative comparison', async ({ browser }, info) => {
 const originals: Buffer[] = []
 const cyanPixels = (buffer: Buffer) => {
  const png = PNG.sync.read(buffer); let count = 0
  for (let i = 0; i < png.data.length; i += 4) if (png.data[i] < 40 && png.data[i + 1] > 180 && png.data[i + 2] > 180) count++
  return count
 }
 const results = []
 for (let run = 0; run < 2; run++) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } })
  const page = await context.newPage(); await boot(page)
  for (let zoom = 0; zoom < 2; zoom++) {
   if (zoom) { await page.mouse.move(640, 400); await page.mouse.wheel(0, -10000); await settle(page) }
   const png = await page.locator('canvas').screenshot()
   await info.attach(`run-${run}-${zoom ? 'telescope' : 'wide'}.png`, { body: png, contentType: 'image/png' })
   if (!run) originals.push(png)
   else {
    const result = compareImages(originals[zoom], png)
    results.push({ zoom, pass: result.pass, changedPixels: result.changedPixels, rawChangedPixels: result.rawChangedPixels })
    expect(result.pass).toBe(true)
   }
  }
  await context.close()
 }
 expect(cyanPixels(originals[1])).toBeGreaterThan(100)
 expect(cyanPixels(originals[1])).toBeGreaterThan(cyanPixels(originals[0]))
 const negative = compareImages(originals[0], originals[1])
 expect(negative.pass).toBe(false)
 await info.attach('comparisons.json', { body: json({ results, negative: { pass: negative.pass, changedPixels: negative.changedPixels }, cyan: originals.map(cyanPixels) }), contentType: 'application/json' })
})

test('OBS-5 spherical keyboard continuity, visibility/pointer cancellation and moving ship origin', async ({ page }, info) => {
 test.setTimeout(120000) // Two full-sphere software-WebGL turns, ~960 rendered frames.
 await boot(page)
 const samples: Snapshot[] = []
 for (const key of ['KeyW', 'KeyD']) {
  await page.getByRole('button', { name: 'Reset view' }).click(); await settle(page)
  await page.locator('h1').click()
  let previous = await snapshot(page)
  await page.keyboard.down(key)
  // More than 360 degrees per axis, sampling across both poles/backward.
  for (let i = 0; i < 32; i++) {
   await page.clock.runFor(240)
   const next = await snapshot(page)
   expect(angle(previous.view.quaternion, next.view.quaternion)).toBeGreaterThan(0.1)
   expect(angle(previous.view.quaternion, next.view.quaternion)).toBeLessThan(0.3)
   expect(next.ship.yaw).toBe(0); expect(next.ship.pitch).toBe(0)
   samples.push(next); previous = next
  }
  await page.keyboard.up(key)
 }
 await page.keyboard.down('KeyA'); await page.clock.runFor(160)
 await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange')))
 await settle(page); const cancelled = await snapshot(page)
 await page.clock.runFor(320)
 expect((await snapshot(page)).view).toEqual(cancelled.view)
 await page.keyboard.up('KeyA')
 await page.mouse.move(640, 400); await page.mouse.down()
 await page.locator('canvas').dispatchEvent('pointercancel'); await page.mouse.move(740, 400); await page.mouse.up(); await settle(page)
 expect((await snapshot(page)).view).toEqual(cancelled.view)
 // Real throttle input makes the production fixed-step ship translate.
 await page.getByLabel('Throttle').fill('100'); await page.clock.runFor(1000)
 const moving = await snapshot(page)
 expect(moving.origin).toEqual(moving.ship.position)
 expect(moving.ship.position).not.toEqual(cancelled.ship.position)
 expect(moving.render.position).toEqual([0, 0, 0])
 expect(moving.view).toEqual(cancelled.view)
 await info.attach('states.json', { body: json({ samples, cancelled, moving }), contentType: 'application/json' })
})
