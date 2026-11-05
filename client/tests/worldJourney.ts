import { expect, type Page } from '@playwright/test'
import { convertAnchor, point2, type FrameId } from '../src/topDownFrames.ts'
import { createTopDownState } from '../src/topDownRuntime.ts'
export const scene = (page: Page) => page.getByRole('img', { name: 'Strict overhead Scout and planetary test site' })
export const position = async (page: Page) => (await page.locator('[data-entity="person:player"]').getAttribute('data-position'))!.split(',').map(Number)
export async function boot(page: Page, view = '', origin = 'http://127.0.0.1:5175') {
  await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') })
  await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  await page.goto(`${origin}/${view}`)
  // Advance the controlled clock while React commits the world adapter.
  await expect.poll(async () => { await page.clock.runFor(50); return scene(page).count() }).toBe(1)
  await expect(scene(page)).toBeVisible()
  await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
  await page.evaluate(() => document.fonts.ready)
  await page.clock.runFor(32)
}


export async function walk(page: Page, keys: string[], ms: number) {
  for (const key of keys) await page.keyboard.down(key)
  await page.clock.runFor(ms)
  for (const key of keys) await page.keyboard.up(key)
}
// Steer through the real keyboard adapter; DOM attributes are observations only.
export async function positionIn(page: Page, frame: FrameId) {
  const p = await position(page)
  const current = await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  return convertAnchor({ frame: current, point: point2(p[0], p[1]) }, frame, createTopDownState().frames).point.offsetM.slice(0, 2)
}
export async function walkTo(page: Page, target: number[], frame?: FrameId, exactTicks = false) {
  frame ??= await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  for (let i = 0; i < 140; i++) {
    const p = await positionIn(page, frame), dx = target[0] - p[0], dy = target[1] - p[1]
    if (Math.hypot(dx, dy) < .075) return
    const angle = frame === 'ship:scout' ? -.12 : 0, x = Math.cos(angle) * dx - Math.sin(angle) * dy, y = Math.sin(angle) * dx + Math.cos(angle) * dy
    const keys = [...(Math.abs(x) > .035 ? [x > 0 ? 'd' : 'a'] : []), ...(Math.abs(y) > .035 ? [y > 0 ? 's' : 'w'] : [])]
    if (exactTicks) await walkTicks(page, keys, Math.max(1, Math.min(3, Math.floor(Math.hypot(dx, dy) / .04))))
    else await walk(page, keys, Math.max(17, Math.min(65, Math.hypot(dx, dy) / 2.4 * 1000)))
  }
  throw new Error(`Could not walk to ${target}; stopped at ${await position(page)}`)
}

export async function walkTicks(page: Page, key: string | string[], ticks: number) {
  const tick = async () => Number(await page.locator('main').getAttribute('data-tick'))
  const target = await tick() + ticks
  const keys = typeof key === 'string' ? [key] : key
  for (const code of keys) await page.keyboard.down(code)
  await page.clock.runFor(Math.max(0, ticks * 1000 / 60 - 25))
  // A 16 ms RAF interval advances at most one 60 Hz simulation tick.
  for (let i = 0; i < 60 && await tick() < target; i++) await page.clock.runFor(16)
  for (const code of keys) await page.keyboard.up(code)
  expect(await tick()).toBe(target)
}
