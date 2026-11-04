import {test,expect} from '@playwright/test'
import {boot,snapshot,vectors,evidence,json} from './deterministic'

test('NAV-1 seeded replay: exact WASD direction, release and wheel zoom',async({page},info)=>{
 const replays=[]
 for(let replay=0;replay<2;replay++) {
  await boot(page,'scenario=seeded&seed=750&view=front&vector=zero')
  const start=await snapshot(page),samples=[]
  await expect(page.getByRole('button',{name:/^(Pitch up|Pitch down|Yaw left|Yaw right|Zoom in|Zoom out)$/})).toHaveCount(0)
  for(const [key,axis,sign] of [['w',1,1],['s',1,-1],['a',0,1],['d',0,-1]] as const) {
   const before=await snapshot(page)
   await page.keyboard.down(key);await page.clock.runFor(160);await page.keyboard.up(key)
   const after=await snapshot(page)
   expect((after.origin[axis]-before.origin[axis])*sign,`NAV-1 ${key} moves the correct screen direction`).toBeGreaterThan(1)
   expect(after.navigation.ship.yaw).toBe(start.navigation.ship.yaw)
   expect(after.navigation.ship.pitch).toBe(start.navigation.ship.pitch)
   expect(after.navigation.ship.velocityMps).toEqual(start.navigation.ship.velocityMps)
   expect(after.navigation.ship.position).toEqual(start.navigation.ship.position)
   await page.clock.runFor(32)
   expect((await snapshot(page)).origin,'NAV-1 release stops pan').toEqual(after.origin)
   samples.push({key,origin:after.origin})
  }
  await page.mouse.move(400,350)
  await page.mouse.wheel(0,-400);await page.clock.runFor(160)
  const zoomed=await snapshot(page)
  expect(zoomed.scene.zoom,'NAV-1 wheel up zooms in').toBeGreaterThan(start.scene.zoom)
  await page.mouse.wheel(0,400);await page.clock.runFor(160)
  expect((await snapshot(page)).scene.zoom).toBeCloseTo(start.scene.zoom,10)
  replays.push({catalog:start.navigation.catalog,samples,zoom:zoomed.scene.zoom,ticks:(await snapshot(page)).navigation.ship.tick-start.navigation.ship.tick})
 }
 expect(replays[1],'NAV-1 identical seed and input clock replay').toEqual(replays[0])
 await evidence(page,info,'seeded-replay',replays)
 await boot(page,'scenario=seeded&seed=751&view=front&vector=zero')
 expect((await snapshot(page)).navigation.catalog).not.toEqual(replays[0].catalog)
})

