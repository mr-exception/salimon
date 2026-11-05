// Layout evidence only; complete gameplay/repeat acceptance runs in CI.
// Run from the repository root while the built preview is serving :4175.
const { chromium } = require(require.resolve('playwright', { paths: [process.cwd() + '/client'] }))
const fs = require('node:fs')
const path = require('node:path')
const output = __dirname
;(async () => {
  const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage', '--disable-gpu', '--disable-lcd-text'] })
  const records = []
  try {
    for (const [name, viewport] of [['desktop', { width: 1280, height: 800 }], ['narrow', { width: 390, height: 844 }]]) {
      const context = await browser.newContext({ viewport, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
      try {
        const page = await context.newPage(), errors = []
        page.on('pageerror', e => errors.push(e.message))
        await page.goto('http://127.0.0.1:4175')
        await page.locator('[data-inventory-weight]').waitFor()
        await page.waitForFunction(() => document.querySelector('[data-render-ready]')?.getAttribute('data-render-ready') === 'true')
        await page.evaluate(() => document.fonts.ready)
        await page.getByRole('button', { name: 'Equip gathering tool' }).click()
        await page.screenshot({ path: path.join(output, `field-inventory-${name}.png`), fullPage: true })
        records.push({ name, viewport, errors, browser: browser.version(), dpr: 1, renderer: 'headless Chromium CPU SVG/image path; GPU disabled; no WebGL claim',
          input: 'actual tool-button click; no gameplay state injection', origin: 'built preview :4175', clock: 'live; stationary default spawn',
          inventory: await page.getByLabel('Resource inventory').innerText(),
          horizontalOverflow: await page.evaluate(() => document.documentElement.scrollWidth > innerWidth) })
      } finally { await context.close() }
    }
  } finally { await browser.close() }
  fs.writeFileSync(path.join(output, 'preview.json'), JSON.stringify(records, null, 2) + '\n')
  console.log(JSON.stringify(records.map(({ name, errors, horizontalOverflow }) => ({ name, errors, horizontalOverflow }))))
})().catch(e => { console.error(e.message); process.exitCode = 1 })
