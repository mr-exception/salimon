# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: top-down.spec.ts >> TD-2 release/blur/hidden/resize/focus clear movement while camera settles smoothly
- Location: tests/top-down.spec.ts:73:1

# Error details

```
Test timeout of 30000ms exceeded.
```

```
Error: locator.getAttribute: Test timeout of 30000ms exceeded.
Call log:
  - waiting for locator('[data-entity="person:player"]')

```

# Test source

```ts
  1   | import { test, expect, type Page } from '@playwright/test'
  2   | import { PNG } from 'pngjs'
  3   | import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
  4   | import { convertAnchor, type FrameId } from '../src/topDownFrames.ts'
  5   | import { createTopDownState } from '../src/topDownRuntime.ts'
  6   | import { point2 } from '../src/topDownFrames.ts'
  7   | 
  8   | const scene = (page: Page) => page.getByRole('img', { name: 'Strict overhead Scout and planetary test site' })
  9   | const transform = (page: Page, id: string) => page.locator(`[data-entity="${id}"]`).getAttribute('transform')
  10  | async function boot(page: Page, view = 'interior') {
  11  |   await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') })
  12  |   await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  13  |   await page.goto(`/?scene=top-down&view=${view}`)
  14  |   // Lazy development module can resolve under React's delayed Suspense commit.
  15  |   await expect.poll(async () => { await page.clock.runFor(50); return scene(page).count() }).toBe(1)
  16  |   await expect(scene(page)).toBeVisible()
  17  |   await page.evaluate(() => document.fonts.ready)
  18  |   await page.clock.runFor(32)
  19  | }
  20  | 
  21  | test('TD-1/4 one scene, stable layers, visible interior and identical fresh-run baseline captures', async ({ page }, info) => {
  22  |   const errors: string[] = []; page.on('pageerror', error => errors.push(error.message))
  23  |   const captures: Buffer[] = [], observations = []
  24  |   for (let replay = 0; replay < 2; replay++) {
  25  |     await boot(page)
  26  |     await expect(page.locator('canvas')).toHaveCount(0)
  27  |     const layers = await scene(page).locator(':scope > g').evaluateAll(nodes => nodes.map(n => n.getAttribute('data-layer')))
  28  |     expect(layers).toEqual(['terrain', 'hull', 'interior', 'people', 'props', 'cues'])
  29  |     await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
  30  |     await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
  31  |     expect(await scene(page).locator('[data-entity]').count()).toBeGreaterThan(10)
  32  |     const original = await scene(page).screenshot({ path: info.outputPath(`top-down-baseline-${replay}.png`), animations: 'allow' })
  33  |     captures.push(original)
  34  |     const png = PNG.sync.read(original), counts = { floor: 0, player: 0, rocks: 0 }
  35  |     for (let i = 0; i < png.data.length; i += 4) {
  36  |       const [r, g, b] = png.data.subarray(i, i + 3)
  37  |       if (r === 223 && g === 217 && b === 198) counts.floor++
  38  |       if (r === 226 && g === 216 && b === 185) counts.player++
  39  |       if (r === 130 && g === 127 && b === 115) counts.rocks++
  40  |     }
  41  |     expect(counts.floor).toBeGreaterThan(1000); expect(counts.player).toBeGreaterThan(50); expect(counts.rocks).toBeGreaterThan(500)
  42  |     observations.push({ layers, counts, player: await transform(page, 'person:player'), viewport: page.viewportSize(), browser: page.context().browser()!.version(), dpr: await page.evaluate(() => devicePixelRatio) })
  43  |   }
  44  |   expect(captures[1]).toEqual(captures[0])
  45  |   expect(errors).toEqual([])
  46  |   await info.attach('top-down-baseline.png', { body: captures[0], contentType: 'image/png' })
  47  |   await info.attach('top-down-observations.json', { body: JSON.stringify({ observations, errors, clock: '2026-11-04T00:00:01Z; 32ms', input: 'Playwright synthetic', renderer: 'Chromium SVG', status: 'new deterministic capture; no approved pixel reference promoted' }, null, 2), contentType: 'application/json' })
  48  | })
  49  | 
> 50  | const position = async (page: Page) => (await page.locator('[data-entity="person:player"]').getAttribute('data-position'))!.split(',').map(Number)
      |                                                                                             ^ Error: locator.getAttribute: Test timeout of 30000ms exceeded.
  51  | async function walk(page: Page, keys: string[], ms: number) {
  52  |   for (const key of keys) await page.keyboard.down(key)
  53  |   await page.clock.runFor(ms)
  54  |   for (const key of keys) await page.keyboard.up(key)
  55  | }
  56  | // Steer through the real keyboard adapter; DOM attributes are observations only.
  57  | async function positionIn(page: Page, frame: FrameId) {
  58  |   const p = await position(page)
  59  |   const current = await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  60  |   return convertAnchor({ frame: current, point: point2(p[0], p[1]) }, frame, createTopDownState().frames).point.offsetM.slice(0, 2)
  61  | }
  62  | async function walkTo(page: Page, target: number[], frame?: FrameId) {
  63  |   frame ??= await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  64  |   for (let i = 0; i < 140; i++) {
  65  |     const p = await positionIn(page, frame), dx = target[0] - p[0], dy = target[1] - p[1]
  66  |     if (Math.hypot(dx, dy) < .075) return
  67  |     const angle = frame === 'ship:scout' ? -.12 : 0, x = Math.cos(angle) * dx - Math.sin(angle) * dy, y = Math.sin(angle) * dx + Math.cos(angle) * dy
  68  |     const keys = [...(Math.abs(x) > .035 ? [x > 0 ? 'd' : 'a'] : []), ...(Math.abs(y) > .035 ? [y > 0 ? 's' : 'w'] : [])]
  69  |     await walk(page, keys, Math.max(17, Math.min(65, Math.hypot(dx, dy) / 2.4 * 1000)))
  70  |   }
  71  |   throw new Error(`Could not walk to ${target}; stopped at ${await position(page)}`)
  72  | }
  73  | test('TD-2 release/blur/hidden/resize/focus clear movement while camera settles smoothly', async ({ page }, info) => {
  74  |   await boot(page)
  75  |   const before = await position(page), player = await transform(page, 'person:player')
  76  |   await walk(page, ['w'], 160)
  77  |   const walked = await position(page)
  78  |   expect(walked[1]).toBeLessThan(before[1]); expect(await transform(page, 'person:player')).not.toBe(player)
  79  |   const facing = await page.locator('[data-entity="person:player"]').getAttribute('data-facing')
  80  |   await page.clock.runFor(160); expect(await position(page)).toEqual(walked)
  81  |   expect(await page.locator('[data-entity="person:player"]').getAttribute('data-facing')).toBe(facing)
  82  |   for (const event of ['blur', 'resize', 'visibilitychange', 'focusin']) {
  83  |     await page.keyboard.down('w'); await page.clock.runFor(48)
  84  |     await page.evaluate(name => (name === 'visibilitychange' ? document : window).dispatchEvent(new Event(name)), event)
  85  |     const stopped = await position(page)
  86  |     await page.clock.runFor(160); expect(await position(page), event).toEqual(stopped)
  87  |     await page.keyboard.up('w')
  88  |   }
  89  |   await page.evaluate(() => { const input = document.createElement('input'); document.body.append(input); input.focus() })
  90  |   const focused = await position(page)
  91  |   await walk(page, ['d'], 160)
  92  |   await page.keyboard.press('e')
  93  |   expect(await position(page)).toEqual(focused)
  94  |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  95  |   await info.attach('walk.json', { body: JSON.stringify({ before, walked, facing }), contentType: 'application/json' })
  96  | })
  97  | 
  98  | test('TD-3 collisions, keyboard detour, selection and request/cancel use the continuous scene', async ({ page }, info) => {
  99  |   test.setTimeout(90000)
  100 |   await boot(page)
  101 |   await page.keyboard.press('e')
  102 |   await expect(page.getByRole('status')).toContainText('No available object in reach')
  103 |   await walkTo(page, [0, .45])
  104 |   await walk(page, ['w'], 700)
  105 |   expect((await position(page))[1]).toBeCloseTo(.36, 6)
  106 |   await scene(page).screenshot({ path: info.outputPath('crate-collision.png') })
  107 |   await walkTo(page, [1.35, .36])
  108 |   await expect(page.locator('[data-selection="scout:core"]')).toHaveCount(1)
  109 |   await expect(page.locator('[data-prompt]')).toContainText('Core')
  110 |   await page.keyboard.press('e')
  111 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'interacting')
  112 |   await expect(page.getByRole('status')).toContainText('No systems changed')
  113 |   await page.screenshot({ path: info.outputPath('core-inspection.png') })
  114 |   await page.keyboard.press('Escape')
  115 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  116 |   await page.keyboard.press('e')
  117 |   await page.evaluate(() => window.dispatchEvent(new Event('blur')))
  118 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  119 |   await walkTo(page, [1.35, -1.65])
  120 |   await walkTo(page, [0, -1.65])
  121 |   await walkTo(page, [0, -3.1])
  122 |   await expect(page.locator('[data-selection="scout:chair"]')).toHaveCount(1)
  123 |   await page.getByRole('button', { name: 'Inspect fixture' }).click()
  124 |   await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:chair')
  125 |   await page.getByRole('button', { name: 'Cancel inspection' }).click()
  126 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  127 |   await walkTo(page, [1.1, -3.1])
  128 |   await walkTo(page, [1.1, -3.9])
  129 |   await expect(page.locator('[data-selection="scout:desk"]')).toHaveCount(1)
  130 |   await page.keyboard.press('e')
  131 |   await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:desk')
  132 |   await page.screenshot({ path: info.outputPath('cockpit-reached.png') })
  133 |   expect((await position(page))[1]).toBeLessThan(-3)
  134 |   await info.attach('journey.json', { body: JSON.stringify({ final: await position(page), input: 'actual Playwright keyboard/mouse; no state setters', targets: ['scout:core', 'scout:desk'] }), contentType: 'application/json' })
  135 | })
  136 | 
  137 | test('TD-5 built entry retains navigation even with development scene query', async ({ page }, info) => {
  138 |   await page.goto('http://127.0.0.1:4175/?scene=top-down')
  139 |   await expect(page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })).toBeVisible()
  140 |   await expect(scene(page)).toHaveCount(0)
  141 |   await page.screenshot({ path: info.outputPath('radar-preflight.png') })
  142 |   const backend = await page.locator('canvas').evaluate(canvas => {
  143 |     const gl = (canvas as HTMLCanvasElement).getContext('webgl2')!
  144 |     const ext = gl.getExtension('WEBGL_debug_renderer_info')
  145 |     return { renderer: ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER), version: gl.getParameter(gl.VERSION) }
  146 |   })
  147 |   await info.attach('backend.json', { body: JSON.stringify(backend), contentType: 'application/json' })
  148 | })
  149 | 
  150 | 
```