import { test, expect } from '@playwright/test'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { relativeM } from '../src/coordinates.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
import { boot, walkTo, walkTicks } from './worldJourney'

// Shipping route, fresh storage, real input throughout; no pose/save injection.
test.use({ trace: { mode: 'on', snapshots: false, screenshots: false, sources: true } })
test('LOOP-845 fresh outdoor launch, manual flight, original-site landing and depleted source return', async ({ page, browser }, info) => {
 test.setTimeout(300000)
 const errors: string[] = [], states: Record<string, unknown> = {}
 page.on('pageerror', e => errors.push(e.message))
 await boot(page, '', 'http://127.0.0.1:4175')
 const main = page.locator('main'), pano = page.locator('.panoramic')
 const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
 const save = async () => {
  await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
  return page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
 }
 const advance = async (ms: number) => { for (let t = 0; t < ms; t += 100) await page.clock.fastForward(Math.min(100, ms - t)) }
 const capture = async (name: string) => {
  const original = await page.screenshot({ path: info.outputPath(`${name}.png`) })
  const repeat = await page.screenshot({ path: info.outputPath(`${name}-repeat.png`) })
  const result = compareImages(original, repeat)
  await info.attach(`${name}-comparison.json`, { body: JSON.stringify({ ...result, diff: undefined }), contentType: 'application/json' })
  expect(result.pass, `${name} deterministic repeat`).toBe(true)
  return original
 }
 const cabin = async (reverse = false) => {
  const path = [[0, 4.7], [0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]
  for (const p of reverse ? path.reverse() : path) await walkTo(page, p, 'ship:scout', true)
 }
 const initial = await ship()
 await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
 await capture('initial-outdoor')
 await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
 await page.getByRole('button', { name: 'Equip gathering tool' }).click()
 await page.keyboard.press('e'); await walkTicks(page, [], 60); await page.keyboard.press('Escape')
 await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
 const resources = await save()
 for (const p of [[-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
 await walkTo(page, [0, 7], 'ship:scout', true); await cabin()
 await page.keyboard.press('e'); await page.clock.runFor(64)
 await expect(page.getByRole('button', { name: 'Take off', exact: true })).toBeDisabled()
 await expect(page.locator('#takeoff-status')).toContainText('Close the aft boarding door')
 await capture('open-door-warning')
 await page.getByRole('button', { name: 'Exit cockpit' }).click(); await page.clock.runFor(64)
 await cabin(true); await page.keyboard.press('e'); await page.clock.runFor(32)
 await cabin(); await page.keyboard.press('e'); await page.clock.runFor(64)
 const backend = await page.locator('canvas').evaluate(c => {
  const gl = (c as HTMLCanvasElement).getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!
  return gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)
 })
 await page.getByRole('button', { name: 'Take off', exact: true }).click()
 let previous = await ship()
 for (let i = 0; i < 65; i++) {
  await advance(100)
  const next = await ship()
  // Bound each displacement by elapsed ticks and the launch's conservative speed.
  expect(Math.hypot(...relativeM(next.position, previous.position)), 'LOOP_NO_TELEPORT').toBeLessThanOrEqual((next.tick - previous.tick) / 60 * 100 + .001)
  previous = next
  if (i === 29) await capture('panoramic-ascent')
 }
 expect((await ship()).mode, 'LOOP_TAKEOFF_COMPLETES').toBe('flying')
 const view = await pano.getAttribute('data-observation'), before = await ship(), tick = Number(await main.getAttribute('data-tick'))
 await page.locator('canvas').focus(); await page.keyboard.down('ArrowLeft'); await advance(1000); await page.keyboard.up('ArrowLeft')
 const steered = await ship()
 expect(steered.yaw).toBeGreaterThan(before.yaw)
 expect(steered.tick - before.tick).toBe(Number(await main.getAttribute('data-tick')) - tick)
 expect(await pano.getAttribute('data-observation')).toBe(view)
 // Blur clears held controls; hidden and explicit pause freeze the one clock.
 await page.keyboard.down('ArrowLeft'); await page.evaluate(() => window.dispatchEvent(new Event('blur')))
 await advance(200); await page.keyboard.up('ArrowLeft'); expect((await ship()).yaw).toBe(steered.yaw)
 await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: true }); document.dispatchEvent(new Event('visibilitychange')) })
 const hidden = await ship(); await advance(1000); expect(await ship()).toEqual(hidden)
 await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: false }); document.dispatchEvent(new Event('visibilitychange')) })
 await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
 const paused = await ship(); await advance(1000); expect(await ship()).toEqual(paused)
 await page.getByRole('button', { name: 'Resume [P]' }).click()
 await page.getByRole('button', { name: 'Exit cockpit' }).click(); await page.clock.runFor(64)
 await expect(page.locator('[data-entity][data-frame="planet:fixture"]')).toHaveCount(0)
 await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
 await expect(page.locator('[data-entity="person:player"]')).toHaveCount(1)
 await capture('inflight-cabin')
 const flyingSave = await save()
 await page.reload(); await page.clock.runFor(64)
 const restored = await ship(), elapsed = (restored.tick - flyingSave.scout.ship.tick) / 60
 const displacement = relativeM(restored.position, flyingSave.scout.ship.position)
 for (let axis = 0; axis < 3; axis++) expect(displacement[axis]).toBeCloseTo(flyingSave.scout.ship.velocityMps[axis] * elapsed, 3)
 expect(restored.velocityMps).toEqual(flyingSave.scout.ship.velocityMps)
 expect((await ship()).mode).toBe('flying')
 await page.getByRole('button', { name: 'Use cockpit', exact: true }).click(); await page.clock.runFor(64)
 await expect(page.getByRole('button', { name: 'Land', exact: true })).toBeEnabled()
 const original = await capture('panoramic-approach')
 const style = await page.addStyleTag({ content: '.landing-control{visibility:hidden!important}' })
 const broken = await page.screenshot({ path: info.outputPath('negative-missing-landing-control.png') })
 expect(compareImages(original, broken).pass, 'lost landing control must fail visual comparison').toBe(false)
 await style.evaluate(e => e.parentNode!.removeChild(e))
 await page.getByRole('button', { name: 'Land', exact: true }).click(); await page.clock.runFor(32)
 await expect(pano).toHaveAttribute('data-mode', 'landing')
 await expect(page.getByRole('button', { name: 'Land', exact: true })).toBeDisabled()
 previous = await ship()
 for (let i = 0; i < 150 && (await ship()).mode === 'landing'; i++) {
  await advance(100); const next = await ship()
  expect(Math.hypot(...relativeM(next.position, previous.position)), 'LOOP_NO_TELEPORT').toBeLessThanOrEqual((next.tick - previous.tick) / 60 * 100 + .001)
  previous = next
 }
 expect((await ship()).mode, 'LOOP_TOUCHDOWN_COMPLETES').toBe('grounded')
 expect((await ship()).contact.bodyId).toBe('vesper')
 expect(Math.hypot(...relativeM((await ship()).position, initial.position))).toBeLessThan(.001)
 await expect(main).toHaveAttribute('data-surface-bound', 'true')
 await cabin(true); await page.keyboard.press('e'); await page.clock.runFor(32); await walkTicks(page, 's', 70)
 await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
 await expect(page.locator('[data-entity="source:landing:mineral:01"]')).toHaveAttribute('data-remaining-grams', '59000')
 await capture('returned-surface')
 const returned = await save()
 for (const key of ['inventory', 'sources', 'stores']) expect(returned[key]).toEqual(resources[key])
 await page.reload(); await page.clock.runFor(64)
 expect((await ship()).position).toEqual(returned.scout.ship.position)
 await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
 await expect(page.locator('[data-entity="person:player"]')).toHaveCount(1)
 for (const p of [[-2, 7], [-2, 3], [-8, 3], [-8, 1.6]]) await walkTo(page, p, 'planet:fixture', true)
 await page.keyboard.press('e'); await page.clock.runFor(32)
 await expect(main).toHaveAttribute('data-player-mode', 'interacting')
 await expect(page.locator('[data-source-summary]')).toContainText('59 kg remaining')
 await capture('returned-source-inspection')
 expect(errors).toEqual([])
 Object.assign(states, { initial, before, steered, hidden, paused, flyingSave, returned })
 await info.attach('loop-state.json', { body: JSON.stringify({ states, backend, browser: browser.version(), viewport: { width: 1280, height: 800 }, dpr: 1, epoch: '2026-11-04T00:00:00Z', seed: 'sampleUniverse / surface:v1', inputs: 'Playwright keyboard/mouse on built /; no save or pose injection', visual: 'original/repeat and lost-control negative; candidates, no baseline approval' }), contentType: 'application/json' })
})
