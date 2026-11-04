import {expect,type Page,type TestInfo} from '@playwright/test'
import type {} from './fixture'
import type {} from './sceneHarness'
export const EPOCH='2026-11-04T00:00:00.000Z'
export const json=(value:unknown)=>JSON.stringify(value,(_key,v)=>typeof v==='bigint'?v.toString():v,2)
// Freeze time before navigation; only runFor advances RAF, input and simulation.
export async function boot(page:Page,query='scenario=seeded&seed=750&view=front&vector=different') {
 await page.clock.install({time:new Date(EPOCH)})
 // Pause ahead of install, which briefly runs in real time; navigation starts
 // only after pausing. This avoids a millisecond race into the past.
 await page.clock.pauseAt(new Date(Date.parse(EPOCH)+1000))
 await page.goto(`/tests/fixture.html?${query}`)
 await expect.poll(()=>page.evaluate(()=>!!window.navigationTest)).toBe(true)
 // Canvas/ResizeObserver and module loading use real asynchronous work. Give
 // each mounted renderer the same warmup after its first completed frame.
 for(let i=0;i<20&&!await page.evaluate(()=>!!window.radarTest);i++)await page.clock.runFor(16)
 await expect.poll(()=>page.evaluate(()=>!!window.radarTest)).toBe(true)
 await page.clock.runFor(160)
 await expect(page.locator('canvas')).toBeVisible()
}
export const snapshot=(page:Page)=>page.evaluate(()=>({navigation:window.navigationTest.snapshot(),scene:window.radarTest.snapshot(),origin:window.radarTest.projectLs([0,0,0])}))
export async function vectors(page:Page) {
 return page.locator('.ship-vectors line').evaluateAll(nodes=>nodes.filter(n=>(n as SVGElement).style.display!=='none').map(n=>({kind:n.getAttribute('data-vector'),start:[Number(n.getAttribute('x1')),Number(n.getAttribute('y1'))],end:[Number(n.getAttribute('x2')),Number(n.getAttribute('y2'))],stroke:getComputedStyle(n).stroke,dash:getComputedStyle(n).strokeDasharray})))
}
export async function evidence(page:Page,info:TestInfo,name:string,value:unknown) {
 const metadata=await page.evaluate(()=>({dpr:devicePixelRatio,epoch:new Date().toISOString(),width:innerWidth,height:innerHeight}))
 await info.attach(`${name}.json`,{body:json({criteria:info.title,metadata,observation:value}),contentType:'application/json'})
 // No animations:'disabled': a capture must not advance this controlled clock.
 await info.attach(`${name}.png`,{body:await page.locator('.radar').screenshot(),contentType:'image/png'})
}
