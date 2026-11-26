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
