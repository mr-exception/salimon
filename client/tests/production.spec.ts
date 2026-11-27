import {test,expect} from '@playwright/test'
test('built production bundle renders WebGL, scanner and labelled controls',async({page},testInfo)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('http://127.0.0.1:4175/?scene=radar')
 await expect(page.getByRole('img',{name:'Ship-centered three-dimensional radar',exact:true})).toBeVisible()
 await expect(page.locator('.scan-status')).toContainText('5 temporary contacts',{timeout:15000})
 await expect(page.locator('.legend')).toContainText('Scanner 29.98 Tm')
 await expect(page.locator('.stat').filter({hasText:'Speed (sample inertial frame)'})).toContainText('0 m/s')
 await testInfo.attach('metric-radar.png',{body:await page.screenshot(),contentType:'image/png'})
 await expect(page.getByLabel('Throttle',{exact:true})).toHaveValue('0');await expect(page.locator('.target-status')).toHaveText('No selected target');await expect(page.getByRole('status',{name:'Navigation warnings'})).toContainText('No navigation warnings')
 const gpu=await page.locator('canvas').evaluate(element=>{const c=element as HTMLCanvasElement;const gl=c.getContext('webgl2')!;return {width:c.width,height:c.height,version:gl.getParameter(gl.VERSION),renderer:gl.getParameter(gl.getExtension('WEBGL_debug_renderer_info')!.UNMASKED_RENDERER_WEBGL)}})
 expect(gpu.width).toBeGreaterThan(0);expect(gpu.renderer).toContain('SwiftShader');expect(errors).toEqual([])
 await testInfo.attach('backend.json',{body:JSON.stringify({gpu,errors,viewport:page.viewportSize()},null,2),contentType:'application/json'})
})

test('built production serves a deterministic art manifest and decodable assets', async ({ page }, info) => {
 const origin = 'http://127.0.0.1:4175'
 const request = await page.request.get(origin + '/assets/art/manifest.json')
 expect(request.ok()).toBe(true)
 const manifest = await request.json()
 expect(manifest.schemaVersion).toBe(1)
 expect(manifest.atlas.pages.every(({ width, height }: { width: number; height: number }) => width <= 2048 && height <= 2048)).toBe(true)
 await page.goto(origin + '/')
 const decoded = await page.evaluate(async urls => Promise.all(urls.map(async (url: string) => {
   const image = new Image()
   image.src = '/assets/art/' + url
   await image.decode()
   return [image.naturalWidth, image.naturalHeight]
 })), ['scout.chair', 'ui.seat'].map(id => manifest.assets.find((asset: { id: string }) => asset.id === id).parts[0].url))
 expect(decoded).toEqual([[49, 49], [256, 256]])
 await info.attach('art-manifest.json', { body: JSON.stringify({ manifest, decoded }, null, 2), contentType: 'application/json' })
})