test('NAV-2 fixed-clock scan: no discovery before tick 300, stable selection during continuous pan',async({page},info)=>{
 await boot(page,'scenario=seeded-empty&seed=750&view=front&vector=zero')
 await expect(page.locator('.scan-status')).toContainText('awaiting first')
 expect((await snapshot(page)).navigation.pulse).toBe(0)
 // Tick is authoritative: stop short of the first pulse, then cross it.
 while((await snapshot(page)).navigation.ship.tick<295)await page.clock.runFor(16)
 expect((await snapshot(page)).navigation.pulse).toBe(0)
 await expect(page.locator('#contact-select option')).toHaveCount(1)
 await page.clock.runFor(160)
 await expect(page.locator('.scan-status')).toContainText('4 temporary contacts')
 await page.getByLabel('Scanned contacts (4)',{exact:true}).selectOption('planet')
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus()
 const recordings=[]
 for(const input of ['keyboard','mouse']) {
  const recording=page.evaluate(()=>window.radarTest.recordFrames(20))
  if(input==='keyboard')await page.keyboard.down('d')
  else {await page.mouse.move(500,350);await page.mouse.down({button:'right'});await page.mouse.move(540,360,{steps:4})}
  await page.clock.runFor(400)
  const data=await recording
  if(input==='keyboard')await page.keyboard.up('d');else await page.mouse.up({button:'right'})
  const bodies=data.frames.flatMap(f=>f.bodies)
  expect(bodies.length,'NAV-2 nonvacuous in-view body evidence').toBeGreaterThan(20)
  expect(bodies.every(b=>b.present&&b.bright>=10),'NAV-2 bodies remain rendered during pan').toBe(true)
  for(const b of bodies){expect(b.actualPixel![0]).toBeCloseTo(b.pixel[0],5);expect(b.actualPixel![1]).toBeCloseTo(b.pixel[1],5)}
  expect(data.frames.every(f=>f.ship!==null)).toBe(true)
  for(const frame of data.frames) {
   const heading=frame.vectors.find(v=>v.kind==='heading')!
   expect(heading.display,'NAV-2 heading remains visible every frame').not.toBe('none')
   expect(heading.start[0]).toBeCloseTo(frame.ship![0]/970*100,5)
   expect(heading.start[1]).toBeCloseTo(frame.ship![1]/800*100,5)
   expect(heading.end[0]).toBe(100)
   expect(heading.end[1]).toBeCloseTo(heading.start[1],5)
  }
  expect((await snapshot(page)).navigation.selected).toBe('planet')
  for(const [kind,url] of Object.entries(data.captures))await info.attach(`${input}-${kind}.png`,{body:Buffer.from(url.split(',')[1],'base64'),contentType:'image/png'})
  recordings.push({input,frames:data.frames,renderer:data.renderer})
 }
 await evidence(page,info,'continuous-pan',recordings)
})

test('NAV-3 heading and velocity retain approved colors and viewport edges',async({page},info)=>{
 await boot(page)
 const lines=await vectors(page)
 expect(lines.map(l=>l.kind)).toEqual(['heading','velocity'])
 expect(lines[0].stroke,'NAV-3 heading is yellow').toBe('rgb(255, 227, 106)')
 expect(lines[1].stroke,'NAV-3 velocity is light-green').toBe('rgb(182, 245, 162)')
 expect(lines[1].dash).not.toBe('none')
 expect(lines[0].end[0]).toBe(100);expect(lines[1].end[1]).toBe(0)
 await expect(page.locator('.direction-lines,[data-offscreen],[marker-start],[marker-end]')).toHaveCount(0)
 expect((await snapshot(page)).scene.arrows).toBe(0)
 for(const line of lines){expect(line.start).toEqual([50,50]);expect([...line.start,...line.end].every(n=>Number.isFinite(n)&&n>=0&&n<=100)).toBe(true)}
 await evidence(page,info,'different-vectors',{lines,state:await snapshot(page)})
 await boot(page,'scenario=seeded&seed=750&view=front&vector=zero')
 expect((await vectors(page)).map(l=>l.kind)).toEqual(['heading'])
 await evidence(page,info,'stationary',{lines:await vectors(page),state:await snapshot(page)})
})

test('NAV-4 read-only fixture observers return detached state; production has no hooks',async({page,request},info)=>{
 await boot(page)
 const before=await snapshot(page)
 const keys=await page.evaluate(()=>{
  const state=window.navigationTest.snapshot();Object.assign(state.ship.velocityMps,{0:999});state.catalog.length=0
  const scene=window.radarTest.snapshot();scene.objects.length=0
  return {navigation:Object.keys(window.navigationTest),radar:Object.keys(window.radarTest)}
 })
 expect(await snapshot(page)).toEqual(before)
 expect(keys.navigation.sort()).toEqual(['positions','snapshot'])
 expect(keys.radar.sort()).toEqual(['projectLs','recordFrames','snapshot'])
 await info.attach('observer-contract.json',{body:json(keys),contentType:'application/json'})
 await page.goto('http://127.0.0.1:4175/')
 await expect(page.locator('canvas')).toBeVisible()
 expect(await page.evaluate(()=>({navigation:'navigationTest' in window,radar:'radarTest' in window}))).toEqual({navigation:false,radar:false})
 const response=await request.get('http://127.0.0.1:4175/tests/fixture.html')
 expect(await response.text()).not.toContain('/tests/fixture.tsx')
})
