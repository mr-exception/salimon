import assert from 'node:assert/strict'
import fs from 'node:fs'
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE_URL||'playwright')
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'/usr/bin/chromium',headless:true,args:['--no-sandbox','--use-angle=swiftshader','--enable-unsafe-swiftshader']})
const page=await browser.newPage({viewport:{width:1280,height:800}}),errors=[],results=[]
page.on('pageerror',e=>errors.push(e.message))
try {
 for(const scenario of ['warning','destroyed','safe','asteroid']) {
  await page.goto(`http://localhost:5173/tests/fixture.html?scenario=${scenario}`)
  const expected=scenario==='warning'?'dangerous approach':scenario==='destroyed'?'Journey ended — ship destroyed':scenario==='safe'?'Safe surface contact':'Crash recovery'
  await page.getByText(expected,{exact:false}).waitFor()
  if(scenario==='safe'||scenario==='asteroid')await page.waitForFunction(()=>[...document.querySelectorAll('.stat')].some(e=>e.textContent==='Flight statusgrounded'))
  if(scenario!=='warning')assert.equal(await page.locator('input[type=range]').isDisabled(),true)
  results.push({scenario,stats:await page.locator('.stat').allTextContents(),status:await page.locator('.contact-status,.warnings').allTextContents()})
  await page.screenshot({path:`reports/issue-729/${scenario}.png`})
  if(scenario!=='warning'){
   await page.getByRole('button',{name:'Restart navigation'}).click()
   await page.waitForFunction(()=>[...document.querySelectorAll('.stat')].some(e=>e.textContent==='Flight statusflying'))
   assert.equal(await page.locator('input[type=range]').isDisabled(),false)
  }
 }
 assert.deepEqual(errors,[])
 fs.writeFileSync('reports/issue-729/browser.json',JSON.stringify({fixture:'client/tests/fixture.html (production App/runtime, deterministic injected catalog/state)',viewport:[1280,800],results,errors},null,2)+'\n')
}finally{await browser.close()}
