import { relativeM } from '../src/coordinates.ts'
import { distances } from '../src/contact.ts'
import { createTopDownState } from '../src/topDownRuntime.ts'
import { test, expect } from '@playwright/test'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { boot, scene, walkTo, walkTicks, position, positionIn } from './worldJourney'

// One uninterrupted user journey through the built production entry. The
// second fresh browser context replays the same actual input, not state setters.
// Original captures and raw state are explicit below; avoid synchronizing every
// software WebGL frame for a trace filmstrip. Failure action traces remain.
test.use({ trace: { mode: 'retain-on-failure', snapshots: false, screenshots: false, sources: true } })

test('SLICE-1 built outdoor sources, doorway, cutaway, radar, standing exit and return', async ({ browser }, info) => {
  test.setTimeout(300000)
  const originals: Record<string, Buffer> = {}, journeys = []
  for (let replay = 0; replay < 2; replay++) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
    const page = await context.newPage(), errors: string[] = []
    page.on('pageerror', e => errors.push(e.message))
    try {
      await boot(page, '', 'http://127.0.0.1:4175')
      const main = page.locator('main'), node = await scene(page).elementHandle()
      const frame = () => page.locator('[data-entity="person:player"]').getAttribute('data-frame')
      const sourceState = () => page.locator('[data-source-kind]').evaluateAll(nodes => nodes.map(n => ({ id: n.getAttribute('data-entity'), kind: n.getAttribute('data-source-kind'), frame: n.getAttribute('data-frame'), position: n.getAttribute('data-position'), site: n.getAttribute('data-source-site') })))
      const shipState = async () => JSON.parse((await main.getAttribute('data-ship'))!)
      const initial = await shipState()
      const location = createTopDownState('exterior', true).scoutLocation
      const serialized = JSON.stringify(location.ship.position, (_, v) => typeof v === 'bigint' ? v.toString() : v)
      const assertGrounded = async () => {
        const ship = await shipState()
        expect(ship.position).toEqual(JSON.parse(serialized))
        expect(ship.mode).toBe('grounded'); expect(ship.contact.bodyId).toBe(location.body.id)
        expect(ship.velocityMps).toEqual([0, 0, 0]); expect(ship.throttle).toBe(0)
        expect(ship.yaw).toBe(location.frames.ship.angle); expect(ship.pitch).toBe(0)
        return ship
      }
      await assertGrounded()
      expect(Math.abs(distances(location.ship, location.body).surfaceDistanceM)).toBeLessThan(.0001)
      expect(relativeM(location.ship.position, location.body.position)[1]).toBeGreaterThan(location.body.radiusKm * 1000)
      const sources = await sourceState(), checkpoints: unknown[] = [], handoffs: unknown[] = []
      expect(sources.map(s => s.id).sort()).toEqual(['source:landing:biology:01', 'source:landing:mineral:01'])
      expect(await position(page)).toEqual([-6, 3]); expect(await frame()).toBe('planet:fixture')
      await expect(page.locator('canvas')).toHaveCount(0)
      const capture = async (name: string) => {
        await page.clock.runFor(await page.locator('.panoramic').count() ? 64 : 5000)
        const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`), fullPage: true })
        const checkpoint = { ship: await shipState(), location: await main.getAttribute('data-scout-location'), name, position: await position(page), planet: await positionIn(page, 'planet:fixture'), frame: await frame(), posture: await main.getAttribute('data-player-mode'), cameraGroundTransform: await page.locator('[data-entity="terrain:site"]').getAttribute('transform') }
        checkpoints.push(checkpoint)
        await info.attach(`${name}-${replay}-state.json`, { body: JSON.stringify(checkpoint), contentType: 'application/json' })
        if (replay === 0) originals[name] = png
        else {
          const r = compareImages(originals[name]!, png)
          await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: r.pass, changedPixels: r.changedPixels, rawChangedPixels: r.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
          await info.attach(`${name}-diff.png`, { body: r.diff, contentType: 'image/png' })
          expect(r.pass, name).toBe(true)
        }
      }
      await capture('slice-outdoor-spawn')
      await page.keyboard.press('e'); await expect(page.getByRole('status')).toContainText('No available object')
      // Walk to both physical source footprints and use their shared interaction.
      await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 100)
      expect((await position(page))[0]).toBeCloseTo(-7.34, 6)
      await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-target', 'source:landing:mineral:01')
      await capture('slice-mineral-inspection')
      await page.keyboard.press('Escape')
      await walkTicks(page, 'd', 34); await walkTicks(page, 's', 110); await walkTicks(page, 'd', 25)
      await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-target', 'source:landing:biology:01')
      await page.keyboard.press('Escape')
      // Exit the colony's reach to starboard, then approach the aft cargo entry.
      await walkTo(page, [-2, 4.4], 'planet:fixture', true)
      await walkTo(page, [-2, 7], 'planet:fixture', true)
      await walkTo(page, [0, 7], 'ship:scout', true)
      await walkTo(page, [0, 5.65], 'ship:scout', true)
      // Original exterior view contains player, exposed rooms and surface props.
      for (const id of ['scout:floor', 'scout:desk', 'scout:core', 'scout:storage', 'source:landing:biology:01']) {
        const box = (await page.locator(`[data-entity="${id}"]`).boundingBox())!, bounds = (await scene(page).boundingBox())!
        expect(box.x + box.width).toBeGreaterThan(bounds.x); expect(box.x).toBeLessThan(bounds.x + bounds.width)
        expect(box.y + box.height).toBeGreaterThan(bounds.y); expect(box.y).toBeLessThan(bounds.y + bounds.height)
      }
      await capture('slice-exterior-cutaway')
      const cross = async (key: string, finalFrame: string) => {
        const samples = []
        for (let tick = 0; tick < 14; tick++) {
          const before = await positionIn(page, 'planet:fixture'), oldFrame = await frame()
          const oldBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
          await walkTicks(page, key, 1)
          const after = await positionIn(page, 'planet:fixture'), newFrame = await frame()
          const newBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
          expect(Math.hypot(after[0] - before[0], after[1] - before[1])).toBeCloseTo(.04, 7)
          expect(Math.hypot(newBox.x - oldBox.x, newBox.y - oldBox.y)).toBeLessThan(3)
          samples.push({ before, after, oldFrame, newFrame, oldBox, newBox })
        }
        expect(samples.filter(s => s.oldFrame !== s.newFrame)).toHaveLength(1)
        expect(await frame()).toBe(finalFrame); handoffs.push(samples)
      }
      await cross('w', 'ship:scout')
      for (const p of [[0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
      await page.keyboard.press('e')
      await expect.poll(async () => { await page.clock.runFor(50); return page.getByLabel('Panoramic universe', { exact: true }).count() }).toBe(1)
      await expect(main).toHaveAttribute('data-player-mode', 'seated')
      await expect(page.getByLabel('Throttle', { exact: true })).toBeDisabled()
      await expect(page.getByText('grounded · Vesper', { exact: false })).toBeVisible()
      await expect(page.getByRole('button', { name: 'Restart navigation' })).toHaveCount(0)
      for (let elapsed = 0; elapsed < 5200; elapsed += 200) await page.clock.fastForward(200)
      await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
      await page.getByRole('button', { name: 'Vesper', exact: true }).click()
      await page.keyboard.press('Escape')
      await expect(page.getByRole('button', { name: 'Set direction' })).toBeDisabled()
      await page.keyboard.down('ArrowLeft'); await page.clock.runFor(200); await page.keyboard.up('ArrowLeft')
      await page.getByRole('button', { name: 'Reset view', exact: true }).click()
      const anchored = await assertGrounded()
      expect(anchored.tick).toBeGreaterThan(initial.tick)
      await capture('slice-grounded-panoramic')
      const backend = await page.locator('canvas').evaluate(c => {
        const gl = (c as HTMLCanvasElement).getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!
        return { renderer: gl.getParameter(ext.UNMASKED_RENDERER_WEBGL), version: gl.getParameter(gl.VERSION) }
      })
      await page.getByRole('button', { name: 'Exit cockpit' }).click()
      await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
      await capture('slice-standing-return')
      await page.keyboard.press('e')
      await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
      for (let elapsed = 0; elapsed < 5200; elapsed += 200) await page.clock.fastForward(200)
      await assertGrounded()
      await page.getByRole('button', { name: 'Exit cockpit' }).click()
      await assertGrounded()
      for (const p of [[0, -1.65], [1.35, -1.65], [1.35, .45], [0, .45], [0, 2], [0, 5.3]]) await walkTo(page, p, 'ship:scout', true)
      await cross('s', 'planet:fixture')
      await walkTo(page, [0, 7], 'ship:scout', true)
      await walkTo(page, [-2, 7], 'planet:fixture', true)
      await walkTo(page, [-2, 3], 'planet:fixture', true)
      await walkTo(page, [-6, 3], 'planet:fixture', true)
      await capture('slice-returned-outdoors')
      expect(await sourceState()).toEqual(sources)
      expect(await node!.evaluate(n => n.isConnected)).toBe(true)
      expect(await frame()).toBe('planet:fixture'); expect(errors).toEqual([])
      const ship = JSON.parse((await main.getAttribute('data-ship'))!)
      await assertGrounded()
      const savedPosition = await position(page)
      await page.evaluate(() => window.dispatchEvent(new PageTransitionEvent('pagehide')))
      await page.reload()
      await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
      await expect(page.getByText('Expedition saves automatically on this browser.')).toBeVisible()
      expect(await position(page)).toEqual(savedPosition)
      await assertGrounded()
      await capture('slice-reloaded-outdoors')
      journeys.push({ replay, sources, checkpoints, handoffs, ship, backend, errors, browser: browser.version(), input: 'actual keyboard/mouse via production adapters; read-only DOM observations', origin: 'built preview :4175', viewport: page.viewportSize(), dpr: await page.evaluate(() => devicePixelRatio) })
    } finally { await context.close() }
  }
  const wrong = compareImages(originals['slice-outdoor-spawn']!, originals['slice-mineral-inspection']!)
  expect(wrong.pass).toBe(false); expect(wrong.changedPixels).toBeGreaterThan(100)
  await info.attach('slice-journeys.json', { body: JSON.stringify({ journeys, wrongCheckpoint: { pass: wrong.pass, changedPixels: wrong.changedPixels, rawChangedPixels: wrong.rawChangedPixels }, clock: '2026-11-04T00:00:00Z fixed-step controlled browser clock', baseline: 'review candidates; no approved references changed', future: ['mining extraction/depletion/inventory persistence', 'Core energy', 'Fabricator production', 'multiplayer', 'ship damage'] }, null, 2), contentType: 'application/json' })
})
