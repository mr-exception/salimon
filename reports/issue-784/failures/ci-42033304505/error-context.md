# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: top-down.spec.ts >> TD-1/4 one scene, stable layers, visible interior and identical fresh-run baseline captures
- Location: tests/top-down.spec.ts:21:1

# Error details

```
Error: expect(received).toEqual(expected) // deep equality

- Expected  - 279
+ Received  + 279

@@ -120672,11 +120672,11 @@
      238,
      188,
      212,
      211,
      157,
-     17,
+     21,
      136,
      75,
      0,
      166,
      152,
@@ -120730,340 +120730,340 @@
      222,
      238,
      140,
      84,
      232,
-     95,
-     103,
-     104,
+     239,
+     55,
+     180,
+     5,
+     248,
+     100,
+     18,
+     157,
+     65,
      11,
-     240,
-     201,
-     36,
-     58,
-     131,
-     22,
-     168,
-     38,
-     81,
-     4,
-     241,
-     133,
-     137,
-     169,
-     65,
-     200,
-     20,
-     35,
+     84,
      147,
-     140,
-     66,
-     156,
-     104,
-     69,
-     137,
-     43,
-     198,
-     85,
+     40,
+     130,
+     248,
+     194,
+     196,
+     212,
+     32,
+     100,
+     138,
+     145,
+     73,
+     70,
      33,
-     26,
+     78,
+     180,
+     162,
+     196,
+     21,
+     227,
+     170,
+     16,
+     141,
+     194,
+     52,
+     95,
+     28,
      133,
-     105,
-     190,
-     56,
-     10,
-     249,
+     252,
+     84,
+     9,
+     130,
+     32,
+     8,
+     130,
+     32,
+     8,
+     98,
+     126,
+     24,
+     186,
+     146,
+     237,
+     237,
+     147,
+     245,
+     124,
+     247,
+     141,
+     87,
+     238,
+     60,
+     117,
+     110,
+     68,
+     32,
+     58,
+     29,
+     169,
+     185,
+     214,
+     80,
+     76,
+     237,
+     78,
+     23,
+     143,
+     136,
+     238,
      169,
-     18,
-     4,
-     65,
-     16,
-     4,
-     65,
-     16,
+     81,
+     52,
+     195,
+     243,
+     224,
+     203,
      196,
-     252,
-     48,
-     116,
-     37,
-     219,
-     219,
-     39,
+     141,
+     19,
+     215,
+     143,
+     43,
+     40,
+     212,
+     154,
+     125,
+     113,
+     32,
+     218,
+     8,
+     186,
+     145,
+     234,
+     27,
+     9,
+     130,
+     32,
+     8,
+     130,
+     32,
+     8,
+     98,
+     158,
+     60,
+     254,
+     198,
      203,
+     239,
+     248,
+     197,
+     33,
      217,
-     252,
+     149,
+     52,
      77,
-     87,
-     94,
-     118,
-     234,
-     252,
+     215,
+     31,
+     125,
+     245,
+     238,
+     225,
+     177,
+     162,
+     23,
+     132,
+     2,
+     209,
+     185,
      136,
-     64,
+     226,
+     68,
+     59,
+     12,
+     81,
+     104,
+     129,
      116,
-     58,
-     82,
-     115,
-     173,
-     161,
+     100,
+     154,
+     48,
+     78,
+     158,
+     88,
      152,
-     218,
-     157,
-     46,
-     30,
-     17,
-     221,
-     83,
-     163,
-     104,
-     134,
-     231,
-     193,
-     151,
-     137,
-     27,
-     39,
-     174,
-     31,
-     87,
-     80,
-     168,
      53,
-     251,
-     226,
-     64,
-     180,
-     17,
-     116,
-     35,
-     213,
+     198,
+     24,
      55,
-     18,
-     4,
-     65,
-     16,
-     4,
-     65,
-     16,
-     196,
-     60,
-     121,
-     252,
-     141,
-     151,
-     223,
-     241,
-     139,
-     67,
+     193,
+     189,
+     86,
+     99,
+     97,
+     194,
+     113,
+     117,
+     210,
+     243,
+     9,
+     173,
+     103,
      178,
      47,
-     27,
-     166,
-     101,
-     62,
-     250,
-     234,
-     203,
-     134,
-     70,
-     139,
-     94,
-     16,
-     10,
+     142,
      68,
-     231,
-     34,
-     138,
-     19,
-     237,
-     48,
-     68,
-     161,
-     5,
-     210,
-     145,
-     105,
-     194,
-     56,
      121,
-     98,
-     97,
-     214,
-     24,
-     99,
-     220,
-     4,
-     247,
-     90,
-     141,
-     133,
-     9,
-     199,
-     213,
-     73,
-     207,
-     39,
-     180,
-     158,
-     201,
-     190,
-     56,
-     18,
-     229,
-     169,
-     18,
+     170,
      4,
      65,
      16,
      4,
      65,
      16,
-     196,
-     60,
-     201,
-     101,
+     4,
+     49,
+     79,
+     114,
+     25,
+     253,
+     182,
+     71,
+     95,
+     241,
+     240,
+     177,
+     179,
+     7,
+     15,
+     159,
+     148,
+     115,
+     125,
+     235,
+     6,
+     135,
+     198,
+     220,
+     48,
+     126,
+     236,
+     163,
+     175,
+     233,
+     183,
+     84,
+     85,
+     22,
+     61,
+     47,
      244,
-     219,
-     30,
-     125,
-     197,
      195,
-     199,
-     206,
-     30,
-     60,
+     72,
+     32,
+     58,
+     14,
+     94,
+     127,
+     216,
      124,
-     82,
-     206,
-     245,
-     173,
-     59,
-     127,
-     97,
-     180,
-     230,
+     203,
+     210,
+     141,
+     115,
+     36,
+     150,
+     182,
+     42,
+     241,
+     244,
+     84,
+     73,
+     72,
+     43,
+     27,
+     197,
      6,
+     77,
+     151,
+     166,
+     18,
      55,
-     63,
-     246,
-     218,
-     62,
-     75,
+     169,
+     74,
+     33,
+     142,
+     5,
+     97,
+     46,
      85,
-     101,
-     209,
-     243,
-     66,
-     63,
-     140,
-     4,
-     162,
-     227,
-     224,
-     245,
-     135,
-     205,
-     183,
-     44,
-     221,
-     56,
+     217,
+     120,
+     11,
+     11,
+     163,
+     201,
+     23,
      71,
-     98,
-     105,
-     171,
-     18,
-     79,
-     79,
-     149,
-     132,
-     180,
-     178,
-     81,
-     108,
-     208,
-     116,
-     105,
-     42,
-     113,
-     147,
-     170,
-     20,
-     226,
-     88,
-     16,
-     230,
-     82,
-     149,
-     141,
-     183,
-     176,
-     48,
-     154,
-     124,
-     113,
-     68,
-     242,
-     197,
-     33,
+     36,
+     95,
+     28,
+     130,
+     32,
      8,
      130,
      32,
      8,
      130,
-     32,
-     136,
-     249,
-     144,
-     49,
-     212,
-     173,
-     235,
-     187,
+     152,
+     15,
+     25,
+     67,
+     221,
+     186,
+     190,
+     123,
+     223,
+     142,
+     141,
+     59,
      247,
-     237,
-     216,
-     184,
-     115,
-     207,
-     174,
-     219,
-     239,
-     124,
-     248,
-     224,
-     35,
-     39,
-     225,
-     198,
-     255,
-     7,
+     236,
+     186,
+     253,
+     206,
+     135,
+     15,
+     62,
+     114,
+     18,
+     110,
+     252,
+     127,
+     0,
      0,
      0,
      255,
      255,
-     89,
-     45,
-     90,
-     217,
+     95,
+     91,
+     123,
+     123,
      0,
      0,
      0,
      6,
      73,
      68,
      65,
      84,
      3,
      0,
-     231,
-     254,
+     75,
+     125,
      109,
-     210,
-     250,
-     30,
-     234,
-     39,
+     216,
+     254,
+     5,
+     0,
+     93,
      0,
      0,
      0,
      0,
      73,
```

