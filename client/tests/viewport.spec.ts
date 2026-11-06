import { test, expect, type Page } from '@playwright/test'
import { boot, position, scene, walkTicks } from './worldJourney'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'

async function layout(page: Page) {
  return page.evaluate(() => {
    const box = (selector: string) => {
      const r = document.querySelector(selector)!.getBoundingClientRect()
      return { x: r.x, y: r.y, width: r.width, height: r.height }
    }
    const svg = document.querySelector<SVGSVGElement>('.top-down-scene')!
    const matrix = svg.getScreenCTM()!, view = svg.viewBox.baseVal
    return { viewport: { width: innerWidth, height: innerHeight }, dpr: devicePixelRatio,
      scroll: { width: document.documentElement.scrollWidth, height: document.documentElement.scrollHeight },
      world: box('.top-down-scene'), stage: box('.top-down-stage'),
      hud: ['.survival-hud', '.resource-hud', '.top-down-bottom'].map(box),
      projection: { x: matrix.a * view.x + matrix.e, y: matrix.d * view.y + matrix.f,
        width: view.width * matrix.a, height: view.height * matrix.d, scaleX: matrix.a, scaleY: matrix.d } }
  })
}
async function checkLayout(page: Page) {
  // ResizeObserver and React commit asynchronously after viewport changes.
  await expect.poll(async () => {
    await page.clock.runFor(32)
    const r = await layout(page)
    return Math.round(r.projection.width * r.dpr) === r.viewport.width * r.dpr &&
      Math.round(r.projection.height * r.dpr) === r.viewport.height * r.dpr
  }).toBe(true)
  const result = await layout(page), { width, height } = result.viewport
  expect(result.world).toEqual({ x: 0, y: 0, width, height })
  expect(result.stage).toEqual(result.world)
  for (const id of ['ship:scout', 'scout:floor', 'scout:desk', 'scout:entry']) {
    const transform = await page.locator(`[data-entity="${id}"]`).getAttribute('transform')
    expect(transform).toContain('rotate(0)')
  }
  // A ship-local bow-to-stern vector must project vertically in screen space.
  const bow = await page.locator('[data-entity="ship:scout"]').evaluate(el => {
    const m = (el as unknown as SVGGraphicsElement).getScreenCTM()!
    return { dx: m.c * -1, dy: m.d * -1 }
  })
  expect(Math.abs(bow.dx)).toBe(0); expect(bow.dy).toBeLessThan(0)
  expect(result.scroll).toEqual({ width, height })
  // SVG viewBox.baseVal is float32. Measure coverage on the device-pixel grid;
  // retain the unrounded projection in layout.json for numerical inspection.
  for (const key of ['x', 'y'] as const) expect(Math.round(result.projection[key] * result.dpr)).toBe(0)
  expect(Math.round(result.projection.width * result.dpr)).toBe(width * result.dpr)
  expect(Math.round(result.projection.height * result.dpr)).toBe(height * result.dpr)
  expect(result.projection.scaleX).toBeCloseTo(result.projection.scaleY, 8)
  for (const box of result.hud) {
    expect(box.x).toBeGreaterThanOrEqual(0); expect(box.y).toBeGreaterThanOrEqual(0)
    expect(box.x + box.width).toBeLessThanOrEqual(width)
    expect(box.y + box.height).toBeLessThanOrEqual(height)
  }
  for (let i = 0; i < result.hud.length; i++) for (const b of result.hud.slice(i + 1)) {
    const a = result.hud[i]
    expect(a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y).toBe(true)
  }
  // The followed character must remain visible through the space between HUDs.
  const player = page.locator('[data-entity="person:player"]')
  const point = await player.evaluate(el => { const r = el.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 } })
  expect(await page.evaluate(({ x, y }) => !!document.elementFromPoint(x, y)?.closest('.top-down-scene'), point)).toBe(true)
  return result
}

