# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: cockpit.spec.ts >> COCKPIT-1 panoramic round trip, reserved seat, blackout and persistent simulation
- Location: tests/cockpit.spec.ts:10:1

# Error details

```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  locator('main')
Expected: "radar"
Received: "world"
Timeout:  10000ms

Call log:
  - Expect "toHaveAttribute" locator('main') with timeout 10000ms
  - waiting for locator('main')
    23 × locator resolved to <main data-tick="9" data-target="" data-occupant="" data-powered="true" class="top-down-app" data-cockpit-view="world" data-player-mode="standing" data-ship="{"position":{"cell":["0","0","0"],"offsetM":[53962642442,-5390064239,-77946039081]},"velocityMps":[0,0,0],"yaw":-0.12,"pitch":0,"massKg":100000,"throttle":0,"alignment":null,"mode":"grounded","contact":{"bodyId":"vesper","outcome":"safe-contact","impactSpeedMps":0,"normal":[3.2258038501581854e-7,0.9999999999999349,-1.6129019250790927e-7],"ageTi…>…</main>
       - unexpected value "world"

```

```yaml
- main:
  - heading "Scout · Surface study" [level=1]
  - img "Strict overhead Scout and planetary test site"
  - complementary "Resource inventory"
  - complementary "Nearby interaction"
  - paragraph
```

# Test source

