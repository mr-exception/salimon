import { test, expect } from '@playwright/test'
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
    if(!replay)originals[name]=png
    else {const r=compareImages(originals[name]!,png);await info.attach(`${name}-repeat.json`,{body:JSON.stringify({pass:r.pass,changedPixels:r.changedPixels,rawChangedPixels:r.rawChangedPixels,policy:POLICY}),contentType:'application/json'});await info.attach(`${name}-diff.png`,{body:r.diff,contentType:'image/png'});expect(r.pass,name).toBe(true)}
   }
   await expect(door).toHaveAttribute('data-door-state','open')
   for(const p of [[-2,3],[-2,7]])await walkTo(page,p,'planet:fixture',true)
   for(const p of [[0,7],[0,4.7]])await walkTo(page,p,'ship:scout',true)
   await capture('door-open')
   await page.keyboard.press('e');await page.clock.runFor(32)
   await expect(door).toHaveAttribute('data-door-state','closed')
   await expect(page.getByRole('button',{name:'Open door',exact:true})).toBeVisible()
   await capture('door-closed')
   await walkTicks(page,'s',35)
   expect((await position(page))[1]).toBeLessThanOrEqual(4.89+1e-6)
   await expect(player).toHaveAttribute('data-frame','ship:scout')
   await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
   const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('salimon:expedition:v1')!))
   expect(saved.scout.boardingDoor).toBe('closed');expect(saved.version).toBe(10)
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
