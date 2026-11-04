import {test,expect} from '@playwright/test'
import type {} from './sceneHarness'
test('faceted navigation proxies render at readable sizes without a rectangular surface',async({page},testInfo)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message))
 await page.goto('/tests/fixture.html?scenario=visuals')
 await expect(page.locator('.scan-status')).toContainText('4 temporary contacts')
 await page.waitForFunction(()=>!!window.radarTest)
 await expect.poll(()=>page.evaluate(()=>window.radarTest.snapshot().objects.filter(o=>o.name.startsWith('nav-body:')).length)).toBe(4)
 const defaultView=await page.evaluate(()=>window.radarTest.snapshot().objects.filter(o=>o.name.startsWith('nav-body:')).map(o=>({name:o.name,pixel:o.pixel.map(v=>Math.round(v*1e6)/1e6)})))
 await expect.poll(()=>page.evaluate(()=>window.radarTest.snapshot().objects.filter(o=>o.name.startsWith('nav-body:')).length)).toBe(4)
 const samples=[]
 for(const zoom of [0.5,0.1,1]) {
  await page.goto(`/tests/fixture.html?scenario=visuals&view=tilted&zoom=${zoom}`)
  await page.waitForFunction(()=>!!window.radarTest)
  await page.waitForTimeout(350)
  const snapshot=await page.evaluate(()=>window.radarTest.snapshot());samples.push(snapshot)
  expect(snapshot.surfaces).toBe(0);expect(snapshot.geometries).toBeLessThan(20)
  expect(snapshot.renderer).toContain('SwiftShader');expect(snapshot.calls).toBeLessThan(30);expect(snapshot.triangles).toBeLessThan(1000)
  const ship=snapshot.objects.find(o=>o.name==='nav-ship-hull')!
  expect(ship.pixels).toBeGreaterThan(100);expect(ship.colors).toBeGreaterThan(5)
  if(zoom===0.5)for(const kind of ['star','planet','moon','asteroid']) {
   const body=snapshot.objects.find(o=>o.name===`nav-body:${kind}:${kind}`)!
   expect(body.pixels,kind).toBeGreaterThan(30);expect(body.colors,kind).toBeGreaterThan(5)
  }
  await testInfo.attach(`zoom-${zoom}.png`,{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 }
 await page.getByLabel('Scanned contacts (4)',{exact:true}).selectOption('moon')
 await expect.poll(()=>page.evaluate(()=>window.radarTest.snapshot().objects.some(o=>o.name==='nav-selection'))).toBe(true)
 const selected=await page.evaluate(()=>window.radarTest.snapshot());samples.push(selected)
 await testInfo.attach('selected.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await testInfo.attach('geometry-backend.json',{body:JSON.stringify({samples,errors},null,2),contentType:'application/json'})
 await page.goto('/tests/fixture.html?scenario=visuals');await page.waitForFunction(()=>!!window.radarTest)
 // Reset must cancel residual damping, not drift into new groups during clicks.
 await page.mouse.move(300,300);await page.mouse.down();await page.mouse.move(590,420,{steps:3});await page.mouse.up()
 await page.getByRole('button',{name:'Reset camera',exact:true}).click()
 await expect.poll(()=>page.evaluate(()=>window.radarTest.snapshot().objects.filter(o=>o.name.startsWith('nav-body:')).map(o=>({name:o.name,pixel:o.pixel.map(v=>Math.round(v*1e6)/1e6)})))).toEqual(defaultView)
 await page.waitForTimeout(300)
 expect(await page.evaluate(()=>window.radarTest.snapshot().objects.filter(o=>o.name.startsWith('nav-body:')).map(o=>({name:o.name,pixel:o.pixel.map(v=>Math.round(v*1e6)/1e6)})))).toEqual(defaultView)
 await testInfo.attach('camera-reset.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 expect(errors).toEqual([])
})
