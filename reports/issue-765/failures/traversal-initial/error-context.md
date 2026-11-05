# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: top-down.spec.ts >> TRAVERSE-1 outdoor spawn to cockpit and back preserves frames, camera and one visible world
- Location: tests/top-down.spec.ts:262:1

# Error details

```
Error: expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false
```

# Test source

```ts
  187 | async function walkTicks(page: Page, key: string, ticks: number) {
  188 |   const tick = async () => Number(await scene(page).getAttribute('data-tick'))
  189 |   const target = await tick() + ticks
  190 |   await page.keyboard.down(key)
  191 |   await page.clock.runFor(Math.max(0, ticks * 1000 / 60 - 25))
  192 |   for (let i = 0; i < 60 && await tick() < target; i++) await page.clock.runFor(1)
  193 |   await page.keyboard.up(key)
  194 |   expect(await tick()).toBe(target)
  195 | }
  196 | 
  197 | test('PLANET-1 fixed outdoor journey, source footprints and repeat screenshots', async ({ browser }, info) => {
  198 |   test.setTimeout(120000)
  199 |   const captures: Record<string, Buffer> = {}
  200 |   const sourceSnapshots: string[][] = []
  201 |   for (let replay = 0; replay < 2; replay++) {
  202 |     const context = await browser.newContext({ baseURL: 'http://127.0.0.1:5175', viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
  203 |     const page = await context.newPage()
  204 |     try {
  205 |       // Empty view exercises the default development launch, not a test-only spawn.
  206 |       await boot(page, '')
  207 |       await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
  208 |       expect(await position(page)).toEqual([-6, 3])
  209 |       const originalNode = await page.locator('[data-entity="ship:scout"]').elementHandle()
  210 |       const sources = await page.locator('[data-source-kind]').evaluateAll(nodes => nodes.map(n => `${n.getAttribute('data-entity')}/${n.getAttribute('data-source-kind')}/${n.getAttribute('data-frame')}/${n.getAttribute('data-position')}/${n.getAttribute('data-source-site')}`))
  211 |       sourceSnapshots.push(sources)
  212 |       expect(sources).toHaveLength(2)
  213 |       const capture = async (name: string) => {
  214 |         await page.clock.runFor(5000)
  215 |         const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
  216 |         await info.attach(`${name}-${replay}.json`, { body: JSON.stringify({ position: await position(page), tick: await scene(page).getAttribute('data-tick'), ground: await transform(page, 'terrain:site'), player: await transform(page, 'person:player'), viewport: page.viewportSize(), sceneBounds: await scene(page).boundingBox() }), contentType: 'application/json' })
  217 |         if (replay === 0) captures[name] = png
  218 |         else {
  219 |           const comparison = compareImages(captures[name]!, png)
  220 |           await info.attach(`${name}-comparison.json`, { body: JSON.stringify({ policy: POLICY, changedPixels: comparison.changedPixels, rawChangedPixels: comparison.rawChangedPixels, totalPixels: comparison.totalPixels }), contentType: 'application/json' })
  221 |           await info.attach(`${name}-diff.png`, { body: comparison.diff, contentType: 'image/png' })
  222 |           expect(comparison.pass, `${name} uses unchanged repository visual policy`).toBe(true)
  223 |         }
  224 |       }
  225 |       await capture('planet-spawn')
  226 |       await page.keyboard.press('e')
  227 |       await expect(page.getByRole('status')).toContainText('No available object in reach')
  228 |       await walkTicks(page, 'w', 75)
  229 |       await walkTicks(page, 'a', 100)
  230 |       expect((await position(page))[0]).toBeCloseTo(-7.34, 6)
  231 |       await expect(page.locator('[data-selection="source:landing:mineral:01"]')).toHaveCount(1)
  232 |       await page.keyboard.press('e')
  233 |       await expect(page.getByRole('status')).toContainText('Banded mineral outcrop')
  234 |       await capture('planet-near-rock')
  235 |       await page.keyboard.press('e')
  236 |       await page.keyboard.press('Escape')
  237 |       await walkTicks(page, 'd', 34)
  238 |       await walkTicks(page, 's', 110)
  239 |       await walkTicks(page, 'd', 25)
  240 |       await expect(page.locator('[data-selection="source:landing:biology:01"]')).toHaveCount(1)
  241 |       await page.keyboard.press('e')
  242 |       await expect(page.getByRole('status')).toContainText('Alien plate colony')
  243 |       await walkTicks(page, 's', 50)
  244 |       expect((await position(page))[1]).toBeCloseTo(4.69, 6)
  245 |       await capture('planet-near-colony')
  246 |       await walkTicks(page, 'd', 38)
  247 |       await walkTicks(page, 'w', 117)
  248 |       await walkTicks(page, 'd', 24)
  249 |       await capture('planet-near-scout')
  250 |       expect(await originalNode!.evaluate(node => node.isConnected)).toBe(true)
  251 |       expect(await page.locator('[data-source-kind]').evaluateAll(nodes => nodes.map(n => `${n.getAttribute('data-entity')}/${n.getAttribute('data-source-kind')}/${n.getAttribute('data-frame')}/${n.getAttribute('data-position')}/${n.getAttribute('data-source-site')}`))).toEqual(sources)
  252 |     } finally { await context.close() }
  253 |   }
  254 |   expect(sourceSnapshots[1]).toEqual(sourceSnapshots[0])
  255 |   const wrongView = compareImages(captures['planet-spawn']!, captures['planet-near-rock']!)
  256 |   expect(wrongView.pass).toBe(false)
  257 |   expect(wrongView.changedPixels).toBeGreaterThan(100)
  258 |   await info.attach('wrong-checkpoint-negative.json', { body: JSON.stringify({ policy: POLICY, pass: wrongView.pass, changedPixels: wrongView.changedPixels, rawChangedPixels: wrongView.rawChangedPixels }), contentType: 'application/json' })
  259 |   await info.attach('planet-sources.json', { body: JSON.stringify({ sources: sourceSnapshots, input: 'Playwright keyboard; no state setters', site: 'fixed fictional living-world sample v1', comparisons: Object.keys(captures), collection: 'not implemented; inspections only' }, null, 2), contentType: 'application/json' })
  260 | })
  261 | 
  262 | test('TRAVERSE-1 outdoor spawn to cockpit and back preserves frames, camera and one visible world', async ({ browser }, info) => {
  263 |   test.setTimeout(180000)
  264 |   const captures: Record<string, Buffer> = {}, journeys = []
  265 |   for (let replay = 0; replay < 2; replay++) {
  266 |     const context = await browser.newContext({ baseURL: 'http://127.0.0.1:5175', viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
  267 |     const page = await context.newPage()
  268 |     const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  269 |     try {
  270 |       await boot(page, '')
  271 |       const world = await scene(page).elementHandle()
  272 |       const ids = await scene(page).locator('[data-entity]').evaluateAll(nodes => nodes.map(n => n.getAttribute('data-entity')))
  273 |       const capture = async (name: string) => {
  274 |         await page.clock.runFor(5000)
  275 |         for (const id of ['scout:floor', 'scout:desk', 'scout:core', 'source:landing:biology:01']) {
  276 |           const box = await page.locator(`[data-entity="${id}"]`).boundingBox(), bounds = (await scene(page).boundingBox())!
  277 |           expect(box).not.toBeNull()
  278 |           expect(box!.x + box!.width).toBeGreaterThan(bounds.x); expect(box!.x).toBeLessThan(bounds.x + bounds.width)
  279 |           expect(box!.y + box!.height).toBeGreaterThan(bounds.y); expect(box!.y).toBeLessThan(bounds.y + bounds.height)
  280 |         }
  281 |         const png = await scene(page).screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
  282 |         if (replay === 0) captures[name] = png
  283 |         else {
  284 |           const result = compareImages(captures[name]!, png)
  285 |           await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: result.pass, changedPixels: result.changedPixels, rawChangedPixels: result.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
  286 |           await info.attach(`${name}-diff.png`, { body: result.diff, contentType: 'image/png' })
> 287 |           expect(result.pass).toBe(true)
      |                               ^ Error: expect(received).toBe(expected) // Object.is equality
  288 |         }
  289 |       }
  290 |       const frame = () => page.locator('[data-entity="person:player"]').getAttribute('data-frame')
  291 |       const transitions: unknown[][] = []
  292 |       const sampleCrossing = async (key: string, expected: string) => {
  293 |         const samples = []
  294 |         for (let i = 0; i < 14; i++) {
  295 |           const before = await positionIn(page, 'planet:fixture'), previous = await frame()
  296 |           const oldBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
  297 |           await walkTicks(page, key, 1)
  298 |           const after = await positionIn(page, 'planet:fixture'), current = await frame()
  299 |           const newBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
  300 |           expect(Math.hypot(after[0] - before[0], after[1] - before[1])).toBeCloseTo(.04, 7)
  301 |           expect(Math.hypot(newBox.x - oldBox.x, newBox.y - oldBox.y)).toBeLessThan(3)
  302 |           samples.push({ before, after, previous, current, oldBox, newBox })
  303 |         }
  304 |         expect(await frame()).toBe(expected)
  305 |         expect(samples.filter(s => s.previous !== s.current)).toHaveLength(1)
  306 |         transitions.push(samples)
  307 |       }
  308 |       await walkTo(page, [-2, 3], 'planet:fixture')
  309 |       await walkTo(page, [-2, 7], 'planet:fixture')
  310 |       await walkTo(page, [0, 7], 'ship:scout')
  311 |       // Walk into an aft jamb. The real hull sweep blocks entry.
  312 |       await walkTo(page, [2, 7], 'ship:scout')
  313 |       await walk(page, ['w'], 700)
  314 |       expect(await frame()).toBe('planet:fixture')
  315 |       expect((await positionIn(page, 'ship:scout'))[1]).toBeGreaterThan(6)
  316 |       await walkTo(page, [0, 7], 'ship:scout')
  317 |       await walkTo(page, [0, 5.65], 'ship:scout')
  318 |       await capture('door-outside')
  319 |       await sampleCrossing('w', 'ship:scout')
  320 |       await capture('door-inside')
  321 |       await walkTo(page, [0, 2], 'ship:scout')
  322 |       for (const target of [[0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, target, 'ship:scout')
  323 |       await expect(page.locator('[data-selection="scout:chair"]')).toHaveCount(1)
  324 |       await page.keyboard.press('e')
  325 |       await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:chair')
  326 |       await page.keyboard.press('Escape')
  327 |       await capture('cockpit-round-trip')
  328 |       for (const target of [[0, -1.65], [1.35, -1.65], [1.35, .45], [0, .45], [0, 2], [0, 5.3]]) await walkTo(page, target, 'ship:scout')
  329 |       await sampleCrossing('s', 'planet:fixture')
  330 |       await walkTo(page, [0, 7], 'ship:scout')
  331 |       await capture('returned-outdoors')
  332 |       expect(await world!.evaluate(node => node.isConnected)).toBe(true)
  333 |       expect(await scene(page).locator('[data-entity]').evaluateAll(nodes => nodes.map(n => n.getAttribute('data-entity')))).toEqual(ids)
  334 |       expect(errors).toEqual([])
  335 |       journeys.push({ replay, transitions, final: await position(page), frame: await frame(), ids, errors, browser: context.browser()!.version(), input: 'Playwright keyboard through production adapter', renderer: 'Chromium SVG CPU rasterization', viewport: page.viewportSize(), dpr: await page.evaluate(() => devicePixelRatio) })
  336 |     } finally { await context.close() }
  337 |   }
  338 |   const negative = compareImages(captures['door-outside']!, captures['cockpit-round-trip']!)
  339 |   expect(negative.pass).toBe(false)
  340 |   await info.attach('traversal-journeys.json', { body: JSON.stringify({ journeys, wrongCheckpoint: { pass: negative.pass, changedPixels: negative.changedPixels, rawChangedPixels: negative.rawChangedPixels }, clock: '2026-11-04T00:00:01Z fixed steps', baseline: 'repeat evidence only; no approved references changed' }, null, 2), contentType: 'application/json' })
  341 | })
  342 | 
```