test('built audio settings persist, a real door action plays, and muted warnings stay visible', async ({ browser }, info) => {
 const { boot, scene } = await import('./worldJourney.ts')
 const { ResourceSession, RESOURCE_SAVE_KEY } = await import('../src/resourceSession.ts')
 const { createTopDownState, PLAYER_ID } = await import('../src/topDownRuntime.ts')
 const { point2 } = await import('../src/topDownFrames.ts')
 const { AUDIO_PREFERENCES_KEY } = await import('../src/audio/audioManager.ts')
 const { compareImages } = await import('../scripts/visual-comparison.ts')
 let raw = ''
 const initial = createTopDownState('interior', true)
 const world = { ...initial, entities: initial.entities.map(entity => entity.id === PLAYER_ID
   ? { ...entity, anchor: { frame: 'ship:scout' as const, point: point2(0, 4.7) } } : entity) }
 const model = new ResourceSession({ getItem: () => raw || null, setItem: (_, value) => { raw = value } },
   world, undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
 model.survival = { ...model.survival, health: 39 }
 model.checkpoint()
 const records = []
 let original: Buffer | undefined
 for (let repeat = 0; repeat < 2; repeat++) {
   const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, reducedMotion: 'reduce' })
   const page = await context.newPage()
   try {
     await page.addInitScript(({ key, seed }) => {
       if (!localStorage.getItem(key)) localStorage.setItem(key, seed)
       const started: { channels: number; duration: number }[] = []
       Object.defineProperty(window, 'playedCues', { value: started })
       const start = AudioBufferSourceNode.prototype.start
       AudioBufferSourceNode.prototype.start = function (...args: Parameters<typeof start>) {
         start.apply(this, args)
         if (this.buffer) started.push({ channels: this.buffer.numberOfChannels, duration: this.buffer.duration })
       }
     }, { key: RESOURCE_SAVE_KEY, seed: raw })
     const played = () => page.evaluate(() => Reflect.get(window, 'playedCues') as { channels: number; duration: number }[])
     await boot(page, '', 'http://127.0.0.1:4175')
     expect(await played()).toEqual([])
     const gameplayBefore = await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
     await page.getByRole('button', { name: 'Help' }).click()
     await page.getByText('Audio settings', { exact: true }).click()
     const warnings = page.getByRole('slider', { name: /Warnings volume/ })
     await warnings.focus()
     await page.keyboard.press('Home')
     await page.getByRole('checkbox', { name: 'Mono positional audio' }).check()
     await expect(warnings).toHaveValue('0')
     expect(await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)).toBe(gameplayBefore)
     await page.mouse.move(0, 0)
     const capture = await page.screenshot({ path: info.outputPath(`audio-settings-${repeat}.png`) })
     if (original) {
       const result = compareImages(original, capture)
       await info.attach('audio-settings-diff.png', { body: result.diff, contentType: 'image/png' })
       expect(result.pass, `Audio settings repeat changed ${result.changedPixels} pixels`).toBe(true)
     } else original = capture
     await page.keyboard.press('Escape')
     await expect(page.getByRole('status').filter({ hasText: 'Low health · critical condition.' })).toBeVisible()
     await page.getByRole('button', { name: 'Close door', exact: true }).click()
     await page.clock.runFor(34)
     await expect(page.locator('[data-entity="scout:entry"]')).toHaveAttribute('data-door-state', 'closed')
     await expect.poll(played).toHaveLength(1)
     const [cue] = await played()
     expect(cue.channels).toBe(1)
     expect(cue.duration).toBeGreaterThan(.25)
     const codecs = await page.evaluate(() => {
       const audio = new Audio()
       return { mp3: audio.canPlayType('audio/mpeg'), opus: audio.canPlayType('audio/ogg; codecs=opus') }
     })
     expect(codecs.mp3).toBeTruthy()
     await page.reload()
     await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
     await expect(page.locator('[data-entity="scout:entry"]')).toHaveAttribute('data-door-state', 'closed')
     await expect(page.getByRole('status').filter({ hasText: 'Low health · critical condition.' })).toBeVisible()
     expect(await played()).toEqual([])
     const preferences = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), AUDIO_PREFERENCES_KEY)
     expect(preferences).toMatchObject({ volumes: { warnings: 0 }, mono: true })
     records.push({ repeat, cue, codecs, preferences })
   } finally { await context.close() }
 }
 await info.attach('audio-state.json', { body: JSON.stringify({ records, viewport: [1280, 800], dpr: 1 }, null, 2), contentType: 'application/json' })
})

