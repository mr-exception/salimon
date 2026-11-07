import { PNG } from 'pngjs'
import { test, expect } from '@playwright/test'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { boot, scene, walkTo, walkTicks, position } from './worldJourney'

// Original captures and raw state are explicit below; avoid synchronizing every
// software WebGL frame for a trace filmstrip. Failure action traces remain.
test.use({ trace: { mode: 'retain-on-failure', snapshots: false, screenshots: false, sources: true } })

test('COCKPIT-1 panoramic round trip, reserved seat, blackout and persistent simulation', async ({ browser }, info) => {
  // Two complete software-WebGL replays may exceed six wall-clock minutes;
  // simulation intervals, assertions and image thresholds remain unchanged.
  test.setTimeout(600000)
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
        await page.clock.runFor(await page.locator('.panoramic').count() ? 64 : 5000)
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
      await page.keyboard.press('i'); await page.clock.runFor(32)
      await expect(page.locator('#field-inventory')).toBeVisible()
      await page.locator('body').click({ position: { x: 640, y: 400 } })
      await page.keyboard.press('e')
      await expect(page.locator('[aria-controls=field-inventory]')).toHaveAttribute('aria-expanded', 'false')
      await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-occupant', 'player:local')
      await expect.poll(async () => { await page.clock.runFor(50); return page.getByLabel('Panoramic universe', { exact: true }).count() }).toBe(1)
      await expect(page.getByLabel('Panoramic universe', { exact: true })).toBeVisible()
      await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-position', '0,-3.95')
      await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
      const view = async () => JSON.parse((await page.locator('.panoramic').getAttribute('data-observation'))!)
      const enteredView = await view()
      expect(enteredView.fov).toBe(75)
      expect(await page.locator('canvas').count()).toBe(1)
      expect(await page.locator('.radar').count()).toBe(0)
      expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, width: 1280, height: 800 })
      const entered = await ship()
      expect(entered.position).toEqual(before.position); expect(entered.velocityMps).toEqual(before.velocityMps)
      const backend = await page.locator('canvas').evaluate(c => {
        const gl = (c as HTMLCanvasElement).getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!
        return { renderer: gl.getParameter(ext.UNMASKED_RENDERER_WEBGL), version: gl.getParameter(gl.VERSION) }
      })
      await capture('panoramic-horizon')
      const horizon = PNG.sync.read(originals['panoramic-horizon'])
      const rgb = (x: number, y: number) => [...horizon.data.subarray((y * horizon.width + x) * 4, (y * horizon.width + x) * 4 + 3)]
      expect(rgb(800, 200)).not.toEqual(rgb(800, 600)) // physical globe below, sky above
      expect(rgb(800, 200)).not.toEqual([255, 255, 255])
      await expect(page.getByLabel('Throttle', { exact: true })).toBeDisabled(); await page.clock.runFor(500)
      const moving = await ship()
      expect(moving.throttle).toBe(0); expect(moving.velocityMps).toEqual([0, 0, 0]); expect(moving.mode).toBe('grounded')
      await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
      await page.getByRole('button', { name: 'Vesper', exact: true }).click()
      await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeDisabled()
      await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
      await page.getByRole('button', { name: 'Vesper Moon', exact: true }).click()
      await page.getByRole('button', { name: 'Look at', exact: true }).click()
      for (let frame = 0; frame < 25; frame++) await page.clock.fastForward(50)
      await page.keyboard.press('Escape') // dismiss browser before cockpit exit
      await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
      await page.locator('canvas').focus()
      await page.mouse.move(640, 400); await page.mouse.wheel(0, -400)
      await expect.poll(async () => { await page.clock.runFor(32); return (await view()).fov }).toBeLessThan(75)
      await page.keyboard.down('d'); await page.clock.runFor(240); await page.keyboard.up('d')
      await page.keyboard.down('ArrowLeft'); await page.clock.runFor(240); await page.keyboard.up('ArrowLeft')
      await page.clock.runFor(100)
      const observedShip = await ship(), savedView = await view()
      expect(observedShip.yaw).toBe(before.yaw); expect(observedShip.position).toEqual(before.position)
      expect(savedView.quaternion).not.toEqual(enteredView.quaternion)
      expect(savedView.fov).toBeLessThan(75)
      await capture('external-body-observation')
      // Held camera input must be released on unmount.
      await page.keyboard.down('w')
      await page.getByRole('button', { name: 'Exit cockpit' }).click()
      const exited = await ship()
      expect(exited.position).toEqual(before.position); expect(exited.contact.bodyId).toBe('vesper')
      expect(exited.yaw).toBe(before.yaw); expect(exited.pitch).toBe(before.pitch)
      await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
      await capture('standing-return')
      const continued = await ship()
      expect(continued.throttle).toBe(0); expect(continued.tick).toBeGreaterThan(exited.tick); expect(continued.position).toEqual(exited.position)
      await page.keyboard.press('e'); await expect(page.getByLabel('Throttle', { exact: true })).toHaveValue('0')
      await page.clock.runFor(100)
      expect(await view()).toEqual(savedView)
      await page.keyboard.up('w')
      const tickBefore = (await ship()).tick
      // Preserve elapsed domain time with bounded frame gaps on software WebGL.
      for (let elapsed = 0; elapsed < 1000; elapsed += 100) await page.clock.fastForward(100)
      const ticks = (await ship()).tick - tickBefore
      expect(ticks).toBeGreaterThanOrEqual(59); expect(ticks).toBeLessThanOrEqual(61)
      await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
      await page.getByRole('button', { name: 'Caligo', exact: true }).click()
      await page.getByRole('button', { name: 'Look at', exact: true }).click()
      await page.evaluate(() => window.dispatchEvent(new Event('blur')))
      await page.clock.runFor(100)
      const blurredView = await view()
      // Preserve elapsed domain time with bounded frame gaps on software WebGL.
      for (let elapsed = 0; elapsed < 1000; elapsed += 100) await page.clock.fastForward(100)
      expect(await view()).toEqual(blurredView)
      await page.keyboard.press('Escape') // browser closes first
      const beforeBlackout = await view()
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
      await page.clock.runFor(100); expect(await view()).toEqual(beforeBlackout)
      await page.getByRole('button', { name: 'Reset view', exact: true }).click()
      await page.clock.runFor(100); expect(await view()).toEqual(enteredView)
      await page.keyboard.press('Escape')
      await expect(main).toHaveAttribute('data-player-mode', 'standing')
      await page.keyboard.press('e'); await page.clock.runFor(100)
      await page.getByRole('button', { name: 'Test cockpit power loss' }).click()
      await page.getByRole('button', { name: 'Stand up', exact: true }).click()
      await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
      expect(await world!.evaluate(n => n.isConnected)).toBe(true); expect(errors).toEqual([])
      measurements.push({ replay, before, entered, moving, exited, continued, final: await ship(), enteredView, savedView, beforeBlackout, ticks, backend, errors })
    } finally { await context.close() }
  }
  await info.attach('cockpit-measurements.json', { body: JSON.stringify(measurements, null, 2), contentType: 'application/json' })
})