```ts
  1   | import { PNG } from 'pngjs'
  2   | import { test, expect } from '@playwright/test'
  3   | import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
  4   | import { boot, scene, walkTo, walkTicks, position } from './worldJourney'
  5   | 
  6   | // Original captures and raw state are explicit below; avoid synchronizing every
  7   | // software WebGL frame for a trace filmstrip. Failure action traces remain.
  8   | test.use({ trace: { mode: 'retain-on-failure', snapshots: false, screenshots: false, sources: true } })
  9   | 
  10  | test('COCKPIT-1 panoramic round trip, reserved seat, blackout and persistent simulation', async ({ browser }, info) => {
  11  |   test.setTimeout(360000)
  12  |   const originals: Record<string, Buffer> = {}, measurements = []
  13  |   for (let replay = 0; replay < 2; replay++) {
  14  |     const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
  15  |     const page = await context.newPage(), errors: string[] = []
  16  |     page.on('pageerror', e => errors.push(e.message))
  17  |     try {
  18  |       await boot(page, '?powerFixture=1')
  19  |       const main = page.locator('main'), world = await scene(page).elementHandle()
  20  |       const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
  21  |       const capture = async (name: string) => {
  22  |         await page.clock.runFor(await page.locator('.panoramic').count() ? 64 : 5000)
  23  |         const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
  24  |         if (replay === 0) originals[name] = png
  25  |         else {
  26  |           const r = compareImages(originals[name]!, png)
  27  |           await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: r.pass, changedPixels: r.changedPixels, rawChangedPixels: r.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
  28  |           await info.attach(`${name}-diff.png`, { body: r.diff, contentType: 'image/png' })
  29  |           expect(r.pass, name).toBe(true)
  30  |         }
  31  |       }
  32  |       for (const p of [[-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
  33  |       for (const p of [[0, 7], [0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
  34  |       const before = await ship()
  35  |       await page.keyboard.press('e')
  36  |       await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-occupant', 'player:local')
  37  |       await expect.poll(async () => { await page.clock.runFor(50); return page.getByLabel('Panoramic universe', { exact: true }).count() }).toBe(1)
  38  |       await expect(page.getByLabel('Panoramic universe', { exact: true })).toBeVisible()
  39  |       await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-position', '0,-3.95')
  40  |       await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
  41  |       const view = async () => JSON.parse((await page.locator('.panoramic').getAttribute('data-observation'))!)
  42  |       const enteredView = await view()
  43  |       expect(enteredView.fov).toBe(75)
  44  |       expect(await page.locator('canvas').count()).toBe(1)
  45  |       expect(await page.locator('.radar').count()).toBe(0)
  46  |       expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, width: 1280, height: 800 })
  47  |       const entered = await ship()
  48  |       expect(entered.position).toEqual(before.position); expect(entered.velocityMps).toEqual(before.velocityMps)
  49  |       const backend = await page.locator('canvas').evaluate(c => {
  50  |         const gl = (c as HTMLCanvasElement).getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!
  51  |         return { renderer: gl.getParameter(ext.UNMASKED_RENDERER_WEBGL), version: gl.getParameter(gl.VERSION) }
  52  |       })
  53  |       await capture('panoramic-horizon')
  54  |       const horizon = PNG.sync.read(originals['panoramic-horizon'])
  55  |       const rgb = (x: number, y: number) => [...horizon.data.subarray((y * horizon.width + x) * 4, (y * horizon.width + x) * 4 + 3)]
  56  |       expect(rgb(800, 200)).not.toEqual(rgb(800, 600)) // physical globe below, sky above
  57  |       expect(rgb(800, 200)).not.toEqual([255, 255, 255])
  58  |       await expect(page.getByLabel('Throttle', { exact: true })).toBeDisabled(); await page.clock.runFor(500)
  59  |       const moving = await ship()
  60  |       expect(moving.throttle).toBe(0); expect(moving.velocityMps).toEqual([0, 0, 0]); expect(moving.mode).toBe('grounded')
  61  |       await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
  62  |       await page.getByRole('button', { name: 'Vesper', exact: true }).click()
  63  |       await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeDisabled()
  64  |       await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
  65  |       await page.getByRole('button', { name: 'Vesper Moon', exact: true }).click()
  66  |       await page.getByRole('button', { name: 'Look at', exact: true }).click()
  67  |       for (let frame = 0; frame < 25; frame++) await page.clock.fastForward(50)
  68  |       await page.keyboard.press('Escape') // dismiss browser before cockpit exit
> 69  |       await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
      |                          ^ Error: expect(locator).toHaveAttribute(expected) failed
  70  |       await page.locator('canvas').focus()
  71  |       await page.mouse.move(640, 400); await page.mouse.wheel(0, -400)
  72  |       await expect.poll(async () => { await page.clock.runFor(32); return (await view()).fov }).toBeLessThan(75)
  73  |       await page.keyboard.down('d'); await page.clock.runFor(240); await page.keyboard.up('d')
  74  |       await page.keyboard.down('ArrowLeft'); await page.clock.runFor(240); await page.keyboard.up('ArrowLeft')
  75  |       await page.clock.runFor(100)
  76  |       const observedShip = await ship(), savedView = await view()
  77  |       expect(observedShip.yaw).toBe(before.yaw); expect(observedShip.position).toEqual(before.position)
  78  |       expect(savedView.quaternion).not.toEqual(enteredView.quaternion)
  79  |       expect(savedView.fov).toBeLessThan(75)
  80  |       await capture('external-body-observation')
  81  |       // Held camera input must be released on unmount.
  82  |       await page.keyboard.down('w')
  83  |       await page.getByRole('button', { name: 'Exit cockpit' }).click()
  84  |       const exited = await ship()
  85  |       expect(exited.position).toEqual(before.position); expect(exited.contact.bodyId).toBe('vesper')
  86  |       expect(exited.yaw).toBe(before.yaw); expect(exited.pitch).toBe(before.pitch)
  87  |       await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
  88  |       await capture('standing-return')
  89  |       const continued = await ship()
  90  |       expect(continued.throttle).toBe(0); expect(continued.tick).toBeGreaterThan(exited.tick); expect(continued.position).toEqual(exited.position)
  91  |       await page.keyboard.press('e'); await expect(page.getByLabel('Throttle', { exact: true })).toHaveValue('0')
  92  |       await page.clock.runFor(100)
  93  |       expect(await view()).toEqual(savedView)
  94  |       await page.keyboard.up('w')
  95  |       const tickBefore = (await ship()).tick
  96  |       // Preserve elapsed domain time with bounded frame gaps on software WebGL.
  97  |       for (let elapsed = 0; elapsed < 1000; elapsed += 100) await page.clock.fastForward(100)
  98  |       const ticks = (await ship()).tick - tickBefore
  99  |       expect(ticks).toBeGreaterThanOrEqual(59); expect(ticks).toBeLessThanOrEqual(61)
  100 |       await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
  101 |       await page.getByRole('button', { name: 'Caligo', exact: true }).click()
  102 |       await page.getByRole('button', { name: 'Look at', exact: true }).click()
  103 |       await page.evaluate(() => window.dispatchEvent(new Event('blur')))
  104 |       await page.clock.runFor(100)
  105 |       const blurredView = await view()
  106 |       // Preserve elapsed domain time with bounded frame gaps on software WebGL.
  107 |       for (let elapsed = 0; elapsed < 1000; elapsed += 100) await page.clock.fastForward(100)
  108 |       expect(await view()).toEqual(blurredView)
  109 |       await page.keyboard.press('Escape') // browser closes first
  110 |       const beforeBlackout = await view()
  111 |       await page.getByRole('button', { name: 'Test cockpit power loss' }).click()
  112 |       await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
  113 |       await expect(main).toHaveAttribute('data-occupant', 'player:local'); await capture('blackout-seated')
  114 |       const seat = await position(page)
  115 |       await walkTicks(page, 's', 12); expect(await position(page)).toEqual(seat)
  116 |       await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
  117 |       await page.getByRole('button', { name: 'Restore cockpit power' }).click(); await page.clock.runFor(100)
  118 |       await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
  119 |       await capture('restored-seated')
  120 |       await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
  121 |       await page.clock.runFor(100); expect(await view()).toEqual(beforeBlackout)
  122 |       await page.getByRole('button', { name: 'Reset view', exact: true }).click()
  123 |       await page.clock.runFor(100); expect(await view()).toEqual(enteredView)
  124 |       await page.keyboard.press('Escape')
  125 |       await expect(main).toHaveAttribute('data-player-mode', 'standing')
  126 |       await page.keyboard.press('e'); await page.clock.runFor(100)
  127 |       await page.getByRole('button', { name: 'Test cockpit power loss' }).click()
  128 |       await page.getByRole('button', { name: 'Stand up', exact: true }).click()
  129 |       await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
  130 |       expect(await world!.evaluate(n => n.isConnected)).toBe(true); expect(errors).toEqual([])
  131 |       measurements.push({ replay, before, entered, moving, exited, continued, final: await ship(), enteredView, savedView, beforeBlackout, ticks, backend, errors })
  132 |     } finally { await context.close() }
  133 |   }
  134 |   await info.attach('cockpit-measurements.json', { body: JSON.stringify(measurements, null, 2), contentType: 'application/json' })
  135 | })
  136 | 
```