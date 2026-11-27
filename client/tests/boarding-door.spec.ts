import { test, expect } from '@playwright/test'
import { PNG } from 'pngjs'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { boot, walkTo, walkTicks, position } from './worldJourney'

// Production route and built assets; only real keyboard/mouse actions mutate the run.
test('DOOR-839 built boarding interaction, blocked crossing, reload and cockpit journey repeat', async ({browser},info)=>{
 test.setTimeout(300000)
 const originals: Record<string,Buffer>={}, records=[]
 for(let replay=0;replay<2;replay++){
  const context=await browser.newContext({viewport:{width:1280,height:800},deviceScaleFactor:1,colorScheme:'dark',locale:'en-US',timezoneId:'UTC',reducedMotion:'reduce'})
  const page=await context.newPage(),errors:string[]=[];page.on('pageerror',e=>errors.push(e.message))
  try {
   await boot(page,'','http://127.0.0.1:4175')
   const door=page.locator('[data-entity="scout:entry"]'),player=page.locator('[data-entity="person:player"]')
   const capture=async(name:string)=>{
    await page.mouse.move(0,0);await page.clock.runFor(5000)
    await expect(page.locator('.top-down-scene')).toHaveAttribute('data-render-ready','true')
    const png=await page.screenshot({path:info.outputPath(`${name}-${replay}.png`)})
    if (name === 'door-closed') {
      const leaf = (await door.locator('[data-scout-door-leaf] rect').first().boundingBox())!
      const image = PNG.sync.read(png)
      const x = Math.floor(leaf.x + leaf.width / 2), y = Math.floor(leaf.y + leaf.height / 2)
      const pixel = (y * image.width + x) * 4
      expect([...image.data.subarray(pixel, pixel + 3)]).toEqual([178, 129, 81])
    }
    if(!replay)originals[name]=png
    else {const r=compareImages(originals[name]!,png);await info.attach(`${name}-repeat.json`,{body:JSON.stringify({pass:r.pass,changedPixels:r.changedPixels,rawChangedPixels:r.rawChangedPixels,policy:POLICY}),contentType:'application/json'});await info.attach(`${name}-diff.png`,{body:r.diff,contentType:'image/png'});expect(r.pass,name).toBe(true)}
   }
   await expect(door).toHaveAttribute('data-door-state','open')
   for(const p of [[-2,3],[-2,7]])await walkTo(page,p,'planet:fixture',true)
   for(const p of [[0,7],[0,4.7]])await walkTo(page,p,'ship:scout',true)
   await capture('door-open')
   await page.keyboard.press('e');await page.clock.runFor(32)
   await expect(door).toHaveAttribute('data-door-state','closed')
   await expect(door.locator('[data-scout-door-leaf]')).toBeVisible()
   await expect(page.getByRole('button',{name:'Open door',exact:true})).toBeVisible()
   await capture('door-closed')
   await walkTicks(page,'s',35)
   expect((await position(page))[1]).toBeLessThanOrEqual(4.89+1e-6)
   await expect(player).toHaveAttribute('data-frame','ship:scout')
   await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
   const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('salimon:expedition:v1')!))
   expect(saved.scout.boardingDoor).toBe('closed');expect(saved.version).toBe(19)
   await page.reload();await page.clock.runFor(64)
   await expect.poll(async()=>{await page.clock.runFor(50);return door.getAttribute('data-door-state')}).toBe('closed')
   await page.getByRole('button',{name:'Open door',exact:true}).click();await page.clock.runFor(32)
   await expect(door).toHaveAttribute('data-door-state','open')
   await walkTo(page,[0,6.7],'ship:scout',true);await expect(player).toHaveAttribute('data-frame','planet:fixture')
   await page.getByRole('button',{name:'Close door',exact:true}).click();await page.clock.runFor(32)
   await expect(door).toHaveAttribute('data-door-state','closed')
   await walkTicks(page,'w',35);await expect(player).toHaveAttribute('data-frame','planet:fixture')
   await page.getByRole('button',{name:'Open door',exact:true}).click();await page.clock.runFor(32)
   await walkTo(page,[0,5.5],'ship:scout',true)
   await page.keyboard.press('e');await page.clock.runFor(32)
   await expect(page.getByRole('status').filter({hasText:/doorway occupied/})).toBeVisible()
   await expect(door).toHaveAttribute('data-door-state','open')
   await capture('door-occupied-denial')
   await walkTo(page,[0,4.7],'ship:scout',true);await page.keyboard.press('e');await page.clock.runFor(32)
   await expect(door).toHaveAttribute('data-door-state','closed')
   for(const p of [[0,2],[0,.45],[1.35,.45],[1.35,-1.65],[0,-1.65],[0,-3.1]])await walkTo(page,p,'ship:scout',true)
   await page.keyboard.press('e');await page.clock.runFor(64)
   await expect(page.locator('.panoramic')).toBeVisible()
   await expect(page.locator('[data-boarding-door-status]')).toHaveText('Aft door: closed')
   const backend=await page.locator('canvas').evaluate(c=>{const gl=(c as HTMLCanvasElement).getContext('webgl2')!,ext=gl.getExtension('WEBGL_debug_renderer_info')!;return gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)})
   expect(backend).toContain('SwiftShader');expect(errors).toEqual([])
   records.push({replay,backend,saved,ship:JSON.parse((await page.locator('main').getAttribute('data-ship'))!),errors})
  }finally{await context.close()}
 }
 await info.attach('door-state.json',{body:JSON.stringify({viewport:{width:1280,height:800},dpr:1,epoch:'2026-11-04T00:00:00Z',records}),contentType:'application/json'})
})