test('built SVG round trips through the cockpit on one 60 Hz clock and reloads the same anchor', async ({ browser }, info) => {
 test.setTimeout(120_000)
 const { boot, scene } = await import('./worldJourney.ts')
 const { ResourceSession, RESOURCE_SAVE_KEY } = await import('../src/resourceSession.ts')
 const { createTopDownState, LAYERS, PLAYER_ID } = await import('../src/topDownRuntime.ts')
 const { point2 } = await import('../src/topDownFrames.ts')
 const { compareImages } = await import('../scripts/visual-comparison.ts')
 const initial = createTopDownState('interior', true)
 const world = { ...initial, entities: initial.entities.map(entity => entity.id === PLAYER_ID
   ? { ...entity, anchor: { frame: 'ship:scout' as const, point: point2(0, -3.1) } } : entity) }
 let raw = ''
 const model = new ResourceSession({ getItem: () => raw || null, setItem: (_, value) => { raw = value } },
   world, undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
 model.upgrade(10000, 12)
 expect(model.error).toBe('')
 const captures = new Map<string, Buffer>()
 const records = []
 for (let repeat = 0; repeat < 2; repeat++) {
   const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, reducedMotion: 'reduce' })
   const page = await context.newPage()
   const errors: string[] = []
   page.on('pageerror', error => errors.push(error.message))
   try {
     await page.addInitScript(({ key, seed }) => {
       if (!localStorage.getItem(key)) localStorage.setItem(key, seed)
     }, { key: RESOURCE_SAVE_KEY, seed: raw })
     await boot(page, '', 'http://127.0.0.1:4175')
     const main = page.locator('main')
     const worldSvg = await scene(page).elementHandle()
     const player = page.locator('[data-entity="person:player"]')
     const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
     const ticks = async () => ({ world: Number(await main.getAttribute('data-tick')), ship: (await ship()).tick as number })
     const measureSecond = async () => {
       const before = await ticks()
       for (let i = 0; i < 10; i++) await page.clock.fastForward(100)
       const after = await ticks()
       expect(after.world - before.world).toBeGreaterThanOrEqual(59)
       expect(after.world - before.world).toBeLessThanOrEqual(61)
       expect(after.ship - before.ship).toBe(after.world - before.world)
       return after.world - before.world
     }
     const capture = async (name: string) => {
       await page.mouse.move(0, 0)
       const png = await page.screenshot({ path: info.outputPath(`${name}-${repeat}.png`) })
       const first = captures.get(name)
       if (!first) { captures.set(name, png); return }
       const comparison = compareImages(first, png)
       await info.attach(`${name}-diff.png`, { body: comparison.diff, contentType: 'image/png' })
       expect(comparison.pass, `${name} changed ${comparison.changedPixels} pixels`).toBe(true)
     }
     expect(await scene(page).locator(':scope > [data-layer]').evaluateAll(nodes => nodes.map(node => node.getAttribute('data-layer')))).toEqual([...LAYERS])
     const worldTicks = await measureSecond()
     await capture('world')
     let gpu = ''
     const cockpitTicks = []
     const art = async () => JSON.parse((await worldSvg!.getAttribute('data-art-resources'))!)
     const worldArt = await art()
     expect(worldArt.decodedBytes).toBeGreaterThan(0)
     for (let round = 0; round < 3; round++) {
       const before = await ship()
       await page.keyboard.press('e')
       await expect(main).toHaveAttribute('data-cockpit-view', 'radar')
       await expect(page.locator('canvas')).toHaveCount(1)
       await expect.poll(art).toEqual({ count: 0, encodedBytes: 0, decodedBytes: 0 })
       await expect(player).toHaveAttribute('data-position', '0,-3.95')
       if (round === 0) {
         gpu = await page.locator('canvas').evaluate(canvas => {
           const gl = (canvas as HTMLCanvasElement).getContext('webgl2')!
           return gl.getParameter(gl.getExtension('WEBGL_debug_renderer_info')!.UNMASKED_RENDERER_WEBGL) as string
         })
         expect(gpu).toContain('SwiftShader')
       }
       cockpitTicks.push(await measureSecond())
       if (round === 0) await capture('panoramic')
       await page.getByRole('button', { name: 'Exit cockpit', exact: true }).click()
       await expect(main).toHaveAttribute('data-cockpit-view', 'world')
       await expect(page.locator('canvas')).toHaveCount(0)
       await expect(player).toHaveAttribute('data-position', '0,-3.1')
       expect((await ship()).position).toEqual(before.position)
       expect(await worldSvg!.evaluate(svg => svg.isConnected)).toBe(true)
       await expect.poll(art).toEqual(worldArt)
     }
     await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
     const saved = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
     expect(saved.location.anchor.frame).toBe('ship:scout')
     await page.reload()
     await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
     await expect(player).toHaveAttribute('data-position', '0,-3.1')
     await expect(player).toHaveAttribute('data-frame', 'ship:scout')
     expect(errors).toEqual([])
     records.push({ repeat, gpu, worldTicks, cockpitTicks, worldArt, savedLocation: saved.location, errors })
   } finally { await context.close() }
 }
 await info.attach('renderer-production-state.json', { body: JSON.stringify({ records, viewport: [1280, 800], dpr: 1 }, null, 2), contentType: 'application/json' })
})

