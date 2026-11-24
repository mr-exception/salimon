import { test, expect, type Page } from '@playwright/test'
import { boot, positionIn, scene, walkTicks } from './worldJourney'
import { terrainArtSave } from './terrainArtFixtures'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { compareImages } from '../scripts/visual-comparison.ts'

const avatar = (page: Page) => page.locator('[data-entity="person:player"]')
async function screenPoint(page: Page, x: number, y: number) {
  return page.locator('[data-entity="ship:scout"]').evaluate((g, p) => {
    const matrix = (g as SVGGraphicsElement).getScreenCTM()!
    const screen = new DOMPoint(p[0], p[1]).matrixTransform(matrix)
    const rounded = { x: Math.round(screen.x), y: Math.round(screen.y) }
    const clicked = new DOMPoint(rounded.x, rounded.y).matrixTransform(matrix.inverse())
    return { ...rounded, target: [clicked.x, clicked.y] }
  }, [x, y])
}
async function pan(page: Page, dx = 130, dy = 60) {
  const box = (await scene(page).boundingBox())!
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down({ button: 'middle' })
  await page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, { steps: 4 })
  await page.mouse.up({ button: 'middle' })
  await page.clock.runFor(32)
}
async function wheel(page: Page, delta: number, expected?: number) {
  const box = (await scene(page).boundingBox())!
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  const previous = await scene(page).getAttribute('data-camera-scale')
  await page.mouse.wheel(0, delta)
  await expect.poll(async () => { await page.clock.runFor(32); return scene(page).getAttribute('data-camera-scale') }).not.toBe(previous)
  if (expected) {
    // Each wheel packet is bounded to avoid an enormous device delta snapping
    // the whole zoom range. Continue actual wheel input until the bound holds.
    for (let i = 0; i < 3 && Number(await scene(page).getAttribute('data-camera-scale')) !== expected; i++) {
      await page.mouse.wheel(0, delta); await page.clock.runFor(64)
    }
    await expect(scene(page)).toHaveAttribute('data-camera-scale', String(expected))
  }
}
async function lag(page: Page) {
  return avatar(page).evaluate(g => {
    const m = (g as SVGGraphicsElement).transform.baseVal.consolidate()!.matrix
    return Math.hypot(m.e, m.f) / Math.hypot(m.a, m.b)
  })
}
async function moveTo(page: Page, x: number, y: number, samples: unknown[]) {
  const screen = await screenPoint(page, x, y)
  await page.mouse.click(screen.x, screen.y, { button: 'right' })
  await expect(scene(page)).toHaveAttribute('data-camera-panned', 'false')
  await expect.poll(async () => {
    await page.clock.runFor(250)
    const sample = { position: await positionIn(page, 'ship:scout'), frame: await avatar(page).getAttribute('data-frame'), lag: await lag(page) }
    samples.push(sample)
    expect(sample.lag).toBeLessThan(.23)
    const status = await page.locator('[data-movement-status]').getAttribute('data-movement-status')
    expect(status).not.toBe('failed')
    return status
  }, { timeout: 60000, intervals: [10, 25] }).toBe('arrived')
  const actual = await positionIn(page, 'ship:scout')
  expect(Math.hypot(actual[0]! - screen.target[0]!, actual[1]! - screen.target[1]!)).toBeLessThan(1e-6)
}

