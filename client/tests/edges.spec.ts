import {test,expect,type Page} from '@playwright/test'
import type {} from './sceneHarness'
import type {} from './fixture'
async function front(page:Page,scenario='edges',vector='zero',view='front') {
 await page.goto(`/tests/fixture.html?scenario=${scenario}&view=${view}&vector=${vector}`)
 await page.waitForFunction(()=>!!window.radarTest)
}
async function lines(page:Page) {
 return page.locator('.ship-vectors line').evaluateAll(nodes=>nodes.filter(n=>(n as SVGElement).style.display!=='none').map(n=>({kind:n.getAttribute('data-vector'),start:[Number(n.getAttribute('x1')),Number(n.getAttribute('y1'))],end:[Number(n.getAttribute('x2')),Number(n.getAttribute('y2'))],stroke:getComputedStyle(n).stroke,dash:getComputedStyle(n).strokeDasharray})))
}
function clipped(samples:Awaited<ReturnType<typeof lines>>) {
 for(const line of samples) {
  expect([...line.start,...line.end].every(v=>Number.isFinite(v)&&v>=0&&v<=100)).toBe(true)
  expect(Math.min(...line.end.map(v=>Math.min(v,100-v)))).toBeLessThan(1e-6)
 }
}
test('ship rays reach every edge/corner; no object guides or arrowheads; zero, differing, overlapping and opposite vectors',async({page},info)=>{
 await front(page)
 await expect(page.locator('.scan-status')).toContainText('8 temporary contacts')
 await expect(page.locator('.direction-lines,[data-offscreen]')).toHaveCount(0);await expect(page.locator('.edge')).toHaveCount(0)
 const samples=[]
 for(const [x,y] of [[1,0],[-1,0],[0,1],[0,-1],[1,1],[-1,1],[1,-1],[-1,-1]]) {
  const vector=y?(y>0?'top':'bottom')+(x?(x>0?'-right':'-left'):''):(x>0?'right':'left')
  await front(page,'edges',vector)
  await expect.poll(async()=> (await lines(page)).length).toBe(2)
  await expect.poll(async()=>{const line=(await lines(page))[0];return line?[...(x?[Math.round(line.end[0])]:[]),...(y?[Math.round(line.end[1])]:[])]:[]}).toEqual([...(x?[x>0?100:0]:[]),...(y?[y>0?0:100]:[])])
  const sample=await lines(page);clipped(sample);samples.push(sample)
  const heading=sample[0];if(x)expect(heading.end[0]).toBeCloseTo(x>0?100:0);if(y)expect(heading.end[1]).toBeCloseTo(y>0?0:100)
  expect(heading.stroke).toBe('rgb(255, 227, 106)');expect(sample[1].stroke).toBe('rgb(182, 245, 162)');expect(sample[1].dash).not.toBe('none')
 }
 await front(page,'edges','opposite');await page.waitForTimeout(150)
 const opposite=await lines(page);expect(opposite[0].end[0]).toBe(100);expect(opposite[1].end[0]).toBe(0)
 await info.attach('opposite-vectors.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await front(page,'edges','different');await page.waitForTimeout(150)
 const differing=await lines(page);expect(differing[0].end[0]).toBe(100);expect(differing[1].end[1]).toBe(0)
 await info.attach('differing-vectors.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await front(page,'edges','right');await page.waitForTimeout(150)
 await info.attach('overlapping-vectors.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await front(page,'edges','near');await page.waitForTimeout(150)
 await info.attach('nearly-overlapping-vectors.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await front(page,'edges','zero');await expect.poll(async()=> (await lines(page)).length).toBe(1)
 await expect(page.locator('.legend')).toContainText('STATIONARY')
 await page.getByLabel('Scanned contacts (8)',{exact:true}).selectOption('right');await expect(page.locator('.target-status')).toContainText('right')
 expect(await page.evaluate(()=>window.radarTest.snapshot().arrows)).toBe(0)
 await expect(page.locator('[marker-end],[marker-start]')).toHaveCount(0)
 await info.attach('vector-state.json',{body:JSON.stringify({samples,opposite,differing,gpu:await page.evaluate(()=>window.radarTest.snapshot())},null,2),contentType:'application/json'})
})
test('ship rays track actual pointer/WASD pan, wheel zoom, turn and off-screen origin clipping',async({page},info)=>{
 await front(page,'visuals','different');await page.waitForTimeout(150)
 const initial=await lines(page);clipped(initial)
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus()
 await page.keyboard.down('d');await page.waitForTimeout(200);await page.keyboard.up('d')
 const keyboard=await lines(page);clipped(keyboard);expect(keyboard).not.toEqual(initial)
 await page.mouse.move(400,400);await page.mouse.down({button:'right'});await page.mouse.move(480,450,{steps:6});await page.mouse.up({button:'right'})
 const pointer=await lines(page);clipped(pointer);expect(pointer).not.toEqual(keyboard)
 await page.mouse.wheel(0,-500);await page.waitForTimeout(150);const zoomed=await lines(page);clipped(zoomed);expect(zoomed).not.toEqual(pointer)
 await page.keyboard.down('ArrowUp');await page.waitForTimeout(300);await page.keyboard.up('ArrowUp')
 const turned=await lines(page);clipped(turned);expect(turned[0]).not.toEqual(zoomed[0]);expect(turned[1].end).toEqual(zoomed[1].end)
 await front(page,'edges','right','outside');await page.waitForTimeout(150)
 const outside=await lines(page);clipped(outside);expect(outside).toHaveLength(2)
 for(const line of outside){expect(line.start[0]).toBe(0);expect(line.end[0]).toBe(100)}
 await info.attach('ship-off-screen.png',{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
 await front(page,'edges','away','outside');await expect.poll(async()=> (await lines(page)).length).toBe(0)
 await info.attach('camera-vector-state.json',{body:JSON.stringify({initial,keyboard,pointer,zoomed,turned,outside,gpu:await page.evaluate(()=>window.radarTest.snapshot())},null,2),contentType:'application/json'})
})
test('off-screen contacts stay in picker after pulse and selection still expires with scanner range',async({page})=>{
 await front(page,'edges-unscanned')
 await expect(page.locator('.scan-status')).toContainText('awaiting first');await expect(page.locator('#contact-select option')).toHaveCount(1)
 await expect(page.locator('.scan-status')).toContainText('8 temporary contacts',{timeout:15000})
 await expect(page.locator('#contact-select option')).toHaveCount(9);await expect(page.locator('.edge,[data-offscreen],.direction-lines')).toHaveCount(0)
 await page.getByLabel('Scanned contacts (8)',{exact:true}).selectOption('right')
 await front(page,'expiry','');await expect(page.locator('.scan-status')).toContainText('8 temporary contacts');await page.getByLabel('Scanned contacts (8)',{exact:true}).selectOption('right');await expect(page.locator('.target-status')).toHaveText('No selected target');await expect(page.locator('#contact-select option')).toHaveCount(1)
})
