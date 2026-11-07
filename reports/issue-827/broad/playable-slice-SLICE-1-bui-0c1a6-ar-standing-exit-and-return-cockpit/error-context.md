# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: playable-slice.spec.ts >> SLICE-1 built outdoor sources, doorway, cutaway, radar, standing exit and return
- Location: tests/playable-slice.spec.ts:14:1

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator:  getByText('Expedition saves automatically on this browser.')
Expected: visible
Received: hidden
Timeout:  10000ms

Call log:
  - Expect "toBeVisible" getByText('Expedition saves automatically on this browser.') with timeout 10000ms
  - waiting for getByText('Expedition saves automatically on this browser.')
    23 × locator resolved to <p class="resource-note">Expedition saves automatically on this browser.</p>
       - unexpected value "hidden"

```

```yaml
- complementary "Survival condition":
  - text: "Health: 200 / 200 HP"
  - meter "Health"
  - text: "Hunger: 97 / 100"
  - meter "Hunger"
  - text: "Fatigue: 100 / 100"
  - meter "Fatigue"
  - button "Pause [P]"
  - button "Help"
- main:
  - heading "Scout · Surface study" [level=1]
  - img "Strict overhead Scout and planetary test site"
  - complementary "Resource inventory":
    - button "Field inventory I"
    - paragraph:
      - strong: "0"
      - text: / 10 kg
    - paragraph:
      - strong: "0"
      - text: / 12 slots
    - button "Equip gathering tool"
  - complementary "Nearby interaction":
    - status: Saved expedition restored. Gathering is stopped.
  - paragraph:
    - strong: W A S D
    - text: Walk ·
    - strong: E
    - text: Interact ·
    - strong: Esc
    - text: Cancel ·
    - strong: P
    - text: Pause
