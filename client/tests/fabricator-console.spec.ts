import { test, expect, type Page } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { point2 } from '../src/topDownFrames.ts'
import { FABRICATOR_INPUT_ID as inputId, FABRICATOR_RECIPE_ID, MARKER_KIT_ID } from '../src/scoutFabricator.ts'
import { SCOUT_CORE_FUEL_ID as fuelId } from '../src/scoutEnergyCore.ts'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { boot, position, walkTicks, walkTo } from './worldJourney'

const origin = 'http://127.0.0.1:4175'
const panel = (page: Page, name = 'Fabricator') => page.getByRole('dialog', { name, exact: true })
const saved = (page: Page) => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
async function seed(page: Page, raw: string) {
  await page.addInitScript(({ key, raw }) => { if (!localStorage.getItem(key)) localStorage.setItem(key, raw) }, { key: RESOURCE_SAVE_KEY, raw })
}
async function transfer(page: Page, direction: 'deposit' | 'withdraw', grams: number) {
  await page.getByRole('textbox', { name: 'Amount in grams' }).fill(String(grams))
  await page.getByRole('button', { name: direction === 'deposit' ? 'Review deposit' : 'Review withdrawal' }).click()
  await page.getByRole('button', { name: `Confirm ${direction}` }).click()
}
function fixture(state: 'journey' | 'empty' | 'ready' | 'powered' | 'paused' | 'complete' | 'full') {
  const initial = createTopDownState('exterior', true)
  const world = state === 'journey' ? initial : { ...initial, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(-.7, 3.9) } } : e) }
  let raw: string | null = null
  const s = new ResourceSession({ getItem: () => raw, setItem: (_, v) => { raw = v } }, world, undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
  // A valid prior expedition spent its reserve. The journey itself gathers,
  // fuels, starts, exhausts power and resumes only through production controls.
  if (state === 'journey' || state === 'paused') s.advance({ ...s.world, tick: 1 }, { watts: s.snapshot.core.reserveJ60 - (state === 'journey' ? 5000 * 60 : 0) })
  else if (state === 'empty') {
    expect(s.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1 })).toBe(true)
  } else s.checkpoint()
  if (!['journey', 'empty'].includes(state)) {
    expect(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 2000 })).toBe(true)
    expect(s.depositFabricatorInput('mineral', 2000, s.snapshot.revision)).toBe(true)
    if (state !== 'ready') {
      expect(s.startFabricator(FABRICATOR_RECIPE_ID, s.snapshot.revision)).toBe(true)
      s.advance({ ...s.world, tick: s.world.tick + (state === 'complete' || state === 'full' ? 1200 : 1) })
    }
    if (state === 'full') expect(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 10000 })).toBe(true)
  }
  return raw!
}

