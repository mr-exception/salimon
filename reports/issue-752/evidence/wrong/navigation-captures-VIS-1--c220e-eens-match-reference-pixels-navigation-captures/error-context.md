# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: navigation-captures.spec.ts >> VIS-1/2 eight navigation screens match reference pixels
- Location: tests/navigation-captures.spec.ts:22:1

# Error details

```
Error: VISUAL_PIXEL_MISMATCH heading-velocity: 520 changed pixels

expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false
```

```
Error: VISUAL_PIXEL_MISMATCH opposite-vectors: 630 changed pixels

expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false
```

# Page snapshot

```yaml
- generic [ref=f7e3]:
  - region "Navigation radar" [ref=f7e4]:
    - img "Ship-centered three-dimensional radar" [ref=f7e7]
    - generic: SALIMON 3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE
    - img "Ship heading and velocity vector lines"
    - generic: "HEADING VELOCITY · dashed · Scanner 100,000 light-seconds · pulse #1"
  - complementary [ref=f7e8]:
    - heading "NAVIGATION" [level=1] [ref=f7e9]
    - paragraph [ref=f7e10]: Navigation-only prototype · unlimited energy
    - status [ref=f7e11]: "Scanner pulse 1: 4 temporary contacts"
    - generic [ref=f7e12]:
      - generic [ref=f7e13]: Total ship mass
      - strong [ref=f7e14]: 100,000 kg
    - generic [ref=f7e15]:
      - generic [ref=f7e16]: Relative position
      - strong [ref=f7e17]: "-0.2 / 0.0 / 0.0 m (cell local)"
    - generic [ref=f7e18]:
      - generic [ref=f7e19]: Speed (sample inertial frame)
      - strong [ref=f7e20]: 1.0 m/s
    - generic [ref=f7e21]:
      - generic [ref=f7e22]: Yaw / pitch
      - strong [ref=f7e23]: "-90.0° / 0.0°"
    - generic [ref=f7e24]:
      - generic [ref=f7e25]: Alignment
      - strong [ref=f7e26]: Manual
    - generic [ref=f7e27]:
      - generic [ref=f7e28]: Flight status
      - strong [ref=f7e29]: flying
    - status "Navigation warnings" [ref=f7e30]:
      - paragraph [ref=f7e31]: No navigation warnings
    - separator [ref=f7e32]
    - heading "Contact" [level=2] [ref=f7e33]
    - status [ref=f7e34]: No selected target
    - generic [ref=f7e35]: Find scanned contacts
    - searchbox "Find scanned contacts" [ref=f7e36]
    - generic [ref=f7e37]: Scanned contacts (4)
    - combobox "Scanned contacts (4)" [ref=f7e38]:
      - option "Choose a contact" [selected]
      - option "asteroid"
      - option "moon"
      - option "planet"
      - option "star"
    - paragraph [ref=f7e39]: Select a visible contact or use the scanned-contact list.
    - separator [ref=f7e40]
    - heading "Camera and orientation" [level=2] [ref=f7e41]
    - paragraph [ref=f7e42]: WASD pans the view · wheel zooms · arrow keys turn the ship.
    - generic [ref=f7e43]:
      - button "Help" [ref=f7e44] [cursor=pointer]
      - button "Reset camera" [ref=f7e45] [cursor=pointer]
    - separator [ref=f7e46]
    - heading "Throttle 0%" [level=2] [ref=f7e47]:
      - generic [ref=f7e48]: Throttle
      - strong [ref=f7e49]: 0%
    - slider "Throttle" [ref=f7e50]: "0"
    - button "Zero throttle (Space)" [ref=f7e51] [cursor=pointer]
    - paragraph [ref=f7e52]: Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run. Lower-speed contact settles the ship.
```

# Test source

