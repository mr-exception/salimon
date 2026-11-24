import { test, expect, type Page } from '@playwright/test'
import { boot, positionIn, walkTicks } from './worldJourney'
import { compareImages } from '../scripts/visual-comparison.ts'
const source = 'source:landing:mineral:01'
const command = (page: Page) => page.locator('[data-command-status]')
async function choose(page: Page, id: string, action: string) {
  const p = await page.locator(`[data-entity="${id}"]`).evaluate(g => {
    const p = new DOMPoint().matrixTransform((g as SVGGraphicsElement).getScreenCTM()!)
    return { x: p.x, y: p.y }
  })
  await page.mouse.click(p.x, p.y, { button: 'right' })
  await page.getByRole('menuitem', { name: action }).click()
  await page.clock.runFor(64)
}
async function completed(page: Page) {
  await expect.poll(async () => {
    await page.clock.runFor(250)
    const status = await command(page).getAttribute('data-command-status')
    expect(status).not.toBe('failed'); expect(status).not.toBe('cancelled')
    return status
  }, { timeout: 60000, intervals: [10, 25] }).toBe('completed')
}
const saved = (page: Page) => page.evaluate(() => JSON.parse(localStorage.getItem('salimon:expedition:v1')!))
test('COMMAND-893 built remote mine → confirmed deposit → cockpit journey with original/repeat evidence', async ({ page }, info) => {
  const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  await boot(page, '', 'http://127.0.0.1:4175')
  await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  const initial = await positionIn(page, 'planet:fixture')
  const before = await page.screenshot({ path: info.outputPath('before-command.png') })
  await choose(page, source, 'Gather / Mine')
  await expect(command(page)).toHaveAttribute('data-command-status', 'approaching')
  await page.screenshot({ path: info.outputPath('approaching-source.png') })
  await completed(page)
  expect(await positionIn(page, 'planet:fixture')).not.toEqual(initial)
  await walkTicks(page, [], 60)
  await expect(page.locator('[data-inventory-weight]')).toHaveText('1 / 10 kg')
  const original = await page.screenshot({ path: info.outputPath('mining.png') })
  const repeat = await page.screenshot({ path: info.outputPath('mining-repeat.png') })
  const pixels = compareImages(original, repeat)
  expect(pixels.pass).toBe(true); expect(compareImages(before, original).pass).toBe(false)
  await info.attach('repeat-diff.png', { body: pixels.diff, contentType: 'image/png' })
  await choose(page, 'scout:storage', 'Use Resource Container')
  await expect(page.getByLabel('Resource inventory')).toHaveAttribute('data-gathering', '')
  await completed(page)
  const panel = page.getByRole('dialog', { name: 'Resource Container', exact: true })
  await expect(panel).toBeVisible()
  const beforeTransfer = await saved(page)
  expect(beforeTransfer.inventory.quantities.mineral).toBe(1000)
  await page.screenshot({ path: info.outputPath('approached-container.png') })
  await panel.getByRole('textbox', { name: 'Amount in grams' }).fill('333')
  await panel.getByRole('button', { name: 'Review deposit' }).click()
  expect((await saved(page)).inventory).toEqual(beforeTransfer.inventory)
  const confirm = panel.getByRole('button', { name: 'Confirm deposit' })
  await confirm.press('Enter'); await page.keyboard.press('Enter')
  const transferred = await saved(page)
  expect(transferred.inventory.quantities.mineral).toBe(667)
  expect(transferred.stores['scout:storage'].quantities.mineral).toBe(333)
  await page.keyboard.press('Escape'); await page.clock.runFor(32)
  await choose(page, 'scout:chair', 'Use cockpit'); await completed(page)
  await expect(page.locator('.panoramic')).toBeVisible()
  await page.screenshot({ path: info.outputPath('approached-cockpit.png') })
  expect(errors).toEqual([])
  await info.attach('journey-state.json', { body: JSON.stringify({ initial, beforeTransfer, transferred, repeatPixels: pixels.changedPixels, errors, route: 'built /', input: 'actual Playwright mouse and keyboard', viewport: '1280x800 DPR1', renderer: 'SVG CPU; cockpit ANGLE SwiftShader' }), contentType: 'application/json' })
})
test('COMMAND-893 built replacement, WASD, Escape, stow, pause, blur and reload prevent delayed mining', async ({ page }, info) => {
  await boot(page, '', 'http://127.0.0.1:4175')
  await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  const observations = []
  for (const interrupt of ['wasd', 'escape', 'replacement', 'stow', 'pause', 'blur', 'reload']) {
    await choose(page, source, 'Gather / Mine')
    await expect(command(page)).toHaveAttribute('data-command-status', 'approaching')
    if (interrupt === 'wasd') await walkTicks(page, 's', 1)
    else if (interrupt === 'escape') await page.keyboard.press('Escape')
    else if (interrupt === 'replacement') await choose(page, 'scout:chair', 'Inspect')
    else if (interrupt === 'stow') await page.getByRole('button', { name: 'Stow gathering tool' }).click()
    else if (interrupt === 'pause') await page.keyboard.press('p')
    else if (interrupt === 'blur') await page.evaluate(() => window.dispatchEvent(new Event('blur')))
    else { await page.reload(); await page.clock.runFor(100); await expect(command(page)).toHaveCount(0) }
    await page.clock.runFor(1000)
    await expect(page.getByLabel('Resource inventory')).toHaveAttribute('data-gathering', '')
    expect((await saved(page)).inventory.quantities.mineral ?? 0).toBe(0)
    if (interrupt !== 'reload') observations.push({ interrupt, status: await command(page).getAttribute('data-command-status') })
    if (interrupt === 'stow' || interrupt === 'reload') await page.getByRole('button', { name: 'Equip gathering tool' }).click()
    if (interrupt === 'pause') await page.keyboard.press('p')
  }
  await page.screenshot({ path: info.outputPath('interrupted-reloaded.png') })
  await info.attach('interruptions.json', { body: JSON.stringify(observations), contentType: 'application/json' })
})
test('COMMAND-893 built closed door rejects remote cockpit route; door command opens once', async ({ page }, info) => {
  await boot(page, '', 'http://127.0.0.1:4175')
  // First stand outside the door, so closing it isolates the cockpit route.
  const outside = await page.locator('[data-entity="ship:scout"]').evaluate(g => {
    const p = new DOMPoint(0, 7).matrixTransform((g as SVGGraphicsElement).getScreenCTM()!)
    return { x: p.x, y: p.y }
  })
  await page.mouse.click(outside.x, outside.y, { button: 'right' })
  await expect.poll(async () => { await page.clock.runFor(250); return page.locator('[data-movement-status]').getAttribute('data-movement-status') }, { timeout: 60000, intervals: [10, 25] }).toBe('arrived')
  await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
  await choose(page, 'scout:entry', 'Close door'); await completed(page)
  await expect(page.locator('[data-entity="scout:entry"]')).toHaveAttribute('data-door-state', 'closed')
  await choose(page, 'scout:chair', 'Use cockpit')
  await expect.poll(async () => { await page.clock.runFor(250); return command(page).getAttribute('data-command-status') }, { timeout: 60000 }).toBe('failed')
  await expect(page.locator('.panoramic')).toHaveCount(0)
  await expect(command(page)).toContainText(/blocked|reachable/i)
  await page.screenshot({ path: info.outputPath('closed-door-blocked.png') })
  await choose(page, 'scout:entry', 'Open door'); await completed(page)
  await expect(page.locator('[data-entity="scout:entry"]')).toHaveAttribute('data-door-state', 'open')
  await page.clock.runFor(1000)
  await expect(page.locator('[data-entity="scout:entry"]')).toHaveAttribute('data-door-state', 'open')
})