```

# Test source

```ts
  44  |       await expect(page.locator('canvas')).toHaveCount(0)
  45  |       const capture = async (name: string) => {
  46  |         await page.clock.runFor(await page.locator('.panoramic').count() ? 64 : 5000)
  47  |         const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`), fullPage: true })
  48  |         const checkpoint = { ship: await shipState(), location: await main.getAttribute('data-scout-location'), name, position: await position(page), planet: await positionIn(page, 'planet:fixture'), frame: await frame(), posture: await main.getAttribute('data-player-mode'), cameraGroundTransform: await page.locator('[data-entity="terrain:site"]').getAttribute('transform') }
  49  |         checkpoints.push(checkpoint)
  50  |         await info.attach(`${name}-${replay}-state.json`, { body: JSON.stringify(checkpoint), contentType: 'application/json' })
  51  |         if (replay === 0) originals[name] = png
  52  |         else {
  53  |           const r = compareImages(originals[name]!, png)
  54  |           await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: r.pass, changedPixels: r.changedPixels, rawChangedPixels: r.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
  55  |           await info.attach(`${name}-diff.png`, { body: r.diff, contentType: 'image/png' })
  56  |           expect(r.pass, name).toBe(true)
  57  |         }
  58  |       }
  59  |       await capture('slice-outdoor-spawn')
  60  |       await page.keyboard.press('e'); await expect(page.getByRole('status')).toContainText('No available object')
  61  |       // Walk to both physical source footprints and use their shared interaction.
  62  |       await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 100)
  63  |       expect((await position(page))[0]).toBeCloseTo(-7.34, 6)
  64  |       await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-target', 'source:landing:mineral:01')
  65  |       await capture('slice-mineral-inspection')
  66  |       await page.keyboard.press('Escape')
  67  |       await walkTicks(page, 'd', 34); await walkTicks(page, 's', 110); await walkTicks(page, 'd', 25)
  68  |       await page.keyboard.press('e'); await expect(main).toHaveAttribute('data-target', 'source:landing:biology:01')
  69  |       await page.keyboard.press('Escape')
  70  |       // Exit the colony's reach to starboard, then approach the aft cargo entry.
  71  |       await walkTo(page, [-2, 4.4], 'planet:fixture', true)
  72  |       await walkTo(page, [-2, 7], 'planet:fixture', true)
  73  |       await walkTo(page, [0, 7], 'ship:scout', true)
  74  |       await walkTo(page, [0, 5.65], 'ship:scout', true)
  75  |       // Original exterior view contains player, exposed rooms and surface props.
  76  |       for (const id of ['scout:floor', 'scout:desk', 'scout:core', 'scout:storage', 'source:landing:biology:01']) {
  77  |         const box = (await page.locator(`[data-entity="${id}"]`).boundingBox())!, bounds = (await scene(page).boundingBox())!
  78  |         expect(box.x + box.width).toBeGreaterThan(bounds.x); expect(box.x).toBeLessThan(bounds.x + bounds.width)
  79  |         expect(box.y + box.height).toBeGreaterThan(bounds.y); expect(box.y).toBeLessThan(bounds.y + bounds.height)
  80  |       }
  81  |       await capture('slice-exterior-cutaway')
  82  |       const cross = async (key: string, finalFrame: string) => {
  83  |         const samples = []
  84  |         for (let tick = 0; tick < 14; tick++) {
  85  |           const before = await positionIn(page, 'planet:fixture'), oldFrame = await frame()
  86  |           const oldBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
  87  |           await walkTicks(page, key, 1)
  88  |           const after = await positionIn(page, 'planet:fixture'), newFrame = await frame()
  89  |           const newBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
  90  |           expect(Math.hypot(after[0] - before[0], after[1] - before[1])).toBeCloseTo(.04, 7)
  91  |           expect(Math.hypot(newBox.x - oldBox.x, newBox.y - oldBox.y)).toBeLessThan(3)
  92  |           samples.push({ before, after, oldFrame, newFrame, oldBox, newBox })
  93  |         }
  94  |         expect(samples.filter(s => s.oldFrame !== s.newFrame)).toHaveLength(1)
  95  |         expect(await frame()).toBe(finalFrame); handoffs.push(samples)
  96  |       }
  97  |       await cross('w', 'ship:scout')
  98  |       for (const p of [[0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
  99  |       await page.keyboard.press('e')
  100 |       await expect.poll(async () => { await page.clock.runFor(50); return page.getByLabel('Panoramic universe', { exact: true }).count() }).toBe(1)
  101 |       await expect(main).toHaveAttribute('data-player-mode', 'seated')
  102 |       await expect(page.getByLabel('Throttle', { exact: true })).toBeDisabled()
  103 |       await expect(page.getByText('grounded · Vesper', { exact: false })).toBeVisible()
  104 |       await expect(page.getByRole('button', { name: 'Restart navigation' })).toHaveCount(0)
  105 |       for (let elapsed = 0; elapsed < 5200; elapsed += 200) await page.clock.fastForward(200)
  106 |       await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
  107 |       await page.getByRole('button', { name: 'Vesper', exact: true }).click()
  108 |       await page.keyboard.press('Escape')
  109 |       await expect(page.getByRole('button', { name: 'Set direction' })).toBeDisabled()
  110 |       await page.keyboard.down('ArrowLeft'); await page.clock.runFor(200); await page.keyboard.up('ArrowLeft')
  111 |       await page.getByRole('button', { name: 'Reset view', exact: true }).click()
  112 |       const anchored = await assertGrounded()
  113 |       expect(anchored.tick).toBeGreaterThan(initial.tick)
  114 |       await capture('slice-grounded-panoramic')
  115 |       const backend = await page.locator('canvas').evaluate(c => {
  116 |         const gl = (c as HTMLCanvasElement).getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!
  117 |         return { renderer: gl.getParameter(ext.UNMASKED_RENDERER_WEBGL), version: gl.getParameter(gl.VERSION) }
  118 |       })
  119 |       await page.getByRole('button', { name: 'Exit cockpit' }).click()
  120 |       await expect(main).toHaveAttribute('data-player-mode', 'standing'); await expect(main).toHaveAttribute('data-occupant', '')
  121 |       await capture('slice-standing-return')
  122 |       await page.keyboard.press('e')
  123 |       await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
  124 |       for (let elapsed = 0; elapsed < 5200; elapsed += 200) await page.clock.fastForward(200)
  125 |       await assertGrounded()
  126 |       await page.getByRole('button', { name: 'Exit cockpit' }).click()
  127 |       await assertGrounded()
  128 |       for (const p of [[0, -1.65], [1.35, -1.65], [1.35, .45], [0, .45], [0, 2], [0, 5.3]]) await walkTo(page, p, 'ship:scout', true)
  129 |       await cross('s', 'planet:fixture')
  130 |       await walkTo(page, [0, 7], 'ship:scout', true)
  131 |       await walkTo(page, [-2, 7], 'planet:fixture', true)
  132 |       await walkTo(page, [-2, 3], 'planet:fixture', true)
  133 |       await walkTo(page, [-6, 3], 'planet:fixture', true)
  134 |       await capture('slice-returned-outdoors')
  135 |       expect(await sourceState()).toEqual(sources)
  136 |       expect(await node!.evaluate(n => n.isConnected)).toBe(true)
  137 |       expect(await frame()).toBe('planet:fixture'); expect(errors).toEqual([])
  138 |       const ship = JSON.parse((await main.getAttribute('data-ship'))!)
  139 |       await assertGrounded()
  140 |       const savedPosition = await position(page)
  141 |       await page.evaluate(() => window.dispatchEvent(new PageTransitionEvent('pagehide')))
  142 |       await page.reload()
  143 |       await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
> 144 |       await expect(page.getByText('Expedition saves automatically on this browser.')).toBeVisible()
      |                                                                                       ^ Error: expect(locator).toBeVisible() failed
  145 |       expect(await position(page)).toEqual(savedPosition)
  146 |       await assertGrounded()
  147 |       await capture('slice-reloaded-outdoors')
  148 |       journeys.push({ replay, sources, checkpoints, handoffs, ship, backend, errors, browser: browser.version(), input: 'actual keyboard/mouse via production adapters; read-only DOM observations', origin: 'built preview :4175', viewport: page.viewportSize(), dpr: await page.evaluate(() => devicePixelRatio) })
  149 |     } finally { await context.close() }
  150 |   }
  151 |   const wrong = compareImages(originals['slice-outdoor-spawn']!, originals['slice-mineral-inspection']!)
  152 |   expect(wrong.pass).toBe(false); expect(wrong.changedPixels).toBeGreaterThan(100)
  153 |   await info.attach('slice-journeys.json', { body: JSON.stringify({ journeys, wrongCheckpoint: { pass: wrong.pass, changedPixels: wrong.changedPixels, rawChangedPixels: wrong.rawChangedPixels }, clock: '2026-11-04T00:00:00Z fixed-step controlled browser clock', baseline: 'review candidates; no approved references changed', future: ['mining extraction/depletion/inventory persistence', 'Core energy', 'Fabricator production', 'multiplayer', 'ship damage'] }, null, 2), contentType: 'application/json' })
  154 | })
  155 | 
```