```ts
  1  | import {test,expect} from '@playwright/test'
  2  | import {readFile,writeFile,mkdir} from 'node:fs/promises'
  3  | import {execFileSync} from 'node:child_process'
  4  | import {join} from 'node:path'
  5  | import {boot,snapshot,json,EPOCH} from './deterministic'
  6  | import {POLICY,sha256,compareImages} from '../scripts/visual-comparison.ts'
  7  | 
  8  | const directory='tests/navigation-baselines'
  9  | const candidate=process.env.SALIMON_CAPTURE_CANDIDATES||undefined
  10 | if(candidate&&process.env.CI)throw Error('Candidate generation is forbidden in CI')
  11 | const cases=[
  12 |  {id:'empty-scanner',scenario:'seeded-empty',vector:'zero'},
  13 |  {id:'visible-contacts'},
  14 |  {id:'panned-contacts',pan:true},
  15 |  {id:'selected-body',selected:true},
  16 |  {id:'zoom-out',zoom:.1},
  17 |  {id:'zoom-in',zoom:1},
  18 |  {id:'heading-velocity',vector:'different'},
  19 |  {id:'opposite-vectors',vector:'opposite'}
  20 | ] as const
  21 | // One ordered session preserves a single manifest and complete comparison summary.
  22 | test('VIS-1/2 eight navigation screens match reference pixels',async({page,browser},info)=>{
  23 |  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message))
  24 |  const fonts=await Promise.all(['DejaVuSans.ttf','DejaVuSans-Bold.ttf','fonts.css','fonts.conf'].map(async f=>[f,sha256(await readFile(`tests/assets/${f}`))]))
  25 |  const launchArgs=info.project.use.launchOptions?.args??[]
  26 |  expect(launchArgs,'VISUAL_RASTERIZATION_POLICY').toContain('--disable-lcd-text')
  27 |  expect(info.project.use.launchOptions?.env?.FONTCONFIG_FILE,'VISUAL_FONTCONFIG_POLICY').toMatch(/tests\/assets\/fonts\.conf$/)
  28 |  const source=(await readFile('.revision-for-negative','utf8').catch(()=>null))??execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim()
  29 |  const environment={platform:process.platform,arch:process.arch,browser:browser.version(),playwright:JSON.parse(await readFile('node_modules/@playwright/test/package.json','utf8')).version,viewport:{width:1280,height:800},dpr:1,locale:'en-US',timezone:'UTC',colorScheme:'dark',fontHashes:Object.fromEntries(fonts),seed:750,epoch:EPOCH,nativeFallback:'vendored DejaVu Sans only',policy:POLICY}
  30 |  const manifest=candidate?null:JSON.parse(await readFile(`${directory}/manifest.json`,'utf8'))
  31 |  const results:Record<string,unknown>[]=[],images:Record<string,string>={}
  32 |  const output=candidate??info.outputPath('comparisons')
  33 |  await mkdir(output,{recursive:true})
  34 |  // Drift is a failure, never an instruction to regenerate references.
  35 |  if(manifest)expect(environment,'VISUAL_ENVIRONMENT_DRIFT: propose separately reviewed candidates').toEqual(manifest.environment)
  36 |  for(const fixture of cases) {
  37 |   const c:{id:string;scenario?:string;vector?:string;pan?:boolean;selected?:boolean;zoom?:number}=fixture
  38 |   await boot(page,`scenario=${c.scenario??'seeded'}&seed=750&view=front&vector=${c.vector??'zero'}&zoom=${c.zoom??.5}`)
  39 |   await page.evaluate(()=>document.fonts.ready)
  40 |   if(c.selected)await page.getByLabel('Scanned contacts (4)',{exact:true}).selectOption('moon')
  41 |   if(c.pan){await page.keyboard.down('d');await page.clock.runFor(160);await page.keyboard.up('d')}
  42 |   // Deterministic completed RAF; no screenshot animation fast-forward or polling
  43 |   // against mutating frames. Font loading completes before this last frame.
  44 |   await page.clock.runFor(32)
  45 |   await page.mouse.move(1279,799)
  46 |   const state=await snapshot(page)
  47 |   expect(state.scene.renderer,'VISUAL_BACKEND').toContain('SwiftShader')
  48 |   if(manifest)expect(state.scene.renderer,'VISUAL_RENDERER_DRIFT').toBe(manifest.renderer)
  49 |   if(c.scenario==='seeded-empty')expect(state.navigation.pulse).toBe(0)
  50 |   else expect(state.scene.objects.filter(o=>o.name.startsWith('nav-body:')).length,'VISUAL_NONVACUOUS_CONTACTS').toBeGreaterThan(0)
  51 |   if(c.selected)expect(state.navigation.selected).toBe('moon')
  52 |   const actual=await page.screenshot({caret:'hide'})
  53 |   const name=`${c.id}.png`;images[name]=sha256(actual)
  54 |   await writeFile(join(output,name),actual)
  55 |   await writeFile(join(output,`${c.id}.json`),json({source,environment,launchArgs,fixture,state,errors}))
  56 |   await info.attach(name,{body:actual,contentType:'image/png'})
  57 |   if(candidate){results.push({id:c.id,status:'CANDIDATE',sha256:sha256(actual)});continue}
  58 |   const original=await readFile(join(directory,name))
  59 |   expect(sha256(original),'BASELINE_HASH_MISMATCH').toBe(manifest.images[name])
  60 |   const comparison=compareImages(original,actual)
  61 |   await writeFile(join(output,`${c.id}-expected.png`),original)
  62 |   await writeFile(join(output,`${c.id}-diff.png`),comparison.diff)
  63 |   await info.attach(`${c.id}-diff.png`,{body:comparison.diff,contentType:'image/png'})
  64 |   const {diff:_,...measurement}=comparison
  65 |   results.push({id:c.id,...measurement,expected:sha256(original),actual:sha256(actual)})
  66 |  }
  67 |  expect(errors,'VISUAL_PAGE_ERRORS').toEqual([])
  68 |  if(candidate) {
  69 |   const renderer=(await snapshot(page)).scene.renderer
  70 |   await writeFile(join(output,'manifest.json'),json({schema:1,environment,renderer,images})+'\n')
  71 |  }
  72 |  await writeFile(join(output,'summary.json'),json({source,launchArgs,candidate:!!candidate,results})+'\n')
  73 |  await info.attach('comparison-summary.json',{body:json({source,launchArgs,candidate:!!candidate,results}),contentType:'application/json'})
  74 |  // Soft per-frame failures let QA inspect every unexpected screen in one run.
> 75 |  for(const r of results)if(!candidate)expect.soft(r.pass,`VISUAL_PIXEL_MISMATCH ${r.id}: ${r.changedPixels} changed pixels`).toBe(true)
     |                                                                                                                              ^ Error: VISUAL_PIXEL_MISMATCH opposite-vectors: 630 changed pixels
  76 | })
  77 | 
```