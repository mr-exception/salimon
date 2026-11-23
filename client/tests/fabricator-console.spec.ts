import { test, expect } from '@playwright/test'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { MARKER_KIT_ID } from '../src/scoutFabricator.ts'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { boot } from './worldJourney'
import { origin, panel, saved, seed, transfer, fixture } from './fabricatorFixtures'

test.use({ trace: { mode: 'on', snapshots: false, screenshots: true, sources: true } })

for (const [width, height] of [[960, 600], [1280, 720], [1920, 1080]]) test(`FAB-UI-2 states, negatives and fixed-tolerance repeat captures ${width}`, async ({ page }, info) => {
  await page.setViewportSize({ width, height }); await seed(page, fixture('empty')); await boot(page, '', origin)
  const results: any[] = [], cssNegatives: any[] = [], captures: Record<string, Buffer> = {}
  const capture = async (name: string) => {
    const dialog = panel(page); await dialog.focus(); await page.mouse.move(0, 0); await dialog.evaluate(el => { el.scrollTop = 0 })
    const original = await dialog.screenshot({ animations: 'disabled', path: info.outputPath(`${name}.png`) })
    const repeat = await dialog.screenshot({ animations: 'disabled', path: info.outputPath(`${name}-repeat.png`) })
    const result = compareImages(original, repeat); expect(result.pass).toBe(true); captures[name] = original
    await info.attach(`${name}-diff.png`, { body: result.diff, contentType: 'image/png' })
    if (await dialog.evaluate(el => el.scrollHeight > el.clientHeight)) {
      await dialog.evaluate(el => { el.scrollTop = el.scrollHeight }); await dialog.screenshot({ path: info.outputPath(`${name}-controls.png`) })
    }
    results.push({ name, changedPixels: result.changedPixels, state: await saved(page) })
  }
  for (const state of ['empty', 'ready', 'powered', 'paused', 'complete', 'full'] as const) {
    if (state !== 'empty') { await page.evaluate(({ key, raw }) => localStorage.setItem(key, raw), { key: RESOURCE_SAVE_KEY, raw: fixture(state) }); await page.reload(); await page.clock.runFor(100) }
    await page.keyboard.press('e'); await expect(panel(page)).toBeFocused()
    const dialog = panel(page)
    await expect(page.getByRole('status', { name: 'Fabricator operating status' })).toContainText({ empty: 'Blocked', ready: 'Ready', powered: 'Powered', paused: 'Paused', complete: 'Complete', full: 'Complete' }[state])
    const box = (await dialog.boundingBox())!
    expect(box.x).toBeGreaterThanOrEqual(0); expect(box.y).toBeGreaterThanOrEqual(0)
    expect(box.x + box.width).toBeLessThanOrEqual(width); expect(box.y + box.height).toBeLessThanOrEqual(height)
    expect(await dialog.evaluate(el => el.scrollWidth > el.clientWidth)).toBe(false)
    expect(await dialog.evaluate(el => getComputedStyle(el).animationDuration)).toBe('0s')
    await page.keyboard.press('Shift+Tab'); await expect(dialog.getByRole('button', { name: 'Close Fabricator' })).toBeFocused()
    await page.keyboard.press('Tab'); await expect(dialog.getByRole('button', { name: 'Start fabrication' })).toBeFocused()
    await capture(state)
    if (state === 'paused') {
      await dialog.evaluate(el => { el.scrollTop = 0 })
      const defect = await page.addStyleTag({ content: '.resource-container .ui-button { background: #ff00ff !important; }' })
      const broken = await dialog.screenshot({ path: info.outputPath('negative-css.png'), animations: 'disabled' })
      const comparison = compareImages(captures.paused!, broken)
      expect(comparison.pass).toBe(false)
      cssNegatives.push({ pass: comparison.pass, changedPixels: comparison.changedPixels })
      await info.attach('negative-css-diff.png', { body: comparison.diff, contentType: 'image/png' })
      await defect.evaluate(el => el.parentNode?.removeChild(el))
    }
    if (state === 'full') {
      const before = await saved(page)
      await page.getByRole('button', { name: 'Claim output' }).click()
      await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('overweight')
      expect(await saved(page)).toEqual(before); await capture('claim-rejected')
      await transfer(page, 'deposit', 1000)
      await page.getByRole('button', { name: 'Claim output' }).click()
      expect((await saved(page)).inventory.items[MARKER_KIT_ID]).toBe(1)
      await page.getByRole('textbox', { name: 'Amount in grams' }).fill('1')
      await page.getByRole('button', { name: 'Review deposit' }).click(); await capture('pending')
      await page.evaluate(key => localStorage.setItem(key, localStorage.getItem(key)! + ' '), RESOURCE_SAVE_KEY)
      const bytes = await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
      await page.getByRole('button', { name: 'Confirm deposit' }).click()
      await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Saving failed')
      expect(await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)).toBe(bytes)
      await capture('save-error')
    }
    await page.keyboard.press('Escape'); await expect(panel(page)).toHaveCount(0)
  }
  const negative = compareImages(captures.empty!, captures.paused!); expect(negative.pass).toBe(false)
  await info.attach('comparisons.json', { body: JSON.stringify({ viewport: { width, height }, browser: page.context().browser()!.version(), policy: POLICY, results, cssNegatives, negative: { pass: negative.pass, changedPixels: negative.changedPixels }, baselineApproval: 'owner deployed-version visual review pending' }, null, 2), contentType: 'application/json' })
})


