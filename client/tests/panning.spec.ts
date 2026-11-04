import {test,expect} from '@playwright/test'
import type {PanRecording} from './sceneHarness'
import type {} from './fixture'
for(const input of ['keyboard','mouse'] as const)test(`continuous ${input} pan renders scanned bodies at their current projection on every frame`,async({page},info)=>{
 await page.goto('/tests/fixture.html?scenario=visuals&view=panned&vector=zero')
 await page.waitForFunction(()=>!!window.radarTest)
 await expect(page.locator('.scan-status')).toContainText('4 temporary contacts')
 await page.getByLabel('Scanned contacts (4)',{exact:true}).selectOption('planet')
 await page.waitForTimeout(200)
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus()
 const recording=page.evaluate(()=>window.radarTest.recordFrames(65))
 if(input==='keyboard')await page.keyboard.down('d')
 else {await page.mouse.move(820,400);await page.mouse.down({button:'right'});await page.mouse.move(120,400,{steps:45})}
 const data:PanRecording=await recording
 if(input==='keyboard')await page.keyboard.up('d');else await page.mouse.up({button:'right'})
 await expect(page.locator('.target-status')).toHaveText('Selected target: planet')
 await expect(page.locator('.direction-lines,[data-offscreen]')).toHaveCount(0)
 for(const [name,url] of Object.entries(data.captures))await info.attach(`${input}-${name}.png`,{body:Buffer.from(url.split(',')[1],'base64'),contentType:'image/png'})
 await info.attach(`${input}-frames.json`,{body:JSON.stringify({frames:data.frames,renderer:data.renderer},null,2),contentType:'application/json'})
 const observations=data.frames.flatMap(frame=>frame.bodies)
 expect(observations.length).toBeGreaterThan(20)
 expect(new Set(observations.map(b=>b.id)).size).toBe(4)
 expect(observations.filter(b=>!b.present),'every in-view body must exist on each movement frame').toEqual([])
 expect(observations.filter(b=>b.bright<10),'actual framebuffer must contain the body, even before drag/key release').toEqual([])
 for(const body of observations){expect(body.actualPixel![0]).toBeCloseTo(body.pixel[0],5);expect(body.actualPixel![1]).toBeCloseTo(body.pixel[1],5)}
 expect(data.frames.every(frame=>frame.ship!==null)).toBe(true)
})
