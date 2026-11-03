import {test,expect,type Page} from '@playwright/test'
import type {} from './sceneHarness'
import type {} from './fixture'
async function front(page:Page,zoom=.5,target:[number,number,number]=[0,0,0]) {
 await page.waitForFunction(()=>!!window.radarTest)
 await page.evaluate(({zoom,target})=>window.radarTest.camera([target[0],target[1],200000],target,zoom),{zoom,target})
 await page.waitForTimeout(300)
}
async function lines(page:Page) {
 return page.locator('.direction-lines line').evaluateAll(nodes=>nodes.map(node=>({id:node.getAttribute('data-contact-id'),selected:node.classList.contains('selected'),start:[Number(node.getAttribute('x1')),Number(node.getAttribute('y1'))],end:[Number(node.getAttribute('x2')),Number(node.getAttribute('y2'))]})))
}
function clipped(samples:Awaited<ReturnType<typeof lines>>) {
 for(const line of samples) {
  expect([...line.start,...line.end].every(Number.isFinite)).toBe(true)
  expect([...line.start,...line.end].every(v=>v>=0&&v<=100)).toBe(true)
  expect(Math.min(...line.end.map(v=>Math.min(v,100-v)))).toBeLessThan(1e-8)
 }
}
test('all edges/corners, selected keyboard contact and actual pan/orbit/zoom keep correct clipped lines',async({page},testInfo)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('/tests/fixture.html?scenario=edges')
 await front(page);await expect(page.locator('.direction-lines line')).toHaveCount(8)
 const initial=await lines(page);clipped(initial)
 for(const line of initial) {
  expect(line.start).toEqual([50,50])
  if(line.id?.includes('right'))expect(line.end[0]).toBeCloseTo(100)
  if(line.id?.includes('left'))expect(line.end[0]).toBeCloseTo(0)
  if(line.id?.includes('top'))expect(line.end[1]).toBeCloseTo(0)
  if(line.id?.includes('bottom'))expect(line.end[1]).toBeCloseTo(100)
 }
 const labels=await page.locator('.edge').evaluateAll(nodes=>nodes.map(n=>{const r=n.getBoundingClientRect();return {x:r.x,y:r.y,right:r.right,bottom:r.bottom}}))
 const banners=await page.locator('.top-label,.legend').evaluateAll(nodes=>nodes.map(n=>{const r=n.getBoundingClientRect();return {x:r.x,y:r.y,right:r.right,bottom:r.bottom}}))
 for(let i=0;i<labels.length;i++)for(const b of [...banners,...labels.slice(i+1)]) {
  const a=labels[i];expect(a.right<=b.x||b.right<=a.x||a.bottom<=b.y||b.bottom<=a.y,'edge labels must not overlap each other or navigation banners').toBe(true)
 }
 await testInfo.attach('all-edges.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 const right=page.getByRole('button',{name:'Off-screen right',exact:true});await right.focus();await page.keyboard.press('Enter')
 await expect(page.locator('.target-status')).toHaveText('Selected target: right')
 await expect(page.getByRole('button',{name:'Off-screen right, selected right',exact:true})).toHaveAttribute('aria-pressed','true')
 await expect(page.locator('.direction-lines line.selected')).toHaveAttribute('data-contact-id','right')
 await testInfo.attach('selected-edge.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 const before=await page.locator('.stat').allTextContents()
 // Actual browser mouse input, through production OrbitControls.
 await page.mouse.move(450,400);await page.mouse.down({button:'right'});await page.mouse.move(750,500,{steps:8});await page.mouse.up({button:'right'});await page.waitForTimeout(350)
 const panned=await lines(page);clipped(panned);expect(panned).not.toEqual(initial);expect(await page.locator('.stat').allTextContents()).toEqual(before)
 await page.mouse.move(450,400);await page.mouse.wheel(0,-450);await page.waitForTimeout(350)
 const zoomed=await lines(page);clipped(zoomed);expect(zoomed).not.toEqual(panned)
 await page.mouse.move(450,400);await page.mouse.down();await page.mouse.move(620,460,{steps:8});await page.mouse.up();await page.waitForTimeout(350)
 const rotated=await lines(page);clipped(rotated);expect(rotated).not.toEqual(zoomed)
 expect(await page.locator('.stat').allTextContents()).toEqual(before)
 await testInfo.attach('camera-moved.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await front(page,.5,[1200,0,0]);const shipOutside=await lines(page);clipped(shipOutside)
 expect(shipOutside.some(line=>Math.abs(line.start[0])<1e-8)).toBe(true)
 expect(shipOutside.some(line=>line.start[0]===50&&line.start[1]===50)).toBe(false)
 await testInfo.attach('ship-off-screen.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await front(page,.5,[2500,0,0]);await expect(page.locator('.direction-lines line')).toHaveCount(0)
 await front(page);await page.evaluate(()=>window.navigationTest.moveShipLs([200,0,0]));await page.waitForTimeout(350)
 const moved=await lines(page);clipped(moved);expect(moved).not.toEqual(initial)
 const projections=await page.evaluate(()=>({ship:window.radarTest.projectLs([0,0,0]),contacts:window.navigationTest.positions().map(b=>({id:b.id,point:window.radarTest.projectLs([...b.positionLs])}))}))
 for(const line of moved) {
  const target=projections.contacts.find(b=>b.id===line.id)!.point
  const dx=target[0]-projections.ship[0],dy=target[1]-projections.ship[1]
  for(const point of [line.start,line.end])expect(Math.abs((point[0]-projections.ship[0])*dy-(point[1]-projections.ship[1])*dx)).toBeLessThan(1e-6)
 }
 const gpu=await page.evaluate(()=>window.radarTest.snapshot())
 await testInfo.attach('edge-state.json',{body:JSON.stringify({initial,panned,zoomed,rotated,shipOutside,moved,projections,gpu,errors},null,2),contentType:'application/json'})
 expect(errors).toEqual([])
})
test('edge discovery waits for pulse; out-of-range targets stay hidden and selected contacts expire',async({page})=>{
 await page.goto('/tests/fixture.html?scenario=edges-unscanned');await front(page)
 await expect(page.locator('.scan-status')).toContainText('awaiting first');await expect(page.locator('.direction-lines line')).toHaveCount(0)
 await expect(page.locator('.scan-status')).toContainText('8 temporary contacts',{timeout:15000})
 await expect(page.locator('.direction-lines line')).toHaveCount(8)
 await expect(page.getByRole('button',{name:/Out of range/})).toHaveCount(0)
 await page.getByRole('button',{name:'Off-screen right',exact:true}).click()
 await page.evaluate(()=>window.navigationTest.moveShipLs([300000,0,0]))
 await expect(page.locator('.direction-lines line')).toHaveCount(0)
 await expect(page.locator('.edge')).toHaveCount(0);await expect(page.locator('.target-status')).toHaveText('No selected target')
})
test('collinear contacts share one subdued ray while all IDs and selected representative stay accessible',async({page},testInfo)=>{
 await page.goto('/tests/fixture.html?scenario=edges-dense');await front(page)
 await expect(page.locator('.scan-status')).toContainText('2000 temporary contacts');await expect(page.locator('.direction-lines line')).toHaveCount(1)
 await expect(page.locator('.edge')).toHaveCount(1)
 const grouped=page.getByRole('button',{name:'Off-screen 2000 contacts',exact:true});await grouped.focus();await page.keyboard.press('Enter')
 await expect(page.locator('.contact-menu button')).toHaveCount(34)
 await page.getByRole('button',{name:'Next contacts',exact:true}).click();await page.getByRole('button',{name:'Edge contact 0032',exact:true}).click()
 await expect(page.locator('.target-status')).toContainText('Edge contact 0032')
 await page.getByLabel('Find scanned contacts').fill('1999');await page.getByLabel('Scanned contacts (2000)',{exact:true}).selectOption('edge-1999')
 await expect(page.locator('.direction-lines line.selected')).toHaveAttribute('data-contact-id','edge-1999')
 await expect(page.getByRole('button',{name:'Off-screen 2000 contacts, selected Edge contact 1999',exact:true})).toHaveAttribute('aria-pressed','true')
 await testInfo.attach('overlap.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await testInfo.attach('overlap-state.json',{body:JSON.stringify(await lines(page),null,2),contentType:'application/json'})
})