test('built artwork culls offscreen sprites, releases URLs on hide, and recovers a failed initial raster', async ({ page }, info) => {
 const { boot, scene } = await import('./worldJourney.ts')
 const { compareImages } = await import('../scripts/visual-comparison.ts')
 const origin = 'http://127.0.0.1:4175'
 const requests: string[] = []
 page.on('request', request => {
   const url = new URL(request.url())
   if (url.origin === origin && url.pathname.startsWith('/assets/')) requests.push(url.pathname)
 })
 await page.addInitScript(() => {
   const urls = new Set<string>()
   const create = URL.createObjectURL.bind(URL), revoke = URL.revokeObjectURL.bind(URL)
   URL.createObjectURL = blob => { const url = create(blob); urls.add(url); return url }
   URL.revokeObjectURL = url => { urls.delete(url); revoke(url) }
   Object.defineProperty(window, 'activeArtUrls', { get: () => urls.size })
 })
 await boot(page, '', origin)
 const world = scene(page)
 const art = async () => JSON.parse((await world.getAttribute('data-art-resources'))!)
 const urls = () => page.evaluate(() => Reflect.get(window, 'activeArtUrls') as number)
 const normal = await art()
 const player = page.locator('[data-entity="person:player"]')
 const physicalPosition = await player.getAttribute('data-position')
 await page.mouse.move(0, 0)
 const original = await page.screenshot({ path: info.outputPath('world-initial.png') })
 await page.mouse.move(140, 480)
 await page.mouse.down({ button: 'middle' })
 await page.mouse.move(1200, 480, { steps: 8 })
 await page.mouse.up({ button: 'middle' })
 await expect(world).toHaveAttribute('data-camera-panned', 'true')
 await expect(player.locator('image')).toHaveCount(0)
 await expect(player).toHaveAttribute('data-position', physicalPosition!)
 await page.keyboard.press('Space')
 await page.clock.runFor(3000)
 await expect.poll(art).toEqual(normal)
 await page.evaluate(() => {
   Object.defineProperty(document, 'hidden', { configurable: true, value: true })
   document.dispatchEvent(new Event('visibilitychange'))
 })
 await expect.poll(urls).toBe(0)
 const before = await page.locator('main').getAttribute('data-tick')
 await page.clock.runFor(1000)
 await expect(page.locator('main')).toHaveAttribute('data-tick', before!)
 await page.evaluate(() => {
   Reflect.deleteProperty(document, 'hidden')
   document.dispatchEvent(new Event('visibilitychange'))
 })
 await expect.poll(art).toEqual(normal)
 await expect.poll(urls).toBe(normal.count)
 await page.mouse.move(0, 0)
 const restored = await page.screenshot({ path: info.outputPath('world-restored.png') })
 const comparison = compareImages(original, restored)
 await info.attach('world-diff.png', { body: comparison.diff, contentType: 'image/png' })
 expect(comparison.pass, `World artwork changed ${comparison.changedPixels} pixels`).toBe(true)
 expect(requests.some(path => /\/index-[^/]+\.js$/.test(path))).toBe(true)
 expect(requests.some(path => path.includes('PanoramicNavigation-'))).toBe(false)
 await info.attach('asset-lifecycle.json', { body: JSON.stringify({ renderer: 'SVG CPU', normal, hiddenUrls: 0, requests, comparison: comparison.changedPixels }, null, 2), contentType: 'application/json' })
 await page.addInitScript(() => {
   const original = HTMLCanvasElement.prototype.toBlob
   HTMLCanvasElement.prototype.toBlob = function (callback) {
     HTMLCanvasElement.prototype.toBlob = original
     callback(null)
   }
 })
 await page.reload()
 await expect(page.getByRole('alert').filter({ hasText: 'Some world artwork could not load' })).toBeVisible()
 await page.getByRole('button', { name: 'Retry artwork' }).click()
 await expect(world).toHaveAttribute('data-render-ready', 'true')
 await expect(page.getByRole('button', { name: 'Retry artwork' })).toHaveCount(0)
 await expect(player).toHaveAttribute('data-position', physicalPosition!)
})

