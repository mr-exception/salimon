import fs from 'node:fs'
import assert from 'node:assert/strict'
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE_URL||'../../client/node_modules/playwright/index.mjs')
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'/usr/bin/chromium',headless:true,args:['--no-sandbox','--use-angle=swiftshader','--enable-unsafe-swiftshader']})
const evidence=[]
try{
 for(const [name,width,height] of [['desktop',1280,800],['mobile',390,844]]){
  const page=await browser.newPage({viewport:{width,height},isMobile:name==='mobile',hasTouch:name==='mobile'})
  const errors=[];page.on('pageerror',e=>errors.push(e.message))
  await page.goto(process.env.NAVIGATION_URL||'http://127.0.0.1:4173')
  await page.waitForFunction(()=>document.querySelector('#contact-select')?.options.length===6)
  await page.getByLabel('Scanned contacts (5)',{exact:true}).selectOption('vesper')
  if(name==='desktop')for(let i=0;i<3;i++)await page.getByRole('button',{name:'Zoom in',exact:true}).click()
  await page.waitForTimeout(300)
  assert.deepEqual(errors,[])
  const gpu=await page.locator('canvas').evaluate(c=>{const gl=c.getContext('webgl2');return gl.getParameter(gl.getExtension('WEBGL_debug_renderer_info').UNMASKED_RENDERER_WEBGL)})
  if(name==='desktop')await page.locator('aside').evaluate(el=>el.scrollTop=0)
  if(name==='mobile')await page.evaluate(()=>scrollTo(0,0))
  await page.screenshot({path:`reports/issue-730/${name}.png`,fullPage:name==='mobile'})
  evidence.push({name,viewport:[width,height],gpu,errors,selected:await page.locator('.target-status').textContent(),scan:await page.locator('.scan-status').textContent()})
  await page.close()
 }
 fs.writeFileSync('reports/issue-730/backend.json',JSON.stringify(evidence,null,2)+'\n')
}finally{await browser.close()}
