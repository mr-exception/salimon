import {test,expect} from '@playwright/test'
import {readFile,writeFile,mkdir} from 'node:fs/promises'
import {execFileSync} from 'node:child_process'
import {join} from 'node:path'
import {boot,snapshot,json,EPOCH} from './deterministic'
import {POLICY,sha256,compareImages} from '../scripts/visual-comparison.ts'

const directory='tests/navigation-baselines'
const candidate=process.env.SALIMON_CAPTURE_CANDIDATES
if(candidate&&process.env.CI)throw Error('Candidate generation is forbidden in CI')
const cases=[
 {id:'empty-scanner',scenario:'seeded-empty',vector:'zero'},
 {id:'visible-contacts'},
 {id:'panned-contacts',pan:true},
 {id:'selected-body',selected:true},
 {id:'zoom-out',zoom:.1},
 {id:'zoom-in',zoom:1},
 {id:'heading-velocity',vector:'different'},
 {id:'opposite-vectors',vector:'opposite'}
] as const
// One ordered session preserves a single manifest and complete comparison summary.
test('VIS-1/2 eight navigation screens match reference pixels',async({page,browser},info)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message))
 const fonts=await Promise.all(['DejaVuSans.ttf','DejaVuSans-Bold.ttf','fonts.css'].map(async f=>[f,sha256(await readFile(`tests/assets/${f}`))]))
 const source=(await readFile('.revision-for-negative','utf8').catch(()=>null))??execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim()
 const environment={platform:process.platform,arch:process.arch,browser:browser.version(),playwright:JSON.parse(await readFile('node_modules/@playwright/test/package.json','utf8')).version,viewport:{width:1280,height:800},dpr:1,locale:'en-US',timezone:'UTC',colorScheme:'dark',fontHashes:Object.fromEntries(fonts),seed:750,epoch:EPOCH,policy:POLICY}
 const manifest=candidate?null:JSON.parse(await readFile(`${directory}/manifest.json`,'utf8'))
 const results:Record<string,unknown>[]=[],images:Record<string,string>={}
 const output=candidate??info.outputPath('comparisons')
 await mkdir(output,{recursive:true})
 // Drift is a failure, never an instruction to regenerate references.
 if(manifest)expect(environment,'VISUAL_ENVIRONMENT_DRIFT: propose separately reviewed candidates').toEqual(manifest.environment)
 for(const fixture of cases) {
  const c:{id:string;scenario?:string;vector?:string;pan?:boolean;selected?:boolean;zoom?:number}=fixture
  await boot(page,`scenario=${c.scenario??'seeded'}&seed=750&view=front&vector=${c.vector??'zero'}&zoom=${c.zoom??.5}`)
  await page.evaluate(()=>document.fonts.ready)
  if(c.selected)await page.getByLabel('Scanned contacts (4)',{exact:true}).selectOption('moon')
  if(c.pan){await page.keyboard.down('d');await page.clock.runFor(160);await page.keyboard.up('d')}
  // Deterministic completed RAF; no screenshot animation fast-forward or polling
  // against mutating frames. Font loading completes before this last frame.
  await page.clock.runFor(32)
  await page.mouse.move(1279,799)
  const state=await snapshot(page)
  expect(state.scene.renderer,'VISUAL_BACKEND').toContain('SwiftShader')
  if(manifest)expect(state.scene.renderer,'VISUAL_RENDERER_DRIFT').toBe(manifest.renderer)
  if(c.scenario==='seeded-empty')expect(state.navigation.pulse).toBe(0)
  else expect(state.scene.objects.filter(o=>o.name.startsWith('nav-body:')).length,'VISUAL_NONVACUOUS_CONTACTS').toBeGreaterThan(0)
  if(c.selected)expect(state.navigation.selected).toBe('moon')
  const actual=await page.screenshot({caret:'hide'})
  const name=`${c.id}.png`;images[name]=sha256(actual)
  await writeFile(join(output,name),actual)
  await writeFile(join(output,`${c.id}.json`),json({source,environment,fixture,state,errors}))
  await info.attach(name,{body:actual,contentType:'image/png'})
  if(candidate){results.push({id:c.id,status:'CANDIDATE',sha256:sha256(actual)});continue}
  const original=await readFile(join(directory,name))
  expect(sha256(original),'BASELINE_HASH_MISMATCH').toBe(manifest.images[name])
  const comparison=compareImages(original,actual)
  await writeFile(join(output,`${c.id}-expected.png`),original)
  await writeFile(join(output,`${c.id}-diff.png`),comparison.diff)
  await info.attach(`${c.id}-diff.png`,{body:comparison.diff,contentType:'image/png'})
  const {diff:_,...measurement}=comparison
  results.push({id:c.id,...measurement,expected:sha256(original),actual:sha256(actual)})
 }
 expect(errors,'VISUAL_PAGE_ERRORS').toEqual([])
 if(candidate) {
  const renderer=(await snapshot(page)).scene.renderer
  await writeFile(join(output,'manifest.json'),json({schema:1,environment,renderer,images})+'\n')
 }
 await writeFile(join(output,'summary.json'),json({source,candidate:!!candidate,results})+'\n')
 await info.attach('comparison-summary.json',{body:json({source,candidate:!!candidate,results}),contentType:'application/json'})
 // Soft per-frame failures let QA inspect every unexpected screen in one run.
 for(const r of results)if(!candidate)expect.soft(r.pass,`VISUAL_PIXEL_MISMATCH ${r.id}: ${r.changedPixels} changed pixels`).toBe(true)
})
