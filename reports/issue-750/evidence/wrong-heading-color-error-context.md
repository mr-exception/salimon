# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: deterministic.spec.ts >> NAV-3 heading and velocity retain approved colors and viewport edges
- Location: tests/deterministic.spec.ts:77:1

# Error details

```
Error: NAV-3 heading is yellow

expect(received).toBe(expected) // Object.is equality

Expected: "rgb(255, 227, 106)"
Received: "rgb(255, 0, 0)"
```

# Page snapshot

```yaml
- generic [ref=e3]:
  - region "Navigation radar" [ref=e4]:
    - img "Ship-centered three-dimensional radar" [ref=e7]
    - generic: SALIMON 3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE
    - img "Ship heading and velocity vector lines"
    - generic: "HEADING VELOCITY · dashed · Scanner 100,000 light-seconds · pulse #1"
  - complementary [ref=e8]:
    - heading "NAVIGATION" [level=1] [ref=e9]
    - paragraph [ref=e10]: Navigation-only prototype · unlimited energy
    - status [ref=e11]: "Scanner pulse 1: 4 temporary contacts"
    - generic [ref=e12]:
      - generic [ref=e13]: Total ship mass
      - strong [ref=e14]: 100,000 kg
    - generic [ref=e15]:
      - generic [ref=e16]: Relative position
      - strong [ref=e17]: 0.0 / 0.2 / 0.0 m (cell local)
    - generic [ref=e18]:
      - generic [ref=e19]: Speed (sample inertial frame)
      - strong [ref=e20]: 1.0 m/s
    - generic [ref=e21]:
      - generic [ref=e22]: Yaw / pitch
      - strong [ref=e23]: "-90.0° / 0.0°"
    - generic [ref=e24]:
      - generic [ref=e25]: Alignment
      - strong [ref=e26]: Manual
    - generic [ref=e27]:
      - generic [ref=e28]: Flight status
      - strong [ref=e29]: flying
    - status "Navigation warnings" [ref=e30]:
      - paragraph [ref=e31]: No navigation warnings
    - separator [ref=e32]
    - heading "Contact" [level=2] [ref=e33]
    - status [ref=e34]: No selected target
    - generic [ref=e35]: Find scanned contacts
    - searchbox "Find scanned contacts" [ref=e36]
    - generic [ref=e37]: Scanned contacts (4)
    - combobox "Scanned contacts (4)" [ref=e38]:
      - option "Choose a contact" [selected]
      - option "asteroid"
      - option "moon"
      - option "planet"
      - option "star"
    - paragraph [ref=e39]: Select a visible contact or use the scanned-contact list.
    - separator [ref=e40]
    - heading "Camera and orientation" [level=2] [ref=e41]
    - paragraph [ref=e42]: WASD pans the view · wheel zooms · arrow keys turn the ship.
    - generic [ref=e43]:
      - button "Help" [ref=e44] [cursor=pointer]
      - button "Reset camera" [ref=e45] [cursor=pointer]
    - separator [ref=e46]
    - heading "Throttle 0%" [level=2] [ref=e47]:
      - generic [ref=e48]: Throttle
      - strong [ref=e49]: 0%
    - slider "Throttle" [ref=e50]: "0"
    - button "Zero throttle (Space)" [ref=e51] [cursor=pointer]
    - paragraph [ref=e52]: Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run. Lower-speed contact settles the ship.
```

# Test source