for (const viewport of [{ width: 1280, height: 720 }, { width: 960, height: 600 }]) {
  test(`VIEWPORT production ${viewport.width}x${viewport.height} HUD, input, repeat and negative comparison`, async ({ browser }, info) => {
    let original: Buffer | undefined, fresh: Buffer | undefined
    const records = []
    for (let repeat = 0; repeat < 2; repeat++) {
      const context = await browser.newContext({ viewport, deviceScaleFactor: 1, reducedMotion: 'reduce', locale: 'en-US', timezoneId: 'UTC' })
      const page = await context.newPage()
      const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
      try {
        await boot(page, '', 'http://127.0.0.1:4175')
        await expect(page.getByRole('button', { name: 'Equip gathering tool' })).toBeEnabled()
        records.push(await checkLayout(page))
        await page.evaluate(async () => {
          await Promise.all(Array.from(document.querySelectorAll('svg image')).map(async el => {
            const image = new Image(); image.src = el.getAttribute('href')!; await image.decode()
          }))
        })
        const freshCapture = await page.screenshot({ path: info.outputPath(`fresh-${repeat}.png`) })
        if (!fresh) fresh = freshCapture
        else {
          const comparison = compareImages(fresh, freshCapture)
          await info.attach('fresh-repeat.json', { body: JSON.stringify({ ...comparison, diff: undefined, policy: POLICY }), contentType: 'application/json' })
          expect(comparison.pass).toBe(true)
        }
        // A keyboard-focused HUD button retains its keys; clicking releases focus.
        const tool = page.getByRole('button', { name: 'Equip gathering tool' })
        await tool.focus(); const before = await position(page)
        await page.keyboard.down('w'); await page.clock.runFor(200); await page.keyboard.up('w')
        expect(await position(page)).toEqual(before)
        await tool.click(); await walkTicks(page, 'w', 10)
        expect((await position(page))[1]).toBeLessThan(before[1])
        await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
        await expect(page.getByLabel('Survival condition')).toContainText('Paused')
        await page.clock.runFor(100)
        records.push(await checkLayout(page))
        await page.evaluate(async () => {
          await Promise.all(Array.from(document.querySelectorAll('svg image')).map(async el => {
            const image = new Image(); image.src = el.getAttribute('href')!; await image.decode()
          }))
        })
        const png = await page.screenshot({ path: info.outputPath(`viewport-${repeat}.png`) })
        if (!original) original = png
        else {
          const result = compareImages(original, png)
          await info.attach('repeat.json', { body: JSON.stringify({ ...result, diff: undefined, policy: POLICY }), contentType: 'application/json' })
          expect(result.pass).toBe(true)
        }
        if (!repeat) {
          // Prove the capture comparator detects the old inset-world defect.
          await page.addStyleTag({ content: '.top-down-stage { inset:32px !important; border:8px solid red !important }' })
          await expect.poll(async () => (await scene(page).boundingBox())!.x).toBe(40)
          const broken = await page.screenshot({ path: info.outputPath('inset-negative.png') })
          const negative = compareImages(png, broken)
          await info.attach('negative.json', { body: JSON.stringify({ pass: negative.pass, changedPixels: negative.changedPixels, rawChangedPixels: negative.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
          expect(negative.pass).toBe(false)
        } else {
          for (const size of [{ width: 1100, height: 680 }, { width: 1600, height: 700 }]) {
            await page.setViewportSize(size)
            records.push(await checkLayout(page))
          }
          await page.getByRole('button', { name: 'Resume [P]' }).click()
          await walkTicks(page, 'w', 10)
        }
        expect(errors).toEqual([])
      } finally { await context.close() }
    }
    await info.attach('layout.json', { body: JSON.stringify({ records, renderer: 'Chromium SVG CPU rasterization', input: 'Playwright keyboard/mouse', baseline: 'same-run repeats; approved navigation references unchanged' }, null, 2), contentType: 'application/json' })
  })
}

test('VIEWPORT DPR 2 and standalone development fixture preserve projection', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 960, height: 600 }, deviceScaleFactor: 2 })
  const page = await context.newPage()
  await boot(page, '', 'http://127.0.0.1:4175')
  await checkLayout(page)
  expect(await page.evaluate(() => devicePixelRatio)).toBe(2)
  await page.goto('http://127.0.0.1:5175/?scene=top-down')
  await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
  expect(await scene(page).boundingBox()).toEqual({ x: 0, y: 0, width: 960, height: 600 })
  await expect(page.getByLabel('Nearby interaction')).toBeVisible()
  await context.close()
})
