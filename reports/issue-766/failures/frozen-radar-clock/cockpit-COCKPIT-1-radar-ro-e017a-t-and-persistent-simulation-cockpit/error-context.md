# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: cockpit.spec.ts >> COCKPIT-1 radar round trip, reserved seat, blackout and persistent simulation
- Location: tests/cockpit.spec.ts:5:1

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })
Expected: visible
Timeout: 10000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true }) with timeout 10000ms
  - waiting for getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })

```

```yaml
- region "Navigation radar":
  - text: SALIMON 3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE
  - img "Ship heading and velocity vector lines"
  - text: "HEADING STATIONARY · Scanner 100,000 light-seconds · pulse #0"
- complementary:
  - heading "NAVIGATION" [level=1]
  - paragraph: Navigation-only prototype · unlimited energy
  - status: "Scanner: awaiting first five-second pulse"
  - text: Total ship mass
  - strong: 100,000 kg
  - text: Relative position
  - strong: 0.0 / 0.0 / 0.0 m (cell local)
  - text: Speed (sample inertial frame)
  - strong: 0.0 m/s
  - text: Yaw / pitch
  - strong: 0.0° / 0.0°
  - text: Alignment
  - strong: Manual
  - text: Flight status
  - strong: flying
  - status "Navigation warnings":
    - paragraph: No navigation warnings
  - separator
  - heading "Contact" [level=2]
  - status: No selected target
  - text: Find scanned contacts
  - searchbox "Find scanned contacts"
  - text: Scanned contacts (5)
  - combobox "Scanned contacts (5)":
    - option "Choose a contact" [selected]
    - option "Caligo"
    - option "Nacre"
    - option "Uncharted asteroid"
    - option "Vesper"
    - option "Vesper Moon"
  - paragraph: Select a visible contact or use the scanned-contact list.
  - separator
  - heading "Camera and orientation" [level=2]
  - paragraph: WASD pans the view · wheel zooms · arrow keys turn the ship.
  - button "Help"
  - button "Reset camera"
  - separator
  - heading "Throttle 0%" [level=2]:
    - text: Throttle
    - strong: 0%
  - slider "Throttle": "0"
  - button "Zero throttle (Space)"
  - paragraph: Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run. Lower-speed contact settles the ship.
- button "Exit navigation · Stand up"
- button "Test cockpit power loss"
```

# Test source

```ts
  1  | import { test, expect } from '@playwright/test'
  2  | import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
  3  | import { boot, scene, walkTo, walkTicks, position } from './worldJourney'
  4  | 
  5  | test('COCKPIT-1 radar round trip, reserved seat, blackout and persistent simulation', async ({ browser }, info) => {
  6  |   const originals: Record<string, Buffer> = {}, measurements = []
  7  |   for (let replay = 0; replay < 2; replay++) {
  8  |     const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
  9  |     const page = await context.newPage(), errors: string[] = []
  10 |     page.on('pageerror', e => errors.push(e.message))
  11 |     try {
  12 |       await boot(page, '?powerFixture=1')
  13 |       const main = page.locator('main'), world = await scene(page).elementHandle()
  14 |       const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
  15 |       const capture = async (name: string) => {
  16 |         await page.clock.runFor(5000)
  17 |         const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
  18 |         if (replay === 0) originals[name] = png
  19 |         else {
  20 |           const r = compareImages(originals[name]!, png)
  21 |           await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: r.pass, changedPixels: r.changedPixels, rawChangedPixels: r.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
  22 |           await info.attach(`${name}-diff.png`, { body: r.diff, contentType: 'image/png' })
  23 |           expect(r.pass, name).toBe(true)
  24 |         }
  25 |       }
  26 |       for (const p of [[-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
  27 |       for (const p of [[0, 7], [0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
  28 |       const before = await ship()
  29 |       await page.keyboard.press('e')
  30 |       await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-occupant', 'player:local')
> 31 |       await expect(page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })).toBeVisible()
     |                                                                                                           ^ Error: expect(locator).toBeVisible() failed
  32 |       await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-position', '0,-3.95')
  33 |       await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
  34 |       const entered = await ship()
  35 |       expect(entered.position).toEqual(before.position); expect(entered.velocityMps).toEqual(before.velocityMps)
  36 |       const backend = await page.locator('canvas').evaluate(c => {
  37 |         const gl = (c as HTMLCanvasElement).getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!
  38 |         return { renderer: gl.getParameter(ext.UNMASKED_RENDERER_WEBGL), version: gl.getParameter(gl.VERSION) }
  39 |       })
  40 |       await capture('radar-open')
  41 |       await page.getByLabel('Throttle', { exact: true }).fill('37'); await page.clock.runFor(500)
  42 |       const moving = await ship()
  43 |       expect(moving.throttle).toBe(37); expect(Math.abs(moving.velocityMps[2])).toBeGreaterThan(0)
  44 |       await page.getByRole('button', { name: 'Exit navigation · Stand up' }).click()
  45 |       const exited = await ship()
  46 |       expect(exited.throttle).toBe(37); expect(exited.position).toEqual(moving.position); expect(exited.velocityMps).toEqual(moving.velocityMps)
  47 |       await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
  48 |       await capture('standing-return')
  49 |       const continued = await ship()
  50 |       expect(continued.throttle).toBe(37); expect(continued.tick).toBeGreaterThan(exited.tick); expect(continued.position).not.toEqual(exited.position)
  51 |       await page.keyboard.press('e'); await expect(page.getByLabel('Throttle', { exact: true })).toHaveValue('37')
  52 |       await page.clock.runFor(32)
  53 |       await page.getByRole('button', { name: 'Test cockpit power loss' }).click()
  54 |       await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
  55 |       await expect(main).toHaveAttribute('data-occupant', 'player:local'); await capture('blackout-seated')
  56 |       const seat = await position(page)
  57 |       await walkTicks(page, 's', 12); expect(await position(page)).toEqual(seat)
  58 |       await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
  59 |       await page.getByRole('button', { name: 'Restore cockpit power' }).click(); await page.clock.runFor(100)
  60 |       await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
  61 |       await capture('restored-seated')
  62 |       await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
  63 |       await page.getByRole('button', { name: 'Test cockpit power loss' }).click()
  64 |       await page.getByRole('button', { name: 'Stand up', exact: true }).click()
  65 |       await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
  66 |       expect(await world!.evaluate(n => n.isConnected)).toBe(true); expect(errors).toEqual([])
  67 |       measurements.push({ replay, before, entered, moving, exited, continued, final: await ship(), backend, errors })
  68 |     } finally { await context.close() }
  69 |   }
  70 |   await info.attach('cockpit-measurements.json', { body: JSON.stringify(measurements, null, 2), contentType: 'application/json' })
  71 | })
  72 | 
```