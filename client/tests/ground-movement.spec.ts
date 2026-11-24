import { test, expect, type Page } from '@playwright/test'
import { boot, positionIn, walkTicks } from './worldJourney'
import { compareImages } from '../scripts/visual-comparison.ts'

// Browser-dispatched mouse/keyboard events against built /; DOM is observation only.
const trajectories = new WeakMap<Page, unknown[]>()
const clickTargets = new WeakMap<Page, { x: number; y: number; pixelRadiusM: number }>()
async function rightClick(page: Page, x: number, y: number) {
  const screen = await page.locator('[data-entity="ship:scout"]').evaluate((g, p) => {
    const matrix = (g as SVGGraphicsElement).getScreenCTM()!
    const point = new DOMPoint(p[0], p[1]).matrixTransform(matrix)
    const x = Math.round(point.x), y = Math.round(point.y)
    const target = new DOMPoint(x, y).matrixTransform(matrix.inverse())
    return { x, y, target: { x: target.x, y: target.y, pixelRadiusM: Math.SQRT1_2 / Math.hypot(matrix.a, matrix.b) } }
  }, [x, y])
  clickTargets.set(page, screen.target)
  await page.mouse.click(screen.x, screen.y, { button: 'right' })
  await page.clock.runFor(32)
}
async function arrive(page: Page, target: number[]) {
  await expect.poll(async () => { await page.clock.runFor(250); const samples = trajectories.get(page) ?? []; samples.push({ tick: await page.locator('main').getAttribute('data-tick'), frame: await page.locator('[data-entity="person:player"]').getAttribute('data-frame'), shipPosition: await positionIn(page, 'ship:scout') }); trajectories.set(page, samples); return page.locator('[data-movement-status]').getAttribute('data-movement-status') }, { timeout: 60000, intervals: [10, 25, 50] }).toBe('arrived')
  const point = await positionIn(page, 'ship:scout')
  const clicked = clickTargets.get(page)!
  // Context-menu coordinates are integer CSS pixels. Assert exact arrival at
  // that pixel's physical point and bound its distance from the intended point.
  expect(Math.hypot(point[0]! - clicked.x, point[1]! - clicked.y)).toBeLessThan(1e-6)
  expect(Math.hypot(clicked.x - target[0]!, clicked.y - target[1]!)).toBeLessThanOrEqual(clicked.pixelRadiusM + 1e-6)
}
test('MOVE-890 built right-click travels around hull and crate, doorway and save; repeat pixels', async ({ browser }, info) => {
  const originals: Buffer[] = [], records = []
  for (let replay = 0; replay < 2; replay++) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, reducedMotion: 'reduce' })
    const page = await context.newPage(), errors: string[] = []
    page.on('pageerror', e => errors.push(e.message))
    try {
      await boot(page, '', 'http://127.0.0.1:4175')
      await rightClick(page, 0, 4.4); await page.screenshot({ path: info.outputPath(`moving-hull-${replay}.png`) }); await arrive(page, [0, 4.4])
      await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
      await rightClick(page, 0, -1.65); await arrive(page, [0, -1.65])
      await page.clock.runFor(5000)
      const png = await page.screenshot({ path: info.outputPath(`crate-arrival-${replay}.png`) })
      if (!replay) originals.push(png)
      else { const diff = compareImages(originals[0]!, png); expect(diff.pass).toBe(true); await info.attach('repeat.json', { body: JSON.stringify({ pass: diff.pass, changedPixels: diff.changedPixels, rawChangedPixels: diff.rawChangedPixels }), contentType: 'application/json' }); await info.attach('repeat-diff.png', { body: diff.diff, contentType: 'image/png' }) }
      // Object right-click reserves interaction; it neither moves nor activates.
      const before = await positionIn(page, 'ship:scout')
      await rightClick(page, 0, -3.95); await page.clock.runFor(500)
      expect(await positionIn(page, 'ship:scout')).toEqual(before)
      await expect(page.locator('.panoramic')).toHaveCount(0)
      await rightClick(page, 0, 4.4); await arrive(page, [0, 4.4])
      await page.keyboard.press('e'); await page.clock.runFor(32)
      await expect(page.locator('[data-entity="scout:entry"]')).toHaveAttribute('data-door-state', 'closed')
      await rightClick(page, 0, 7)
      await expect(page.locator('[data-movement-status]')).toHaveAttribute('data-movement-status', 'failed')
      await page.keyboard.press('e'); await page.clock.runFor(32)
      await rightClick(page, 0, 7); await arrive(page, [0, 7])
      await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
      await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
      const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('salimon:expedition:v1')!))
      expect(JSON.stringify(saved)).not.toMatch(/groundCommand|waypoints/)
      await page.reload(); await page.clock.runFor(100)
      await expect.poll(async () => { await page.clock.runFor(50); return page.locator('[data-entity="person:player"]').getAttribute('data-frame') }).toBe('planet:fixture')
      const restored = await positionIn(page, 'ship:scout')
      expect(Math.hypot(restored[0]!, restored[1]! - 7)).toBeLessThan(.05)
      await expect(page.locator('[data-movement-status]')).toHaveCount(0)
      expect(errors).toEqual([])
      records.push({ replay, saved, errors, trajectory: trajectories.get(page), position: await positionIn(page, 'ship:scout') })
    } finally { await context.close() }
  }
  await info.attach('movement-state.json', { body: JSON.stringify({ records, viewport: [1280, 800], dpr: 1, input: 'Playwright actual mouse and keyboard', renderer: 'SVG CPU rasterization', epoch: '2026-11-04T00:00:00Z' }), contentType: 'application/json' })
})
test('MOVE-890 built replacement, WASD, Escape, pause, focus loss, pointer loss, reload and viewport projection', async ({ page }, info) => {
  await boot(page, '', 'http://127.0.0.1:4175')
  for (const size of [{ width: 1280, height: 800 }, { width: 1600, height: 900 }, { width: 960, height: 720 }]) {
    await page.setViewportSize(size); await page.clock.runFor(64)
    await rightClick(page, -4, 6.8); await arrive(page, [-4, 6.8])
    await rightClick(page, -4, 4); await arrive(page, [-4, 4])
  }
  for (const interrupt of ['Escape', 'wasd', 'pause', 'blur', 'pointer', 'focus', 'reload']) {
    await rightClick(page, -4, 7)
    await expect(page.locator('[data-movement-status]')).toHaveAttribute('data-movement-status', 'moving')
    if (interrupt === 'Escape') await page.keyboard.press('Escape')
    if (interrupt === 'wasd') await walkTicks(page, 'a', 2)
    if (interrupt === 'pause') { await page.keyboard.press('p'); await page.keyboard.press('p') }
    if (interrupt === 'blur') await page.evaluate(() => window.dispatchEvent(new Event('blur')))
    if (interrupt === 'pointer') await page.mouse.move(20, 20)
    if (interrupt === 'focus') await page.getByRole('button', { name: /Field inventory/ }).focus()
    if (interrupt === 'reload') { await page.reload(); await page.clock.runFor(100) }
    await page.clock.runFor(64)
    const before = await positionIn(page, 'ship:scout'); await page.clock.runFor(1000)
    expect(await positionIn(page, 'ship:scout'), interrupt).toEqual(before)
  }
  await rightClick(page, -4, 7); await rightClick(page, -4, 4); await arrive(page, [-4, 4])
  await rightClick(page, 0, -.6)
  await expect(page.locator('[data-movement-status]')).toHaveAttribute('data-movement-status', 'failed')
  await page.screenshot({ path: info.outputPath('invalid-destination.png') })
})