# Page snapshot

```yaml
- main [ref=f1e3]:
  - generic [ref=f1e4]:
    - generic [ref=f1e5]:
      - paragraph [ref=f1e6]: SALIMON / WORLD FOUNDATION
      - heading "Scout · Surface study" [level=1] [ref=f1e7]
    - generic [ref=f1e8]: STRICT OVERHEAD / 2D
  - generic [ref=f1e9]:
    - img "Strict overhead Scout and planetary test site" [ref=f1e10]
    - generic:
      - generic: 01 / QUIET LANDING SITE
      - generic: SCOUT — INTERIOR CUTAWAY
  - complementary "Nearby interaction" [ref=f1e94]:
    - generic [ref=f1e95]:
      - paragraph [ref=f1e96]: No fixture in reach · Move closer
      - status [ref=f1e97]: Approach a fixture to inspect it. Systems are inactive.
    - button "Inspect fixture" [disabled] [ref=f1e98]
  - generic [ref=f1e99]:
    - paragraph [ref=f1e100]:
      - strong [ref=f1e101]: W A S D
      - text: Walk ·
      - strong [ref=f1e102]: E
      - text: Interact ·
      - strong [ref=f1e103]: Esc
      - text: Cancel
    - paragraph [ref=f1e104]: Console inspections are a test · Systems inactive
```

# Test source