test('CAMERA-891 built zoom/pan/Space, transformed routes and smooth doorway follow at two DPRs', async ({ browser }, info) => {
  const records = []
  for (const dpr of [1, 2]) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: dpr, reducedMotion: 'reduce' })
    const page = await context.newPage(), samples: unknown[] = [], errors: string[] = []
    page.on('pageerror', e => errors.push(e.message))
    try {
      await boot(page, '', 'http://127.0.0.1:4175')
      const start = await positionIn(page, 'ship:scout')
      const original = await page.screenshot({ path: info.outputPath(`centered-dpr${dpr}.png`) })
      await pan(page)
      const held = await avatar(page).getAttribute('transform')
      await page.clock.runFor(3000)
      expect(await avatar(page).getAttribute('transform')).toBe(held)
      expect(await positionIn(page, 'ship:scout')).toEqual(start)
      const panned = await page.screenshot({ path: info.outputPath(`panned-dpr${dpr}.png`) })
      expect(compareImages(original, panned).pass).toBe(false) // negative: missing pan cannot pass
      const heldLag = await lag(page)
      await page.keyboard.press('Space')
      await expect(scene(page)).toHaveAttribute('data-camera-panned', 'false')
      expect(await lag(page)).toBe(heldLag) // command does not snap
      await page.clock.runFor(32)
      expect(await lag(page)).toBeGreaterThan(0)
      expect(await lag(page)).toBeLessThan(heldLag)
      await page.clock.runFor(3000)
      expect(await lag(page)).toBeLessThan(1e-6)
      const recentered = await page.screenshot({ path: info.outputPath(`recentered-dpr${dpr}.png`) })
      const repeat = compareImages(original, recentered)
      expect(repeat.pass).toBe(true)
      await info.attach(`repeat-dpr${dpr}.json`, { body: JSON.stringify({ pass: repeat.pass, changedPixels: repeat.changedPixels, rawChangedPixels: repeat.rawChangedPixels }), contentType: 'application/json' })
      await pan(page, 80, 0)
      await walkTicks(page, 'a', 2)
      await expect(scene(page)).toHaveAttribute('data-camera-panned', 'false')
      await page.clock.runFor(3000)
      await wheel(page, -10000, 64)
      await pan(page, 65, -25)
      await page.setViewportSize({ width: 1100, height: 760 }); await page.clock.runFor(100)
      // A route command restores follow without corrupting the picked endpoint.
      // Let the first command settle before checking the normal movement lag.
      const target = await screenPoint(page, -4, 7)
      await page.mouse.click(target.x, target.y, { button: 'right' }); await page.clock.runFor(1000)
      await moveTo(page, 0, 4.4, samples)
      await expect(avatar(page)).toHaveAttribute('data-frame', 'ship:scout')
      await moveTo(page, 0, -1.65, samples) // around interior crate
      await wheel(page, 10000, 14)
      await moveTo(page, 0, 7, samples)
      await expect(avatar(page)).toHaveAttribute('data-frame', 'planet:fixture')
      await pan(page, 40, 0)
      await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
      const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('salimon:expedition:v1')!))
      expect(JSON.stringify(saved)).not.toMatch(/pixelsPerM|panned|camera/)
      const beforeReload = await positionIn(page, 'ship:scout')
      await page.reload(); await page.clock.runFor(200)
      await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-camera-scale') }).toBe('26')
      await expect(scene(page)).toHaveAttribute('data-camera-panned', 'false')
      const restored = await positionIn(page, 'ship:scout')
      expect(Math.hypot(restored[0]! - beforeReload[0]!, restored[1]! - beforeReload[1]!)).toBeLessThan(1e-6)
      expect(errors).toEqual([])
      records.push({ dpr, samples, saved, heldLag, errors })
    } finally { await context.close() }
  }
  await info.attach('camera-state.json', { body: JSON.stringify({ renderer: 'SVG CPU', input: 'Playwright mouse/keyboard; pagehide dispatched for save', records }), contentType: 'application/json' })
})

