import { test, expect } from '@playwright/test'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { boot, scene, walkTo, walkTicks, position } from './worldJourney'

test('COCKPIT-1 radar round trip, reserved seat, blackout and persistent simulation', async ({ browser }, info) => {
  const originals: Record<string, Buffer> = {}, measurements = []
  for (let replay = 0; replay < 2; replay++) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
    const page = await context.newPage(), errors: string[] = []
    page.on('pageerror', e => errors.push(e.message))
    try {
      await boot(page, '?powerFixture=1')
      const main = page.locator('main'), world = await scene(page).elementHandle()
      const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
      const capture = async (name: string) => {
        await page.clock.runFor(5000)
        const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
        if (replay === 0) originals[name] = png
        else {
          const r = compareImages(originals[name]!, png)
          await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: r.pass, changedPixels: r.changedPixels, rawChangedPixels: r.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
          await info.attach(`${name}-diff.png`, { body: r.diff, contentType: 'image/png' })
          expect(r.pass, name).toBe(true)
        }
      }
      for (const p of [[-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
      for (const p of [[0, 7], [0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
      const before = await ship()
      await page.keyboard.press('e')
      await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-occupant', 'player:local')
      await expect.poll(async () => { await page.clock.runFor(50); return page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true }).count() }).toBe(1)
      await expect(page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })).toBeVisible()
      await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-position', '0,-3.95')
      await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
      const entered = await ship()
      expect(entered.position).toEqual(before.position); expect(entered.velocityMps).toEqual(before.velocityMps)
      const backend = await page.locator('canvas').evaluate(c => {
        const gl = (c as HTMLCanvasElement).getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!
        return { renderer: gl.getParameter(ext.UNMASKED_RENDERER_WEBGL), version: gl.getParameter(gl.VERSION) }
      })
      await capture('radar-open')
      await expect(page.getByLabel('Throttle', { exact: true })).toBeDisabled(); await page.clock.runFor(500)
      const moving = await ship()
      expect(moving.throttle).toBe(0); expect(moving.velocityMps).toEqual([0, 0, 0]); expect(moving.mode).toBe('grounded')
      await page.getByRole('button', { name: 'Exit navigation · Stand up' }).click()
      const exited = await ship()
      expect(exited.position).toEqual(before.position); expect(exited.contact.bodyId).toBe('vesper')
      expect(exited.yaw).toBe(before.yaw); expect(exited.pitch).toBe(before.pitch)
      await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
      await capture('standing-return')
      const continued = await ship()
      expect(continued.throttle).toBe(0); expect(continued.tick).toBeGreaterThan(exited.tick); expect(continued.position).toEqual(exited.position)
      await page.keyboard.press('e'); await expect(page.getByLabel('Throttle', { exact: true })).toHaveValue('0')
      await page.clock.runFor(32)
      await page.getByRole('button', { name: 'Test cockpit power loss' }).click()
      await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
      await expect(main).toHaveAttribute('data-occupant', 'player:local'); await capture('blackout-seated')
      const seat = await position(page)
      await walkTicks(page, 's', 12); expect(await position(page)).toEqual(seat)
      await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
      await page.getByRole('button', { name: 'Restore cockpit power' }).click(); await page.clock.runFor(100)
      await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
      await capture('restored-seated')
      await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
      await page.getByRole('button', { name: 'Test cockpit power loss' }).click()
      await page.getByRole('button', { name: 'Stand up', exact: true }).click()
      await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
      expect(await world!.evaluate(n => n.isConnected)).toBe(true); expect(errors).toEqual([])
      measurements.push({ replay, before, entered, moving, exited, continued, final: await ship(), backend, errors })
    } finally { await context.close() }
  }
  await info.attach('cockpit-measurements.json', { body: JSON.stringify(measurements, null, 2), contentType: 'application/json' })
})