test('Scout modular cutaway joins walls, keeps the door readable and fades nearby furniture', async ({ browser }, info) => {
 const { createTopDownState, PLAYER_ID } = await import('../src/topDownRuntime.ts')
 const { point2 } = await import('../src/topDownFrames.ts')
 const { ResourceSession, RESOURCE_SAVE_KEY } = await import('../src/resourceSession.ts')
 const initial = createTopDownState('interior', true)
 const world = { ...initial, entities: initial.entities.map(entity => entity.id === PLAYER_ID
  ? { ...entity, anchor: { frame: 'ship:scout' as const, point: point2(0, -2.03) } } : entity) }
 let raw = ''
 const session = new ResourceSession({
  getItem: () => raw || null, setItem: (_, value) => { raw = value },
 }, world, undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
 session.upgrade(10000, 12)
 expect(session.error).toBe('')
 const states = []
 for (const reducedMotion of ['no-preference', 'reduce'] as const) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, reducedMotion })
  const page = await context.newPage()
  try {
   await page.addInitScript(({ key, value }) => {
    if (!localStorage.getItem(key)) localStorage.setItem(key, value)
   }, { key: RESOURCE_SAVE_KEY, value: raw })
   await boot(page, '', 'http://127.0.0.1:4175')
   const rails = page.locator('[data-scout-module="hull-rail"]')
   const deck = page.locator('[data-scout-module="deck"]')
   const crate = page.locator('[data-entity="fixture:crate"] .scout-obstruction')
   const avatar = page.locator('[data-entity="person:player"] image')
   await expect(rails).toHaveCount(2)
   await expect(deck.locator('[data-scout-module$="-floor"]')).toHaveCount(3)
   await expect(page.locator('[data-scout-module="window"]')).toHaveCount(1)
   await expect(page.locator('[data-entity="wall:cockpit-left"] [data-scout-joins]')).toHaveAttribute('data-scout-joins', 'joined capped')
   await expect(page.locator('[data-scout-module="boarding-door"]')).toBeVisible()
   await expect(page.locator('[data-scout-door-leaf]')).toHaveCount(0)
   await expect(crate).toHaveAttribute('data-scout-faded', 'false')
   await walkTicks(page, 's', 19)
   await expect(crate).toHaveAttribute('data-scout-faded', 'true')
   await expect(avatar).toBeVisible()
   await page.clock.runFor(250)
   await expect(crate).toHaveCSS('opacity', '0.26')
   await expect(crate).toHaveCSS('transition-duration', reducedMotion === 'reduce' ? '0s' : '0.19s')
   const stopped = await position(page)
   await walkTicks(page, 's', 15)
   expect(await position(page)).toEqual(stopped)
   await page.mouse.move(0, 0)
   await page.screenshot({ path: info.outputPath(`scout-fade-${reducedMotion}.png`) })
   await walkTicks(page, 'w', 16)
   await expect(crate).toHaveAttribute('data-scout-faded', 'false')
   await page.clock.runFor(250)
   await expect(crate).toHaveCSS('opacity', '1')
   states.push({ reducedMotion, stopped, restored: await position(page) })
  } finally { await context.close() }
 }
 await info.attach('scout-cutaway-state.json', { body: JSON.stringify(states), contentType: 'application/json' })
})