```ts
  1   | import {test,expect} from '@playwright/test'
  2   | import {boot,snapshot,vectors,evidence,json} from './deterministic'
  3   |
  4   | test('NAV-1 seeded replay: exact WASD direction, release and wheel zoom',async({page},info)=>{
  5   |  const replays=[]
  6   |  for(let replay=0;replay<2;replay++) {
  7   |   await boot(page,'scenario=seeded&seed=750&view=front&vector=zero')
  8   |   const start=await snapshot(page),samples=[]
  9   |   await expect(page.getByRole('button',{name:/^(Pitch up|Pitch down|Yaw left|Yaw right|Zoom in|Zoom out)$/})).toHaveCount(0)
  10  |   for(const [key,axis,sign] of [['w',1,1],['s',1,-1],['a',0,1],['d',0,-1]] as const) {
  11  |    const before=await snapshot(page)
  12  |    await page.keyboard.down(key);await page.clock.runFor(160);await page.keyboard.up(key)
  13  |    const after=await snapshot(page)
  14  |    expect((after.origin[axis]-before.origin[axis])*sign,`NAV-1 ${key} moves the correct screen direction`).toBeGreaterThan(1)
  15  |    expect(after.navigation.ship.yaw).toBe(start.navigation.ship.yaw)
  16  |    expect(after.navigation.ship.pitch).toBe(start.navigation.ship.pitch)
  17  |    expect(after.navigation.ship.velocityMps).toEqual(start.navigation.ship.velocityMps)
  18  |    expect(after.navigation.ship.position).toEqual(start.navigation.ship.position)
  19  |    await page.clock.runFor(32)
  20  |    expect((await snapshot(page)).origin,'NAV-1 release stops pan').toEqual(after.origin)
  21  |    samples.push({key,origin:after.origin})
  22  |   }
  23  |   await page.mouse.move(400,350)
  24  |   await page.mouse.wheel(0,-400);await page.clock.runFor(160)
  25  |   const zoomed=await snapshot(page)
  26  |   expect(zoomed.scene.zoom,'NAV-1 wheel up zooms in').toBeGreaterThan(start.scene.zoom)
  27  |   await page.mouse.wheel(0,400);await page.clock.runFor(160)
  28  |   expect((await snapshot(page)).scene.zoom).toBeCloseTo(start.scene.zoom,10)
  29  |   replays.push({catalog:start.navigation.catalog,samples,zoom:zoomed.scene.zoom,ticks:(await snapshot(page)).navigation.ship.tick-start.navigation.ship.tick})
  30  |  }
  31  |  expect(replays[1],'NAV-1 identical seed and input clock replay').toEqual(replays[0])
  32  |  await evidence(page,info,'seeded-replay',replays)
  33  |  await boot(page,'scenario=seeded&seed=751&view=front&vector=zero')
  34  |  expect((await snapshot(page)).navigation.catalog).not.toEqual(replays[0].catalog)
  35  | })
  36  |
  37  | test('NAV-2 fixed-clock scan: no discovery before tick 300, stable selection during continuous pan',async({page},info)=>{
  38  |  await boot(page,'scenario=seeded-empty&seed=750&view=front&vector=zero')
  39  |  await expect(page.locator('.scan-status')).toContainText('awaiting first')
  40  |  expect((await snapshot(page)).navigation.pulse).toBe(0)
  41  |  // Tick is authoritative: stop short of the first pulse, then cross it.
  42  |  while((await snapshot(page)).navigation.ship.tick<295)await page.clock.runFor(16)
  43  |  expect((await snapshot(page)).navigation.pulse).toBe(0)
  44  |  await expect(page.locator('#contact-select option')).toHaveCount(1)
  45  |  await page.clock.runFor(160)
  46  |  await expect(page.locator('.scan-status')).toContainText('4 temporary contacts')
  47  |  await page.getByLabel('Scanned contacts (4)',{exact:true}).selectOption('planet')
  48  |  await page.getByRole('region',{name:'Navigation radar',exact:true}).focus()
  49  |  const recordings=[]
  50  |  for(const input of ['keyboard','mouse']) {
  51  |   const recording=page.evaluate(()=>window.radarTest.recordFrames(20))
  52  |   if(input==='keyboard')await page.keyboard.down('d')
  53  |   else {await page.mouse.move(500,350);await page.mouse.down({button:'right'});await page.mouse.move(540,360,{steps:4})}
  54  |   await page.clock.runFor(400)
  55  |   const data=await recording
  56  |   if(input==='keyboard')await page.keyboard.up('d');else await page.mouse.up({button:'right'})
  57  |   const bodies=data.frames.flatMap(f=>f.bodies)
  58  |   expect(bodies.length,'NAV-2 nonvacuous in-view body evidence').toBeGreaterThan(20)
  59  |   expect(bodies.every(b=>b.present&&b.bright>=10),'NAV-2 bodies remain rendered during pan').toBe(true)
  60  |   for(const b of bodies){expect(b.actualPixel![0]).toBeCloseTo(b.pixel[0],5);expect(b.actualPixel![1]).toBeCloseTo(b.pixel[1],5)}
  61  |   expect(data.frames.every(f=>f.ship!==null)).toBe(true)
  62  |   for(const frame of data.frames) {
  63  |    const heading=frame.vectors.find(v=>v.kind==='heading')!
  64  |    expect(heading.display,'NAV-2 heading remains visible every frame').not.toBe('none')
  65  |    expect(heading.start[0]).toBeCloseTo(frame.ship![0]/970*100,5)
  66  |    expect(heading.start[1]).toBeCloseTo(frame.ship![1]/800*100,5)
  67  |    expect(heading.end[0]).toBe(100)
  68  |    expect(heading.end[1]).toBeCloseTo(heading.start[1],5)
  69  |   }
  70  |   expect((await snapshot(page)).navigation.selected).toBe('planet')
  71  |   for(const [kind,url] of Object.entries(data.captures))await info.attach(`${input}-${kind}.png`,{body:Buffer.from(url.split(',')[1],'base64'),contentType:'image/png'})
  72  |   recordings.push({input,frames:data.frames,renderer:data.renderer})
  73  |  }
  74  |  await evidence(page,info,'continuous-pan',recordings)
  75  | })
  76  |
  77  | test('NAV-3 heading and velocity retain approved colors and viewport edges',async({page},info)=>{
  78  |  await boot(page)
  79  |  const lines=await vectors(page)
  80  |  expect(lines.map(l=>l.kind)).toEqual(['heading','velocity'])
> 81  |  expect(lines[0].stroke,'NAV-3 heading is yellow').toBe('rgb(255, 227, 106)')
      |                                                    ^ Error: NAV-3 heading is yellow
  82  |  expect(lines[1].stroke,'NAV-3 velocity is light-green').toBe('rgb(182, 245, 162)')
  83  |  expect(lines[1].dash).not.toBe('none')
  84  |  expect(lines[0].end[0]).toBe(100);expect(lines[1].end[1]).toBe(0)
  85  |  await expect(page.locator('.direction-lines,[data-offscreen],[marker-start],[marker-end]')).toHaveCount(0)
  86  |  expect((await snapshot(page)).scene.arrows).toBe(0)
  87  |  for(const line of lines){expect(line.start).toEqual([50,50]);expect([...line.start,...line.end].every(n=>Number.isFinite(n)&&n>=0&&n<=100)).toBe(true)}
  88  |  await evidence(page,info,'different-vectors',{lines,state:await snapshot(page)})
  89  |  await boot(page,'scenario=seeded&seed=750&view=front&vector=zero')
  90  |  expect((await vectors(page)).map(l=>l.kind)).toEqual(['heading'])
  91  |  await evidence(page,info,'stationary',{lines:await vectors(page),state:await snapshot(page)})
  92  | })
  93  |
  94  | test('NAV-4 read-only fixture observers return detached state; production has no hooks',async({page,request},info)=>{
  95  |  await boot(page)
  96  |  const before=await snapshot(page)
  97  |  const keys=await page.evaluate(()=>{
  98  |   const state=window.navigationTest.snapshot();Object.assign(state.ship.velocityMps,{0:999});state.catalog.length=0
  99  |   const scene=window.radarTest.snapshot();scene.objects.length=0
  100 |   return {navigation:Object.keys(window.navigationTest),radar:Object.keys(window.radarTest)}
  101 |  })
  102 |  expect(await snapshot(page)).toEqual(before)
  103 |  expect(keys.navigation.sort()).toEqual(['positions','snapshot'])
  104 |  expect(keys.radar.sort()).toEqual(['projectLs','recordFrames','snapshot'])
  105 |  await info.attach('observer-contract.json',{body:json(keys),contentType:'application/json'})
  106 |  await page.goto('http://127.0.0.1:4175/')
  107 |  await expect(page.locator('canvas')).toBeVisible()
  108 |  expect(await page.evaluate(()=>({navigation:'navigationTest' in window,radar:'radarTest' in window}))).toEqual({navigation:false,radar:false})
  109 |  const response=await request.get('http://127.0.0.1:4175/tests/fixture.html')
  110 |  expect(await response.text()).not.toContain('/tests/fixture.tsx')
  111 | })
  112 |
```