test('CAMERA-891 built capture release/cancel/blur/focus and HUD/dialog priority', async ({ page }, info) => {
  await boot(page, '', 'http://127.0.0.1:4175')
  const start = await positionIn(page, 'ship:scout'), records = []
  for (const terminal of ['release-outside', 'pointercancel', 'blur', 'focus']) {
    await page.mouse.move(600, 400)
    await page.mouse.down({ button: 'middle' })
    await page.mouse.move(650, 420)
    expect(await scene(page).evaluate(svg => svg.hasPointerCapture(1))).toBe(true)
    if (terminal === 'release-outside') { await page.mouse.move(2, 2); await page.mouse.up({ button: 'middle' }) }
    if (terminal === 'pointercancel') await scene(page).dispatchEvent('pointercancel', { pointerId: 1 })
    if (terminal === 'blur') await page.evaluate(() => window.dispatchEvent(new Event('blur')))
    if (terminal === 'focus') await page.getByRole('button', { name: /Field inventory/ }).focus()
    expect(await scene(page).evaluate(svg => svg.hasPointerCapture(1))).toBe(false)
    await page.clock.runFor(32)
    const held = await avatar(page).getAttribute('transform')
    await page.mouse.move(700, 450); await page.clock.runFor(250)
    expect(await avatar(page).getAttribute('transform')).toBe(held)
    await page.mouse.up({ button: 'middle' })
    records.push({ terminal, held })
  }
  const scale = await scene(page).getAttribute('data-camera-scale')
  const inventory = page.getByRole('button', { name: /Field inventory/ })
  await inventory.hover(); await page.mouse.wheel(0, -500); await page.clock.runFor(64)
  expect(await scene(page).getAttribute('data-camera-scale')).toBe(scale)
  await inventory.focus(); await page.keyboard.press('Space'); await page.clock.runFor(32)
  await expect(inventory).toHaveAttribute('aria-expanded', 'true')
  await expect(scene(page)).toHaveAttribute('data-camera-panned', 'true')
  await page.keyboard.press('Escape')
  await page.getByRole('button', { name: 'Help', exact: true }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.keyboard.press('Space')
  await expect(scene(page)).toHaveAttribute('data-camera-panned', 'true')
  await page.screenshot({ path: info.outputPath('dialog-priority.png') })
  await page.keyboard.press('Escape')
  await scene(page).focus()
  await page.keyboard.press('m')
  const name = page.getByLabel('Name (optional)', { exact: true })
  await name.fill('camera')
  await name.press('Space')
  await name.press('KeyD')
  await expect(name).toHaveValue('camera d')
  await expect(scene(page)).toHaveAttribute('data-camera-panned', 'true')
  await page.keyboard.press('Escape')
  expect(await positionIn(page, 'ship:scout')).toEqual(start)
  await info.attach('cancel-state.json', { body: JSON.stringify({ records, start, end: await positionIn(page, 'ship:scout'), note: 'pointercancel and window blur are dispatched lifecycle negatives; mouse drag/release/focus use browser input' }), contentType: 'application/json' })
})


test('CAMERA-891 built generated terrain zoom/pan and walking chart handoff', async ({ page }, info) => {
  const raw = terrainArtSave('caligo', [1, 0, 0], [254, 0])
  await page.addInitScript(({ key, raw }) => { if (!localStorage.getItem(key)) localStorage.setItem(key, raw) }, { key: RESOURCE_SAVE_KEY, raw })
  await boot(page, '', 'http://127.0.0.1:4175')
  const ground = page.locator('[data-generated-ground]')
  await expect(ground).toBeVisible()
  const before = await avatar(page).getAttribute('data-surface-chart')
  await wheel(page, 10000, 14)
  await pan(page, 150, 40)
  await page.screenshot({ path: info.outputPath('generated-panned.png') })
  await page.keyboard.press('Space'); await page.clock.runFor(3000)
  const samples = []
  for (let i = 0; i < 6; i++) {
    await walkTicks(page, 'd', 15)
    const sample = { chart: await avatar(page).getAttribute('data-surface-chart'), lag: await lag(page), position: await avatar(page).getAttribute('data-position') }
    expect(sample.lag).toBeLessThan(.23)
    samples.push(sample)
  }
  expect(await avatar(page).getAttribute('data-surface-chart')).not.toBe(before)
  await expect(ground).toHaveAttribute('data-art-samples', '1681')
  await page.screenshot({ path: info.outputPath('generated-follow.png') })
  await info.attach('generated-camera.json', { body: JSON.stringify({ samples, before, renderer: 'Chromium SVG/Canvas CPU', input: 'actual wheel/drag/Space/WASD', stream: await ground.getAttribute('data-stream') }), contentType: 'application/json' })
})