for (const [width, height] of [[960, 600], [1280, 720], [1920, 1080]]) test(`MARKER-879 confirmation, cancellation, stale save and repeat ${width}`, async ({ page }, info) => {
  await page.setViewportSize({ width, height }); await seed(page, fixture('marker')); await boot(page, '', origin)
  await page.keyboard.press('i')
  const deploy = page.getByRole('button', { name: 'Deploy field marker', exact: true })
  await deploy.click()
  const dialog = page.getByRole('dialog', { name: 'Deploy field marker', exact: true })
  await expect(dialog).toBeFocused()
  await page.keyboard.press('Tab'); await expect(dialog.getByRole('textbox', { name: 'Marker name' })).toBeFocused()
  await dialog.getByLabel('Marker name').fill('West supply')
  const box = (await dialog.boundingBox())!
  expect(box.x).toBeGreaterThanOrEqual(0); expect(box.y).toBeGreaterThanOrEqual(0)
  expect(box.x + box.width).toBeLessThanOrEqual(width); expect(box.y + box.height).toBeLessThanOrEqual(height)
  expect(await dialog.evaluate(el => el.scrollWidth > el.clientWidth)).toBe(false)
  await dialog.focus(); await page.mouse.move(0, 0)
  const original = await dialog.screenshot({ path: info.outputPath('confirmation.png'), animations: 'disabled' })
  const repeat = await dialog.screenshot({ path: info.outputPath('confirmation-repeat.png'), animations: 'disabled' })
  expect(compareImages(original, repeat).pass).toBe(true)
  const style = await page.addStyleTag({ content: '.ui-dialog--equipment { background: #ff00ff !important; }' })
  const broken = await dialog.screenshot({ path: info.outputPath('negative-css.png'), animations: 'disabled' })
  expect(compareImages(original, broken).pass).toBe(false)
  await style.evaluate(el => el.parentNode?.removeChild(el))
  const before = await saved(page)
  await dialog.getByRole('button', { name: 'Cancel deployment' }).click()
  expect((await saved(page)).inventory).toEqual(before.inventory)
  expect((await saved(page)).deployedMarkers).toEqual([])
  await expect(deploy).toBeFocused()
  await deploy.click()
  await page.evaluate(key => localStorage.setItem(key, localStorage.getItem(key)! + ' '), RESOURCE_SAVE_KEY)
  const bytes = await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
  await dialog.getByRole('button', { name: 'Confirm deployment' }).click()
  expect(await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)).toBe(bytes)
  expect((await saved(page)).deployedMarkers).toEqual([])
  expect((await saved(page)).inventory.items[MARKER_KIT_ID]).toBe(1)
  await expect(page.getByLabel('Resource inventory')).toContainText('Saving failed')
  await info.attach('marker-negative.json', { body: JSON.stringify({ width, before, after: await saved(page), repeat: compareImages(original, repeat).changedPixels, cssNegative: compareImages(original, broken).changedPixels, policy: POLICY }, null, 2), contentType: 'application/json' })
})