test('FAB-UI-1 built mine, carry, ship buffer, fuel, start, power loss, resume, claim and reload', async ({ page }, info) => {
  const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  await seed(page, fixture('journey')); await boot(page, '', origin)
  await page.screenshot({ path: info.outputPath('production-start.png') })
  await expect(panel(page)).toHaveCount(0)
  await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
  await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  await page.keyboard.press('e'); await walkTicks(page, [], 120); await page.keyboard.press('Escape')
  await expect(page.locator('[data-resource="mineral"]')).toHaveText('2 kg')
  for (const p of [[-6, 3], [-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
  for (const p of [[0, 7], [0, 4.25], [.5, 4.25]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e'); await expect(panel(page, 'Resource Container')).toBeVisible()
  await transfer(page, 'deposit', 2000); await transfer(page, 'withdraw', 2000); await page.keyboard.press('Escape')
  for (const p of [[0, 3], [.8, 1.8]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e'); await expect(panel(page, 'Energy Core')).toBeVisible()
  await transfer(page, 'deposit', 1); await page.keyboard.press('Escape')
  for (const p of [[0, 3], [-.7, 3.9]]) await walkTo(page, p, 'ship:scout', true)
  await expect(page.locator('[data-prompt]')).toContainText('Fabricator · Use [E]')
  await page.getByRole('button', { name: 'Use Fabricator', exact: true }).click()
  await expect(panel(page)).toBeFocused()
  await expect(page.getByRole('button', { name: 'Start fabrication' })).toBeDisabled()
  await transfer(page, 'deposit', 1000)
  await page.getByRole('button', { name: 'Start fabrication' }).press('Enter')
  const started = await saved(page)
  expect(started.fabricator.started).toBe(1); expect(started.stores[inputId].quantities).toEqual({})
  await expect(page.getByRole('button', { name: 'Start fabrication' })).toBeDisabled()
  await page.clock.runFor(2000)
  await expect(page.getByRole('status', { name: 'Fabricator operating status' })).toContainText('Paused')
  const interrupted = await saved(page)
  await page.clock.runFor(1000); expect((await saved(page)).fabricator).toEqual(interrupted.fabricator)
  await page.screenshot({ path: info.outputPath('power-interruption.png') })
  await page.keyboard.press('Escape'); await page.keyboard.press('e')
  expect((await saved(page)).fabricator).toEqual(interrupted.fabricator)
  const input = page.getByRole('textbox', { name: 'Amount in grams' })
  for (const amount of ['0', '-1', '.5', '1e3', '10000']) {
    await input.fill(amount); await page.getByRole('button', { name: 'Review deposit' }).click()
    expect((await saved(page)).fabricator).toEqual(interrupted.fabricator)
  }
  await input.fill('1'); await page.getByRole('button', { name: 'Review withdrawal' }).click()
  await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('only 0 g')
  await input.fill('1'); await page.getByRole('button', { name: 'Review deposit' }).click()
  await page.keyboard.press('Escape')
  expect((await saved(page)).inventory).toEqual(interrupted.inventory)
  await page.keyboard.press('e'); await input.fill('1'); await page.getByRole('button', { name: 'Review deposit' }).click()
  await page.evaluate(() => window.dispatchEvent(new Event('blur')))
  await expect(panel(page)).toHaveCount(0); expect((await saved(page)).inventory).toEqual(interrupted.inventory)
  for (const p of [[0, 3], [.8, 1.8]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e'); await transfer(page, 'deposit', 200); await page.keyboard.press('Escape')
  for (const p of [[0, 3], [-.7, 3.9]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e')
  await page.clock.runFor(1000)
  const resumed = await saved(page)
  expect(resumed.fabricator.job.powerPaused).toBe(false)
  expect(resumed.fabricator.job.remainingTicks).toBeLessThan(interrupted.fabricator.job.remainingTicks)
  const before = await position(page); await input.fill('wasdep'); await page.keyboard.press('w'); await walkTicks(page, [], 5)
  expect(await position(page)).toEqual(before)
  await page.clock.runFor(21000)
  const complete = await saved(page)
  expect(complete.fabricator.tray.count).toBe(1); expect(complete.inventory.items[MARKER_KIT_ID]).toBeUndefined()
  await expect(page.getByRole('status', { name: 'Fabricator operating status' })).toContainText('Complete')
  await page.getByRole('button', { name: 'Claim output' }).click()
  const claimed = await saved(page)
  expect(claimed.fabricator.tray).toBeNull(); expect(claimed.inventory.items[MARKER_KIT_ID]).toBe(1)
  await expect(page.getByRole('button', { name: 'Claim output' })).toHaveCount(0)
  await page.keyboard.press('Escape'); await page.reload()
  await expect(page.getByRole('button', { name: 'Pause [P]', exact: true })).toBeVisible()
  const restored = await saved(page)
  expect(restored.fabricator).toEqual(claimed.fabricator); expect(restored.inventory).toEqual(claimed.inventory)
  await page.clock.runFor(50); await page.keyboard.press('e'); await expect(panel(page)).toBeVisible()
  const raw = restored.inventory.quantities.mineral + (restored.stores[fuelId].quantities.mineral ?? 0) + (restored.convertedGrams.mineral ?? 0)
  expect(raw + restored.inventory.items[MARKER_KIT_ID] * 1000 + restored.sources['source:landing:mineral:01'].remainingGrams).toBe(60000)
  expect(restored.core.spentJ60 - started.core.spentJ60).toBe(100000 * 60)
  expect(errors).toEqual([])
  await info.attach('journey.json', { body: JSON.stringify({ started, interrupted, resumed, complete, claimed, restored, errors, input: 'Production keyboard/mouse; valid prior low-reserve expedition fixture; SVG CPU rasterization' }, null, 2), contentType: 'application/json' })
})

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
