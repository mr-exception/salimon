import { geologicalDefinition } from '../src/planetDefinition.ts'
import { samplePlanetEnvironment } from '../src/planetEnvironment.ts'
import { test, expect } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { NavigationRuntime } from '../src/navigationRuntime.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { enterCockpit, leaveCockpit } from '../src/cockpit.ts'
import { SAMPLE_BODIES } from '../src/sampleUniverse.ts'
import { surfaceScoutLocation } from '../src/scoutLocation.ts'
import { physicalRadiusM } from '../src/contact.ts'
import { translate } from '../src/coordinates.ts'
import { point2 } from '../src/topDownFrames.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
import { walkTo, walkTicks } from './worldJourney'

test.use({ trace: { mode: 'retain-on-failure', snapshots: false, screenshots: false, sources: true } })
function approach(bodyId = 'vesper') {
 let raw: string | null = null
 let world = createTopDownState('interior', true)
 world.scoutLocation.boardingDoor = 'closed'
 world = enterCockpit({ ...world, selectedId: 'scout:chair', entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout', point: point2(0, -3.1) } } : e) })
 const session = new ResourceSession({ getItem: () => raw, setItem: (_, value) => { raw = value } }, world, undefined, undefined, () => Date.parse('2026-11-04T00:00:00Z'))
 expect(session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 })).toBe(true)
 expect(session.deposit('mineral', 500, session.snapshot.revision)).toBe(true)
 expect(session.transitionScout({ kind: 'begin-takeoff' }, session.snapshot.revision, session.world.cockpit)).toBe(true)
 const n = new NavigationRuntime(SAMPLE_BODIES, session.world.scoutLocation); n.advance(4); n.advance(2)
 if (bodyId !== 'vesper') {
  // Synthetic interplanetary approach only; browser must admit and fly the real
  // landing, not load an already-landed/generated exterior or pre-admitted plan.
  const body = SAMPLE_BODIES.find(b => b.id === bodyId)!
  const {version,digest}=geologicalDefinition(bodyId)
  const target = surfaceScoutLocation({ bodyId, radialM: [0, physicalRadiusM(body), 0], yaw: -.12, definition:{bodyId,version,digest} })
  n.clock.state = { ...n.clock.state, position: translate(target.ship.position, [0, 100, 0]), velocityMps: [0, 0, 0] }
  session.world.scoutLocation.scanner = { lastTick: n.clock.state.tick, pulse: Math.floor(n.clock.state.tick / 300), selected: bodyId, contactIds: [bodyId] }
 }
 session.world = leaveCockpit(session.world); session.checkpoint()
 return JSON.parse(raw!)
}
const epoch = new Date('2026-11-04T00:00:00Z')

