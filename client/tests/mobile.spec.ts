import {test,expect} from '@playwright/test'
test('mobile layout, accessible selection, zoom controls and rotation resize',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('/')
 await expect(page.locator('.scan-status')).toContainText('5 temporary contacts',{timeout:15000})
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true)
 await page.getByLabel('Scanned contacts (5)',{exact:true}).selectOption('vesper');await expect(page.locator('.target-status')).toHaveText('Selected target: Vesper')
 for(let i=0;i<4;i++)await page.getByRole('button',{name:'Zoom in',exact:true}).tap()
 await expect(page.locator('.edge').first()).toBeAttached()
 for(const button of await page.locator('.edge').all()){
  const box=(await button.boundingBox())!;expect(box.x).toBeGreaterThanOrEqual(0);expect(box.x+box.width).toBeLessThanOrEqual(391)
 }
 await page.setViewportSize({width:844,height:390});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true)
 await page.getByRole('button',{name:'Reset camera',exact:true}).tap();expect(errors).toEqual([])
})
test('touch release outside, touch cancel and viewport resize clear held controls',async({page,context})=>{
 await page.goto('/');const button=page.getByRole('button',{name:'Yaw left',exact:true}),heading=page.locator('.stat').filter({hasText:'Yaw / pitch'}).locator('strong')
 const cdp=await context.newCDPSession(page)
 for(const end of ['touchEnd','touchCancel','resize'] as const) {
  await button.scrollIntoViewIfNeeded();const b=(await button.boundingBox())!,initial=await heading.textContent()
  await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:b.x+b.width/2,y:b.y+b.height/2,id:1}]});await page.waitForTimeout(300)
  await expect(heading).not.toHaveText(initial!)
  if(end==='resize')await page.setViewportSize({width:400,height:844})
  else {await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:10,y:10,id:1}]});await cdp.send('Input.dispatchTouchEvent',{type:end,touchPoints:[]})}
  await page.waitForTimeout(250);const stopped=await heading.textContent();await page.waitForTimeout(350);await expect(heading).toHaveText(stopped!)
  if(end==='resize')await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]})
 }
})