test('built graphics settings persist, reduce raster work and retain world controls and cues', async ({ page }, info) => {
 const { boot, position, scene, walkTicks } = await import('./worldJourney.ts')
 const { compareImages } = await import('../scripts/visual-comparison.ts')
 const { RESOURCE_SAVE_KEY } = await import('../src/resourceSession.ts')
 const { GRAPHICS_KEY } = await import('../src/rendering/graphicsPreferences.ts')
 await boot(page, '?renderDiagnostics', 'http://127.0.0.1:4175')
 const world = scene(page)
 const resources = async () => JSON.parse((await world.getAttribute('data-art-resources'))!)
 const original = await resources()
 const save = () => page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
 const before = await save()
 await expect(page.locator('[data-dev-render-diagnostics],[data-dev-frame-diagnostics]')).toHaveCount(0)
 await page.getByRole('button', { name: 'Help', exact: true }).click()
 await page.getByText('Graphics settings', { exact: true }).click()
 const selector = page.getByRole('combobox', { name: 'World quality' })
 await expect(selector).toHaveValue('Medium')
 await selector.focus()
 await page.keyboard.press('Home')
 await expect(selector).toHaveValue('Low')
 await expect(world).toHaveAttribute('data-world-raster-scale', '0.75')
 await expect.poll(async () => (await resources()).decodedBytes).toBeLessThan(original.decodedBytes)
 expect(await save()).toBe(before)
 await page.keyboard.press('Escape')
 const avatar = page.locator('[data-entity="person:player"] image')
 await expect(avatar).toBeVisible()
 await expect(page.getByRole('meter', { name: 'Health' })).toBeVisible()
 await page.reload()
 await expect.poll(async () => { await page.clock.runFor(50); return world.getAttribute('data-render-ready') }).toBe('true')
 await page.mouse.move(0, 0)
 const low = await page.screenshot({ path: info.outputPath('world-low.png') })
 await page.reload()
 await expect.poll(async () => { await page.clock.runFor(50); return world.getAttribute('data-render-ready') }).toBe('true')
 await expect(world).toHaveAttribute('data-world-raster-scale', '0.75')
 const repeat = await page.screenshot({ path: info.outputPath('world-low-reload.png') })
 const comparison = compareImages(low, repeat)
 await info.attach('world-low-diff.png', { body: comparison.diff, contentType: 'image/png' })
 expect(comparison.pass).toBe(true)
 await page.getByRole('button', { name: 'Help', exact: true }).click()
 await page.getByText('Graphics settings', { exact: true }).click()
 await expect(selector).toHaveValue('Low')
 await selector.selectOption('High')
 await expect(page.locator('[data-world-shadows="dynamic"]')).toHaveCount(1)
 await selector.selectOption('Custom')
 const scale = page.getByRole('slider', { name: 'World render scale' })
 await scale.focus()
 await page.keyboard.press('Home')
 await page.getByRole('combobox', { name: 'Resolution cap' }).selectOption('1.5')
 await expect(world).toHaveAttribute('data-world-raster-scale', '0.5')
 await page.keyboard.press('Escape')
 await world.focus()
 const start = await position(page)
 await walkTicks(page, 'w', 12)
 expect(await position(page)).not.toEqual(start)
 const stored = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), GRAPHICS_KEY)
 expect(stored).toMatchObject({ preset: 'Custom', custom: { scale: 50, dprCap: 1.5 } })
 await info.attach('graphics-quality.json', { body: JSON.stringify({ backend: 'SVG CPU', original, custom: await resources(), stored, repeatChangedPixels: comparison.changedPixels }, null, 2), contentType: 'application/json' })
})