```ts
  1   | import { test, expect, type Page } from '@playwright/test'
  2   | import { PNG } from 'pngjs'
  3   | import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
  4   | import { convertAnchor, point2, type FrameId } from '../src/topDownFrames.ts'
  5   | import { createTopDownState } from '../src/topDownRuntime.ts'
  6   |
  7   | const scene = (page: Page) => page.getByRole('img', { name: 'Strict overhead Scout and planetary test site' })
  8   | const transform = (page: Page, id: string) => page.locator(`[data-entity="${id}"]`).getAttribute('transform')
  9   | async function boot(page: Page, view = 'interior') {
  10  |   await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') })
  11  |   await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  12  |   await page.goto(`/?scene=top-down&view=${view}`)
  13  |   // Lazy development module can resolve under React's delayed Suspense commit.
  14  |   await expect.poll(async () => { await page.clock.runFor(50); return scene(page).count() }).toBe(1)
  15  |   await expect(scene(page)).toBeVisible()
  16  |   await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
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
  32  |     const box = (await scene(page).boundingBox())!
  33  |     for (const value of [box.x, box.y, box.width, box.height]) expect(value).toBe(Math.round(value))
  34  |     const original = await scene(page).screenshot({ path: info.outputPath(`top-down-baseline-${replay}.png`), animations: 'allow' })
  35  |     captures.push(original)
  36  |     const png = PNG.sync.read(original), counts = { floor: 0, player: 0, rocks: 0 }
  37  |     for (let i = 0; i < png.data.length; i += 4) {
  38  |       const [r, g, b] = png.data.subarray(i, i + 3)
  39  |       if (r === 223 && g === 217 && b === 198) counts.floor++
  40  |       if (r === 226 && g === 216 && b === 185) counts.player++
  41  |       if (r === 130 && g === 127 && b === 115) counts.rocks++
  42  |     }
  43  |     expect(counts.floor).toBeGreaterThan(1000); expect(counts.player).toBeGreaterThan(50); expect(counts.rocks).toBeGreaterThan(500)
  44  |     observations.push({ layers, counts, player: await transform(page, 'person:player'), viewport: page.viewportSize(), browser: page.context().browser()!.version(), dpr: await page.evaluate(() => devicePixelRatio) })
  45  |   }
> 46  |   expect(captures[1]).toEqual(captures[0])
      |                       ^ Error: expect(received).toEqual(expected) // deep equality
  47  |   expect(errors).toEqual([])
  48  |   await info.attach('top-down-baseline.png', { body: captures[0], contentType: 'image/png' })
  49  |   await info.attach('top-down-observations.json', { body: JSON.stringify({ observations, errors, clock: '2026-11-04T00:00:01Z; 32ms', input: 'Playwright synthetic', renderer: 'Chromium SVG', status: 'new deterministic capture; no approved pixel reference promoted' }, null, 2), contentType: 'application/json' })
  50  | })
  51  |
  52  | const position = async (page: Page) => (await page.locator('[data-entity="person:player"]').getAttribute('data-position'))!.split(',').map(Number)
  53  | async function walk(page: Page, keys: string[], ms: number) {
  54  |   for (const key of keys) await page.keyboard.down(key)
  55  |   await page.clock.runFor(ms)
  56  |   for (const key of keys) await page.keyboard.up(key)
  57  | }
  58  | // Steer through the real keyboard adapter; DOM attributes are observations only.
  59  | async function positionIn(page: Page, frame: FrameId) {
  60  |   const p = await position(page)
  61  |   const current = await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  62  |   return convertAnchor({ frame: current, point: point2(p[0], p[1]) }, frame, createTopDownState().frames).point.offsetM.slice(0, 2)
  63  | }
  64  | async function walkTo(page: Page, target: number[], frame?: FrameId, exactTicks = false) {
  65  |   frame ??= await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  66  |   for (let i = 0; i < 140; i++) {
  67  |     const p = await positionIn(page, frame), dx = target[0] - p[0], dy = target[1] - p[1]
  68  |     if (Math.hypot(dx, dy) < .075) return
  69  |     const angle = frame === 'ship:scout' ? -.12 : 0, x = Math.cos(angle) * dx - Math.sin(angle) * dy, y = Math.sin(angle) * dx + Math.cos(angle) * dy
  70  |     const keys = [...(Math.abs(x) > .035 ? [x > 0 ? 'd' : 'a'] : []), ...(Math.abs(y) > .035 ? [y > 0 ? 's' : 'w'] : [])]
  71  |     if (exactTicks) await walkTicks(page, keys, Math.max(1, Math.min(3, Math.floor(Math.hypot(dx, dy) / .04))))
  72  |     else await walk(page, keys, Math.max(17, Math.min(65, Math.hypot(dx, dy) / 2.4 * 1000)))
  73  |   }
  74  |   throw new Error(`Could not walk to ${target}; stopped at ${await position(page)}`)
  75  | }
  76  | test('TD-2 release/blur/hidden/resize/focus clear movement while camera settles smoothly', async ({ page }, info) => {
  77  |   await boot(page)
  78  |   const before = await position(page), player = await transform(page, 'person:player')
  79  |   await walk(page, ['w'], 160)
  80  |   const walked = await position(page)
  81  |   expect(walked[1]).toBeLessThan(before[1]); expect(await transform(page, 'person:player')).not.toBe(player)
  82  |   const facing = await page.locator('[data-entity="person:player"]').getAttribute('data-facing')
  83  |   await page.clock.runFor(160); expect(await position(page)).toEqual(walked)
  84  |   expect(await page.locator('[data-entity="person:player"]').getAttribute('data-facing')).toBe(facing)
  85  |   for (const event of ['blur', 'resize', 'visibilitychange', 'focusin']) {
  86  |     await page.keyboard.down('w'); await page.clock.runFor(48)
  87  |     await page.evaluate(name => (name === 'visibilitychange' ? document : window).dispatchEvent(new Event(name)), event)
  88  |     const stopped = await position(page)
  89  |     await page.clock.runFor(160); expect(await position(page), event).toEqual(stopped)
  90  |     await page.keyboard.up('w')
  91  |   }
  92  |   await page.evaluate(() => { const input = document.createElement('input'); document.body.append(input); input.focus() })
  93  |   const focused = await position(page)
  94  |   await walk(page, ['d'], 160)
  95  |   await page.keyboard.press('e')
  96  |   expect(await position(page)).toEqual(focused)
  97  |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  98  |   await info.attach('walk.json', { body: JSON.stringify({ before, walked, facing }), contentType: 'application/json' })
  99  | })
  100 |
  101 | test('TD-3 collisions, keyboard detour, selection and request/cancel use the continuous scene', async ({ page }, info) => {
  102 |   test.setTimeout(90000)
  103 |   await boot(page)
  104 |   await page.keyboard.press('e')
  105 |   await expect(page.getByRole('status')).toContainText('No available object in reach')
  106 |   await walkTo(page, [0, .45])
  107 |   await walk(page, ['w'], 700)
  108 |   expect((await position(page))[1]).toBeCloseTo(.36, 6)
  109 |   await scene(page).screenshot({ path: info.outputPath('crate-collision.png') })
  110 |   await walkTo(page, [1.35, .36])
  111 |   await expect(page.locator('[data-selection="scout:core"]')).toHaveCount(1)
  112 |   await expect(page.locator('[data-prompt]')).toContainText('Core')
  113 |   await page.keyboard.press('e')
  114 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'interacting')
  115 |   await expect(page.getByRole('status')).toContainText('No systems changed')
  116 |   await page.screenshot({ path: info.outputPath('core-inspection.png') })
  117 |   await page.keyboard.press('Escape')
  118 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  119 |   await page.keyboard.press('e')
  120 |   await page.evaluate(() => window.dispatchEvent(new Event('blur')))
  121 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  122 |   await walkTo(page, [1.35, -1.65])
  123 |   await walkTo(page, [0, -1.65])
  124 |   await walkTo(page, [0, -3.1])
  125 |   await expect(page.locator('[data-selection="scout:chair"]')).toHaveCount(1)
  126 |   await page.getByRole('button', { name: 'Inspect fixture' }).click()
  127 |   await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:chair')
  128 |   await page.getByRole('button', { name: 'Cancel inspection' }).click()
  129 |   await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  130 |   await walkTo(page, [1.1, -3.1])
  131 |   await walkTo(page, [1.1, -3.9])
  132 |   await expect(page.locator('[data-selection="scout:desk"]')).toHaveCount(1)
  133 |   await page.keyboard.press('e')
  134 |   await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:desk')
  135 |   await page.screenshot({ path: info.outputPath('cockpit-reached.png') })
  136 |   expect((await position(page))[1]).toBeLessThan(-3)
  137 |   await info.attach('journey.json', { body: JSON.stringify({ final: await position(page), input: 'actual Playwright keyboard/mouse; no state setters', targets: ['scout:core', 'scout:desk'] }), contentType: 'application/json' })
  138 | })
  139 |
  140 | test('TD-5 explicit built radar entry retains standalone navigation', async ({ page }, info) => {
  141 |   await page.goto('http://127.0.0.1:4175/?scene=radar')
  142 |   await expect(page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })).toBeVisible()
  143 |   await expect(scene(page)).toHaveCount(0)
  144 |   await page.screenshot({ path: info.outputPath('radar-preflight.png') })
  145 |   const backend = await page.locator('canvas').evaluate(canvas => {
  146 |     const gl = (canvas as HTMLCanvasElement).getContext('webgl2')!
```