test('LANDING-844 built Land control completes Vesper/moon walk, reload, cargo and relaunch journeys', async ({ page }, info) => {
 test.setTimeout(process.env.SALIMON_SLOW_BROWSER === '1' ? 900000 : 420000)
 const surfaceArt: Record<string, string> = {}
 const states: Record<string, unknown> = {}, main = page.locator('main'), pano = page.locator('.panoramic')
 const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
 const advance = async (ms: number) => { for (let t = 0; t < ms; t += 100) await page.clock.fastForward(Math.min(100, ms - t)) }
 const capture = async (name: string) => {
  const a = await page.screenshot({ path: info.outputPath(`${name}.png`) }), b = await page.screenshot({ path: info.outputPath(`${name}-repeat.png`) })
  const result = compareImages(a, b)
  await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ ...result, diff: undefined }), contentType: 'application/json' })
  await info.attach(`${name}-diff.png`, { body: result.diff, contentType: 'image/png' }); expect(result.pass).toBe(true)
  return a
 }
 await page.clock.install({ time: epoch }); await page.clock.pauseAt(new Date(epoch.getTime() + 1000))
 await page.goto('http://127.0.0.1:4175/')
 for (const bodyId of ['vesper', 'vesper-moon']) {
  const initial = approach(bodyId)
  await page.evaluate(({ key, value }) => localStorage.setItem(key, JSON.stringify(value)), { key: RESOURCE_SAVE_KEY, value: initial })
  await page.reload()
  await expect.poll(async () => { await page.clock.runFor(50); return page.locator('.top-down-scene').getAttribute('data-render-ready') }).toBe('true')
  await page.getByRole('button', { name: 'Use cockpit', exact: true }).click(); await page.clock.runFor(64)
  await expect(pano).toBeVisible()
  await expect(page.getByRole('button', { name: 'Land', exact: true })).toBeEnabled()
  await expect(page.locator('#landing-readings')).toContainText(bodyId === 'vesper' ? 'Vesper ·' : 'Vesper Moon ·')
  const before = await ship(), view = await pano.getAttribute('data-observation')
  const original = await capture(`${bodyId}-approach`)
  // Preserve a real mutated rendering and prove the comparison catches a lost HUD.
  const mutation = await page.addStyleTag({ content: '.landing-control { visibility:hidden !important }' })
  const negative = await page.screenshot({ path: info.outputPath(`${bodyId}-negative-hidden-landing.png`) })
  expect(compareImages(original, negative).pass).toBe(false); await mutation.evaluate(e => e.parentNode!.removeChild(e))
  await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
  const typing = await ship()
  await page.getByRole('textbox', { name: 'Search contacts' }).focus(); await page.keyboard.press('l'); await page.keyboard.press('Enter')
  expect((await ship()).mode).toBe(typing.mode)
  await page.keyboard.press('Escape')
  await pano.getByRole('button', { name: 'Help', exact: true }).click(); await page.keyboard.press('l'); await page.keyboard.press('ArrowLeft')
  expect((await ship()).mode).toBe('flying'); await page.keyboard.press('Escape')
  await page.getByRole('button', { name: 'Land', exact: true }).focus(); await page.keyboard.press('Enter'); await page.clock.runFor(32)
  await expect(pano).toHaveAttribute('data-mode', 'landing'); await expect(page.locator('#landing-status')).toContainText('Descent')
  await expect(page.getByRole('button', { name: 'Land', exact: true })).toBeDisabled()
  expect(await pano.getAttribute('data-observation')).toBe(view)
  await advance(2000)
  const during = await ship(); expect(during.position).not.toEqual(before.position)
  expect(await pano.getAttribute('data-observation')).toBe(view)
  await capture(`${bodyId}-descent`)
  await page.locator('canvas').focus(); await page.keyboard.down('w'); await page.clock.runFor(200); await page.keyboard.up('w'); await page.clock.runFor(64)
  expect(await pano.getAttribute('data-observation')).not.toBe(view)
  const observing = await pano.getAttribute('data-observation')
  await page.getByRole('button', { name: 'Exit cockpit' }).click(); await page.clock.runFor(64)
  await expect(page.locator('[data-entity][data-frame="planet:fixture"]')).toHaveCount(0)
  await page.getByRole('button', { name: 'Use cockpit', exact: true }).click(); await page.clock.runFor(64)
  expect(await pano.getAttribute('data-observation')).toBe(observing)
  await advance(10000)
  expect((await ship()).mode).toBe('grounded'); expect((await ship()).contact.bodyId).toBe(bodyId)
  await expect(main).toHaveAttribute('data-cockpit-view', 'world'); await expect(main).toHaveAttribute('data-player-mode', 'standing')
  await expect(main).toHaveAttribute('data-surface-bound', 'true')
  await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
  if (bodyId !== 'vesper') await expect(page.locator('[data-source-kind]')).toHaveCount(0)
  else await expect(page.locator('[data-entity="source:landing:mineral:01"]')).toHaveAttribute('data-remaining-grams', '59000')
  await page.clock.runFor(1600); await capture(`${bodyId}-touchdown-interior`)
  for (const p of [[0, -1.65], [1.35, -1.65], [1.35, .45], [0, .45], [0, 2], [0, 4.7]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e'); await page.clock.runFor(32); await walkTicks(page, 's', 70)
  await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
  await page.clock.runFor(1600); await capture(`${bodyId}-exterior`)
  surfaceArt[bodyId] = (await page.locator('[data-generated-ground] image').getAttribute('href'))!
  expect(surfaceArt[bodyId].length).toBeGreaterThan(1000)
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
  if(bodyId==='vesper-moon') expect(surfaceArt[bodyId]).not.toBe(surfaceArt.vesper)
  // P08 deliberately records the actual destination before departure; a
  // re-landing must preserve this observation with the depleted source/cargo.
  await page.keyboard.press('m')
  const notebook=page.getByRole('dialog',{name:'Field notebook'})
  await notebook.getByLabel('Name (optional)',{exact:true}).fill(`${bodyId} landing observation`)
  await notebook.getByRole('button',{name:'Mark current location'}).click()
  await expect(notebook.getByLabel('Known places')).toContainText(`${bodyId} landing observation`)
  await page.keyboard.press('Escape')
  await page.locator('main').click({position:{x:640,y:350}})
  await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
  const saved = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  expect(saved.stores).toEqual(initial.stores); expect(saved.sources).toEqual(initial.sources); expect(saved.inventory).toEqual(initial.inventory)
  await page.reload(); await page.clock.runFor(64)
  expect((await ship()).position).toEqual(saved.scout.ship.position)
  await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
  await walkTicks(page, 'w', 70); await walkTo(page, [0, 4.7], 'ship:scout', true)
  await page.keyboard.press('e'); await page.clock.runFor(32)
  for (const p of [[0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e'); await page.clock.runFor(64)
  await page.getByRole('button', { name: 'Take off', exact: true }).click(); await advance(6500)
  expect((await ship()).mode).toBe('flying')
  await page.getByRole('button', { name: 'Exit cockpit' }).click(); await page.clock.runFor(64)
  await expect(page.locator('[data-entity][data-frame="planet:fixture"]')).toHaveCount(0)
  const departed = await ship()
  // Revisit the same physical region through the production control, without
  // replacing the save or installing another approach fixture.
  await page.getByRole('button', { name: 'Use cockpit', exact: true }).click(); await page.clock.runFor(64)
  await page.getByRole('button', { name: 'Land', exact: true }).click()
  for (let i = 0; i < 300 && (await ship()).mode === 'landing'; i++) await advance(100)
  expect((await ship()).mode).toBe('grounded')
  const revisited = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  expect(revisited.scout.binding).toBe(saved.scout.binding)
  expect(revisited.sources).toEqual(saved.sources); expect(revisited.stores).toEqual(saved.stores)
  expect(saved.journal.marks).toHaveLength(1)
  expect(revisited.journal.marks).toEqual(saved.journal.marks)
  if (bodyId !== 'vesper') await expect(page.locator('[data-source-kind]')).toHaveCount(0)
  await capture(`${bodyId}-revisited`)
  states[bodyId] = { before, during, saved, departed, revisited }
 }
 await info.attach('journey.json', { body: JSON.stringify({ states, viewport: { width: 1280, height: 800 }, dpr: 1, epoch, seed: 'sampleUniverse / surface:v1', inputs: 'Playwright native keyboard/mouse; actual launch-derived Vesper save and synthetic moon approach; no pre-admitted landings' }), contentType: 'application/json' })
})

test('LANDING-844 production disabled causes, terminal impact and power-loss resume', async ({ page }, info) => {
 test.setTimeout(240000)
 const main = page.locator('main'), ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
 await page.clock.install({ time: epoch }); await page.clock.pauseAt(new Date(epoch.getTime() + 1000))
 await page.goto('/?powerFixture=1')
 for (const reason of ['speed', 'tangential', 'range', 'undetected', 'blocked', 'impact', 'valid']) {
  // A sparse moon patch isolates sideways speed from Vesper's nearby rocks.
  const initial = approach(reason === 'tangential' ? 'vesper-moon' : 'vesper')
  if (reason === 'tangential') initial.scout.ship.velocityMps = [20.01, 0, 0]
  if (reason === 'speed') initial.scout.ship.velocityMps = [0, 20.01, 0]
  if (reason === 'range') initial.scout.ship.position.offsetM[1] += 1e6
  if (reason === 'undetected') { initial.scout.scanner.contactIds = []; initial.scout.scanner.selected = null }
  if (reason === 'blocked') initial.sites['site:illustrated-landing:v1'].anchors['rock:west'] = [2, -1]
  if (reason === 'impact') initial.scout.ship.velocityMps = [0, -100, 0]
  await page.evaluate(({ key, value }) => localStorage.setItem(key, JSON.stringify(value)), { key: RESOURCE_SAVE_KEY, value: initial })
  await page.reload(); await page.clock.runFor(64)
  await expect.poll(async () => { await page.clock.runFor(50); return page.locator('.top-down-scene').getAttribute('data-render-ready') }).toBe('true')
  await page.getByRole('button', { name: 'Use cockpit', exact: true }).click(); await page.clock.runFor(64)
  if (reason === 'impact') {
   for (let i = 0; i < 65; i++) await page.clock.fastForward(100)
   expect((await ship()).mode).toBe('destroyed'); await expect(main).toHaveAttribute('data-surface-bound', 'false')
   await expect(page.locator('#landing-status')).toContainText('whole ship destroyed')
   await expect(page.getByRole('button', { name: 'Restart navigation' })).toHaveCount(0)
   await page.getByRole('button', { name: 'Exit cockpit' }).click(); await page.clock.runFor(64)
   const anchor = await page.locator('[data-entity="person:player"]').getAttribute('data-position')
   await walkTicks(page, 's', 10)
   expect(await page.locator('[data-entity="person:player"]').getAttribute('data-position')).toBe(anchor)
   await page.screenshot({ path: info.outputPath('terminal-interior.png') })
  } else if (reason === 'valid') {
   await page.getByRole('button', { name: 'Land', exact: true }).click(); await page.clock.runFor(64)
   await page.getByRole('button', { name: 'Test cockpit power loss' }).click(); await page.clock.runFor(64)
   await expect(main).toHaveAttribute('data-player-mode', 'seated'); await expect(main).toHaveAttribute('data-cockpit-view', 'world')
   expect((await ship()).mode).toBe('landing')
   await page.getByRole('button', { name: 'Restore cockpit power' }).click(); await page.keyboard.press('e'); await page.clock.runFor(64)
   await expect(page.locator('.panoramic')).toHaveAttribute('data-mode', 'landing')
   await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
   await page.reload(); await page.clock.runFor(64)
   expect((await ship()).mode).toBe('landing')
   for (let i = 0; i < 120; i++) await page.clock.fastForward(100)
   expect((await ship()).mode).toBe('grounded')
   await expect(main).toHaveAttribute('data-player-mode', 'standing')
  } else {
   await expect(page.getByRole('button', { name: 'Land', exact: true })).toBeDisabled()
   await expect(page.locator('#landing-status')).toContainText(({ speed: '20 m/s', tangential: '20 m/s', range: '0.15', undetected: 'Scan', blocked: 'footprint', unauthorized: 'Seal the door' } as Record<string, string>)[reason]!)
   if (reason === 'undetected') await expect(page.locator('#landing-readings')).not.toContainText('Vesper')
   const before = await ship(); await page.keyboard.press('l'); expect(await ship()).toEqual(before)
   await page.screenshot({ path: info.outputPath(`${reason}-denied.png`) })
  }
 }
})

test('SCAN-921 production scan gates the environment and warnings preserve landing admission', async ({ page }, info) => {
 const seed = approach('vesper-moon')
 const scanner = seed.scout.scanner
 seed.scout.scanner = { ...scanner, contactIds: [], selected: null }
 await page.addInitScript(({ key, seed }) => {
  if (!localStorage.getItem(key)) localStorage.setItem(key, JSON.stringify(seed))
 }, { key: RESOURCE_SAVE_KEY, seed })
 await page.setViewportSize({ width: 960, height: 600 })
 await page.clock.install({ time: epoch })
 await page.clock.pauseAt(new Date(epoch.getTime() + 1000))
 await page.goto('http://127.0.0.1:4175/')
 const enter = async () => {
  await expect.poll(async () => {
   await page.clock.runFor(32)
   return page.locator('.top-down-scene').getAttribute('data-render-ready')
  }).toBe('true')
  await page.getByRole('button', { name: 'Use cockpit', exact: true }).click()
  await page.clock.runFor(64)
 }
 await enter()
 const environment = page.getByRole('region', { name: 'Planetary environment and equipment' })
 const land = page.getByRole('button', { name: 'Land', exact: true })
 await expect(environment).toHaveCount(0)
 await expect(land).toBeDisabled()
 await page.screenshot({ path: info.outputPath('unscanned-cockpit.png') })

 seed.scout.scanner = scanner
 await page.evaluate(({ key, seed }) => localStorage.setItem(key, JSON.stringify(seed)), { key: RESOURCE_SAVE_KEY, seed })
 await page.reload()
 await enter()
 const gearWarning = page.getByText('Planetary environment · Gear warning', { exact: true })
 await expect(gearWarning).toBeVisible()
 await expect(land).toBeEnabled()
 await page.screenshot({ path: info.outputPath('compact-gear-warning.png') })
 await gearWarning.press('Enter')
 await expect(environment).toBeVisible()
 await expect(environment).toContainText('Body-wide temperature range -180 to 180 °C')
 await expect(environment).toContainText('Missing protection')
 const localLine = await environment.getByText('Landing site ·', { exact: false }).textContent()
 await expect(land).toBeEnabled()
 await environment.getByText('Missing protection', { exact: false }).last().scrollIntoViewIfNeeded()
 const a = await page.screenshot({ path: info.outputPath('scanned-cockpit.png') })
 const b = await page.screenshot({ path: info.outputPath('scanned-cockpit-repeat.png') })
 expect(compareImages(a, b).pass).toBe(true)
 const hidden = await page.addStyleTag({ content: '.scanner-environment { visibility: hidden !important }' })
 expect(compareImages(a, await page.screenshot({ path: info.outputPath('scanner-hidden.png') })).pass).toBe(false)
 await hidden.evaluate(element => element.parentNode!.removeChild(element))

 await land.click()
 await page.clock.runFor(32)
 const location = JSON.parse((await page.locator('main').getAttribute('data-scout-location'))!)
 const surface = location.landing.surface
 const expected = samplePlanetEnvironment(surface, surface.definition)
 expect(localLine).toBe(`Landing site · ${expected.temperatureC} °C · radiation ${expected.radiationHpPerSecond} HP/s · toxicity ${expected.toxicityHpPerSecond} HP/s`)
 await info.attach('scanner-site.json', { body: JSON.stringify({ scanner, surface, expected, localLine }), contentType: 'application/json' })
})