test('built cockpit honors the quality scale and keeps controls without WebGL 2', async ({ page }, info) => {
 const { boot, scene } = await import('./worldJourney.ts')
 const { ResourceSession, RESOURCE_SAVE_KEY } = await import('../src/resourceSession.ts')
 const { createTopDownState, PLAYER_ID } = await import('../src/topDownRuntime.ts')
 const { point2 } = await import('../src/topDownFrames.ts')
 const { GRAPHICS_KEY } = await import('../src/rendering/graphicsPreferences.ts')
 const initial = createTopDownState('interior', true)
 const world = { ...initial, entities: initial.entities.map(entity => entity.id === PLAYER_ID
   ? { ...entity, anchor: { frame: 'ship:scout' as const, point: point2(0, -3.1) } } : entity) }
 let raw = ''
 const model = new ResourceSession({ getItem: () => null, setItem: (_, value) => { raw = value } },
   world, undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
 model.upgrade(10000, 12)
 expect(model.error).toBe('')
 expect(raw).not.toBe('')
 await page.addInitScript(({ key, seed, graphicsKey }) => {
   if (!localStorage.getItem(key)) localStorage.setItem(key, seed)
   if (!localStorage.getItem(graphicsKey)) localStorage.setItem(graphicsKey, JSON.stringify({ preset: 'Low' }))
   if (!sessionStorage.getItem('deny-test-webgl2')) return
   const original = HTMLCanvasElement.prototype.getContext
   HTMLCanvasElement.prototype.getContext = function (this: HTMLCanvasElement, ...args: unknown[]) {
     return args[0] === 'webgl2' ? null : Reflect.apply(original, this, args)
   } as typeof original
 }, { key: RESOURCE_SAVE_KEY, seed: raw, graphicsKey: GRAPHICS_KEY })
 await boot(page, '', 'http://127.0.0.1:4175')
 const errors: string[] = []
 page.on('pageerror', error => errors.push(error.message))
 await page.keyboard.press('e')
 const canvas = page.locator('canvas')
 await expect(canvas).toBeVisible()
 const measureResolution = () => canvas.evaluate(element => {
   const canvas = element as HTMLCanvasElement
   return { actual: canvas.width, css: canvas.getBoundingClientRect().width, dpr: devicePixelRatio }
 })
 await expect.poll(async () => {
   await page.clock.runFor(50)
   const { actual, css } = await measureResolution()
   return css > 600 && actual === Math.floor(css * .75)
 }).toBe(true)
 const resolution = await measureResolution()
 await info.attach('cockpit-render-scale.json', { body: JSON.stringify(resolution), contentType: 'application/json' })
 await page.getByRole('button', { name: 'Exit cockpit', exact: true }).click()
 await page.evaluate(() => sessionStorage.setItem('deny-test-webgl2', 'yes'))
 await page.reload()
 await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
 await page.keyboard.press('e')
 await expect(page.getByRole('alert').filter({ hasText: 'requires WebGL 2' })).toBeVisible()
 await expect(page.locator('canvas')).toHaveCount(0)
 await page.screenshot({ path: info.outputPath('unsupported-cockpit.png') })
 await page.getByRole('button', { name: 'Exit cockpit', exact: true }).click()
 await expect(scene(page)).toBeVisible()
 await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-position', '0,-3.1')
 expect(errors).toEqual([])
})
