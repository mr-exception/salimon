import { test, expect, type Page } from '@playwright/test'
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import { execFileSync } from 'node:child_process'
import { isAbsolute, resolve, sep } from 'node:path'
import { compareImages, POLICY, sha256 } from '../scripts/visual-comparison.ts'

const directory = 'tests/navigation-baselines'
const candidate = process.env.SALIMON_PANORAMIC_CANDIDATES || undefined
if (candidate && process.env.CI) throw Error('Candidate generation is forbidden in CI')
if (candidate && (!isAbsolute(candidate) || resolve(candidate) === resolve(directory) || resolve(candidate).startsWith(resolve(directory) + sep))) throw Error('Use an absolute candidate directory outside tracked references')
const json = (v: unknown) => JSON.stringify(v, (_, x) => typeof x === 'bigint' ? String(x) : x, 2)
// Observers are fixture-only, read-only snapshots of the production adapters.
async function state(page: Page, owner: string): Promise<Record<string, unknown>> {
 return page.evaluate(owner => (window as unknown as Record<string, () => Record<string, unknown>>)[`${owner}Snapshot`](), owner)
}

test('PANO-VIS panoramic optical, sensor, actions and HUD reference matrix', async ({ page, browser }, info) => {
 test.setTimeout(360000)
 const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
 const source = await readFile('.revision-for-negative', 'utf8').catch(() => execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim())
 const environment = { platform: process.platform, arch: process.arch, browser: browser.version(), playwright: JSON.parse(await readFile('node_modules/@playwright/test/package.json', 'utf8')).version,
  viewport: { width: 1280, height: 800 }, dpr: 1, epoch: '2026-11-06T12:00:00.000Z', locale: 'en-US', timezone: 'UTC', policy: POLICY,
  fonts: Object.fromEntries(await Promise.all(['DejaVuSans.ttf', 'DejaVuSans-Bold.ttf', 'fonts.css', 'fonts.conf'].map(async f => [f, sha256(await readFile(`tests/assets/${f}`))]))) }
 expect(info.project.use.launchOptions?.env?.FONTCONFIG_FILE).toMatch(/tests\/assets\/fonts\.conf$/)
 const manifest = JSON.parse(await readFile(`${directory}/manifest.json`, 'utf8'))
 if (!candidate) expect(environment, 'PANORAMIC_ENVIRONMENT_DRIFT').toEqual(manifest.panoramic.environment)
 const images: Record<string, string> = {}, results: Record<string, unknown>[] = []
 const output = candidate ?? info.outputPath('comparisons'); await mkdir(output, { recursive: true })
 await page.clock.install({ time: new Date(environment.epoch) })
 await page.clock.pauseAt(new Date('2026-11-06T12:00:01Z'))
 const settle = async () => { await page.clock.runFor(160) }
 const boot = async (path: string, owner: string) => {
  await page.goto(path)
  await expect.poll(async () => { await settle(); return (await state(page, owner))?.renderer }).toContain('SwiftShader')
  await page.evaluate(() => document.fonts.ready); await settle()
 }
 const capture = async (id: string, owner: string) => {
  await page.locator('canvas').focus(); await page.mouse.move(1279, 799); await settle()
  const snapshot = await state(page, owner)
  const actual = await page.screenshot({ caret: 'hide' }), name = `panoramic-${id}.png`
  images[name] = sha256(actual)
  await writeFile(`${output}/${name}`, actual)
  await writeFile(`${output}/panoramic-${id}.json`, json({ source, environment, viewport: page.viewportSize(), dpr: await page.evaluate(() => window.devicePixelRatio), snapshot, errors }))
  await info.attach(name, { body: actual, contentType: 'image/png' })
  if (candidate) { results.push({ id, status: 'CANDIDATE', sha256: images[name] }); return }
  const expected = await readFile(`${directory}/${name}`)
  expect(sha256(expected), `PANORAMIC_BASELINE_HASH ${id}`).toBe(manifest.images[name])
  const { diff, ...comparison } = compareImages(expected, actual)
  await writeFile(`${output}/${id}-diff.png`, diff)
  results.push({ id, ...comparison, expected: sha256(expected), actual: images[name] })
  expect.soft(comparison.pass, `PANORAMIC_PIXEL_MISMATCH ${id}`).toBe(true)
 }
 for (const scenario of ['wide', 'telescope', 'telescope-translated', 'translated', 'surface', 'below']) {
  await boot(`/tests/optical.html?scenario=${scenario}`, 'optical')
  expect((await state(page, 'optical')).contacts).toBe(0)
  await capture(scenario, 'optical')
 }
 await boot('/tests/perspective.html?low-resolution', 'sensor')
 expect((await state(page, 'sensor')).selected).toBeNull()
 await capture('empty-scanner', 'sensor')
 // Adversarial optical clicks must never discover or select catalog bodies.
 await page.locator('canvas').click({ position: { x: 640, y: 400 } }); await settle()
 expect((await state(page, 'sensor')).selected, 'OPTICAL_SELECTION_LEAK').toBeNull()
 await page.getByRole('button', { name: 'Pulse', exact: true }).click(); await settle()
 const markers = (await state(page, 'sensor')).markers as { ids: string[] }[]
 expect(markers.flatMap(m => m.ids)).not.toContain('hidden')
 expect(markers.flatMap(m => m.ids)).not.toContain('back')
 await expect(page.getByRole('button', { name: 'Sensor contact Detected hidden', exact: true })).toHaveCount(0)
 await expect(page.getByRole('button', { name: 'Sensor contact Detected back', exact: true })).toHaveCount(0)
 await capture('detected-occluded-rear', 'sensor')
 await page.getByRole('button', { name: 'Sensor contact Detected planet', exact: true }).click(); await settle()
 expect((await state(page, 'sensor')).selected).toBe('planet') // hidden center cannot intercept
 await page.getByRole('button', { name: 'Sensor contact group of 18', exact: true }).click(); await settle()
 await capture('overlap', 'sensor')
 await page.getByRole('button', { name: 'Next', exact: true }).click(); await settle()
 await capture('pagination', 'sensor')
 await page.getByRole('button', { name: 'Telescope', exact: true }).click(); await settle()
 await capture('sensor-telescope', 'sensor')
 await page.getByRole('button', { name: 'Leave range', exact: true }).click(); await settle()
 expect((await state(page, 'sensor')).selected).toBeNull()
 await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toHaveCount(0)
 await capture('lost-contact', 'sensor')
 await boot('/tests/look-at.html', 'look')
 await page.getByRole('button', { name: 'Pulse', exact: true }).click(); await settle()
 await page.getByRole('textbox').fill('Behind')
 await page.getByRole('button', { name: 'Behind', exact: true }).click()
 const before = (await state(page, 'look')).ship
 await page.getByRole('button', { name: 'Look at', exact: true }).click()
 await page.clock.runFor(1200)
 expect((await state(page, 'look')).ship).toEqual(before)
 await capture('look-at-occluded', 'look')
 await page.getByRole('button', { name: 'Set direction', exact: true }).click()
 await page.clock.runFor(320)
 expect(((await state(page, 'look')).ship as { yaw: number }).yaw).not.toBe((before as { yaw: number }).yaw)
 await capture('set-direction', 'look')
 await boot('/tests/panoramic.html', 'hud')
 await page.evaluate(() => (window as unknown as { hudAdvance(s: number): void }).hudAdvance(5))
 await settle()
 expect(((await state(page, 'hud')).contacts as string[]).length).toBe(40)
 for (const kind of ['stationary', 'rear', 'overlap']) {
  await page.evaluate(kind => (window as unknown as { hudScenario(k: string): void }).hudScenario(kind), kind)
  await capture(`compass-${kind}`, 'hud')
 }
 await page.setViewportSize({ width: 960, height: 600 })
 await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
 await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
 for (const kind of ['warning', 'destroyed']) {
  await page.evaluate(kind => (window as unknown as { hudScenario(k: string): void }).hudScenario(kind), kind)
  await settle()
  expect(await page.evaluate(() => ({ w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight }))).toEqual({ w: 960, h: 600 })
  if (kind === 'warning') await expect(page.getByLabel('Navigation warnings')).not.toBeEmpty()
  if (kind === 'destroyed') await expect(page.getByRole('button', { name: 'Restart navigation', exact: true })).toBeInViewport()
  await capture(`small-${kind}`, 'hud')
 }
 expect(errors).toEqual([])
 await writeFile(`${output}/summary.json`, json({ source, environment, candidate, results, errors }))
 await info.attach('panoramic-summary.json', { body: json({ source, environment, candidate, results, errors }), contentType: 'application/json' })
 if (candidate) {
  // A complete proposal retains every other reference family, without editing
  // the tracked corpus or granting appearance approval.
  for (const name of Object.keys(manifest.images)) if (!images[name]) await writeFile(`${output}/${name}`, await readFile(`${directory}/${name}`))
  await writeFile(`${output}/manifest.json`, json({ ...manifest, panoramic: { environment, status: 'AWAITING_HUMAN_REVIEW' }, images: { ...manifest.images, ...images } }) + '\n')
 }
})
