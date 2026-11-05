# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: cockpit.spec.ts >> COCKPIT-1 radar round trip, reserved seat, blackout and persistent simulation
- Location: tests/cockpit.spec.ts:5:1

# Error details

```
Error: expect(received).toBe(expected) // Object.is equality

Expected: 613
Received: 56
```

# Test source

```ts
  1  | import { expect, type Page } from '@playwright/test'
  2  | import { convertAnchor, point2, type FrameId } from '../src/topDownFrames.ts'
  3  | import { createTopDownState } from '../src/topDownRuntime.ts'
  4  | export const scene = (page: Page) => page.getByRole('img', { name: 'Strict overhead Scout and planetary test site' })
  5  | export const position = async (page: Page) => (await page.locator('[data-entity="person:player"]').getAttribute('data-position'))!.split(',').map(Number)
  6  | export async function boot(page: Page, view = '') {
  7  |   await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') })
  8  |   await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  9  |   await page.goto(`http://127.0.0.1:5175/${view}`)
  10 |   // Lazy development module can resolve under React's delayed Suspense commit.
  11 |   await expect.poll(async () => { await page.clock.runFor(50); return scene(page).count() }).toBe(1)
  12 |   await expect(scene(page)).toBeVisible()
  13 |   await page.evaluate(() => document.fonts.ready)
  14 |   await page.clock.runFor(32)
  15 | }
  16 | 
  17 | 
  18 | export async function walk(page: Page, keys: string[], ms: number) {
  19 |   for (const key of keys) await page.keyboard.down(key)
  20 |   await page.clock.runFor(ms)
  21 |   for (const key of keys) await page.keyboard.up(key)
  22 | }
  23 | // Steer through the real keyboard adapter; DOM attributes are observations only.
  24 | export async function positionIn(page: Page, frame: FrameId) {
  25 |   const p = await position(page)
  26 |   const current = await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  27 |   return convertAnchor({ frame: current, point: point2(p[0], p[1]) }, frame, createTopDownState().frames).point.offsetM.slice(0, 2)
  28 | }
  29 | export async function walkTo(page: Page, target: number[], frame?: FrameId, exactTicks = false) {
  30 |   frame ??= await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  31 |   for (let i = 0; i < 140; i++) {
  32 |     const p = await positionIn(page, frame), dx = target[0] - p[0], dy = target[1] - p[1]
  33 |     if (Math.hypot(dx, dy) < .075) return
  34 |     const angle = frame === 'ship:scout' ? -.12 : 0, x = Math.cos(angle) * dx - Math.sin(angle) * dy, y = Math.sin(angle) * dx + Math.cos(angle) * dy
  35 |     const keys = [...(Math.abs(x) > .035 ? [x > 0 ? 'd' : 'a'] : []), ...(Math.abs(y) > .035 ? [y > 0 ? 's' : 'w'] : [])]
  36 |     if (exactTicks) await walkTicks(page, keys, Math.max(1, Math.min(3, Math.floor(Math.hypot(dx, dy) / .04))))
  37 |     else await walk(page, keys, Math.max(17, Math.min(65, Math.hypot(dx, dy) / 2.4 * 1000)))
  38 |   }
  39 |   throw new Error(`Could not walk to ${target}; stopped at ${await position(page)}`)
  40 | }
  41 | 
  42 | export async function walkTicks(page: Page, key: string | string[], ticks: number) {
  43 |   const tick = async () => Number(await scene(page).getAttribute('data-tick'))
  44 |   const target = await tick() + ticks
  45 |   const keys = typeof key === 'string' ? [key] : key
  46 |   for (const code of keys) await page.keyboard.down(code)
  47 |   await page.clock.runFor(Math.max(0, ticks * 1000 / 60 - 25))
  48 |   // A 16 ms RAF interval advances at most one 60 Hz simulation tick.
  49 |   for (let i = 0; i < 60 && await tick() < target; i++) await page.clock.runFor(16)
  50 |   for (const code of keys) await page.keyboard.up(code)
> 51 |   expect(await tick()).toBe(target)
     |                        ^ Error: expect(received).toBe(expected) // Object.is equality
  52 | }
  53 | 
  54 | 
```