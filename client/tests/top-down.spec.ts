import { test, expect, type Page } from '@playwright/test'
import { PNG } from 'pngjs'

const scene = (page: Page) => page.getByRole('img', { name: 'Strict overhead Scout and planetary test site' })
const transform = (page: Page, id: string) => page.locator(`[data-entity="${id}"]`).getAttribute('transform')
async function boot(page: Page) {
  await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') })
  await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  await page.goto('/?scene=top-down')
  // Lazy development module can resolve under React's delayed Suspense commit.
  await expect.poll(async () => { await page.clock.runFor(50); return scene(page).count() }).toBe(1)
  await expect(scene(page)).toBeVisible()
  await page.evaluate(() => document.fonts.ready)
  await page.clock.runFor(32)
}

test('TD-1/4 one scene, stable layers, visible interior and identical fresh-run baseline captures', async ({ page }, info) => {
  const errors: string[] = []; page.on('pageerror', error => errors.push(error.message))
  const captures: Buffer[] = [], observations = []
  for (let replay = 0; replay < 2; replay++) {
    await boot(page)
    await expect(page.locator('canvas')).toHaveCount(0)
    const layers = await scene(page).locator(':scope > g').evaluateAll(nodes => nodes.map(n => n.getAttribute('data-layer')))
    expect(layers).toEqual(['terrain', 'hull', 'interior', 'people', 'props', 'cues'])
    await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
    await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
    expect(await scene(page).locator('[data-entity]').count()).toBe(10)
    const original = await scene(page).screenshot({ path: info.outputPath(`top-down-baseline-${replay}.png`), animations: 'allow' })
    captures.push(original)
    const png = PNG.sync.read(original), counts = { floor: 0, player: 0, rocks: 0 }
    for (let i = 0; i < png.data.length; i += 4) {
      const [r, g, b] = png.data.subarray(i, i + 3)
      if (r === 223 && g === 217 && b === 198) counts.floor++
      if (r === 226 && g === 216 && b === 185) counts.player++
      if (r === 130 && g === 127 && b === 115) counts.rocks++
    }
    expect(counts.floor).toBeGreaterThan(1000); expect(counts.player).toBeGreaterThan(50); expect(counts.rocks).toBeGreaterThan(500)
    observations.push({ layers, counts, player: await transform(page, 'person:player'), viewport: page.viewportSize(), browser: page.context().browser()!.version(), dpr: await page.evaluate(() => devicePixelRatio) })
  }
  expect(captures[1]).toEqual(captures[0])
  expect(errors).toEqual([])
  await info.attach('top-down-baseline.png', { body: captures[0], contentType: 'image/png' })
  await info.attach('top-down-observations.json', { body: JSON.stringify({ observations, errors, clock: '2026-11-04T00:00:01Z; 32ms', input: 'Playwright synthetic', renderer: 'Chromium SVG', status: 'new deterministic capture; no approved pixel reference promoted' }, null, 2), contentType: 'application/json' })
})

test('TD-2 WASD moves world beneath centered player, release/blur/hidden/resize clear movement', async ({ page }, info) => {
  await boot(page)
  const player = await transform(page, 'person:player'), before = await transform(page, 'scout:floor')
  await page.keyboard.down('w'); await page.clock.runFor(160); await page.keyboard.up('w')
  const walked = await transform(page, 'scout:floor')
  expect(walked).not.toBe(before); expect(await transform(page, 'person:player')).toBe(player)
  await page.clock.runFor(160); expect(await transform(page, 'scout:floor')).toBe(walked)
  for (const event of ['blur', 'resize', 'visibilitychange', 'focusin']) {
    await page.keyboard.down('s'); await page.clock.runFor(48)
    await page.evaluate(name => (name === 'visibilitychange' ? document : window).dispatchEvent(new Event(name)), event)
    const stopped = await transform(page, 'scout:floor')
    await page.clock.runFor(160); expect(await transform(page, 'scout:floor'), event).toBe(stopped)
    await page.keyboard.up('s')
  }
  // Keyboard input in an editable control must not leak into movement.
  await page.evaluate(() => { const input = document.createElement('input'); document.body.append(input); input.focus() })
  const focused = await transform(page, 'scout:floor')
  await page.keyboard.down('d'); await page.clock.runFor(160); await page.keyboard.up('d')
  expect(await transform(page, 'scout:floor')).toBe(focused)
  await info.attach('walk.json', { body: JSON.stringify({ before, walked, player }), contentType: 'application/json' })
})

test('TD-5 built entry retains navigation even with development scene query', async ({ page }) => {
  await page.goto('http://127.0.0.1:4175/?scene=top-down')
  await expect(page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })).toBeVisible()
  await expect(scene(page)).toHaveCount(0)
})
