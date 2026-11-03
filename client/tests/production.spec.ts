import {test,expect} from '@playwright/test'
test('built production bundle renders WebGL, scanner and labelled controls',async({page},testInfo)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('http://127.0.0.1:4175/')
 await expect(page.getByRole('img',{name:'Ship-centered three-dimensional radar',exact:true})).toBeVisible()
 await expect(page.locator('.scan-status')).toContainText('5 temporary contacts',{timeout:15000})
 await expect(page.getByLabel('Throttle',{exact:true})).toHaveValue('0');await expect(page.locator('.target-status')).toHaveText('No selected target');await expect(page.getByRole('status',{name:'Navigation warnings'})).toContainText('No navigation warnings')
 const gpu=await page.locator('canvas').evaluate(element=>{const c=element as HTMLCanvasElement;const gl=c.getContext('webgl2')!;return {width:c.width,height:c.height,version:gl.getParameter(gl.VERSION),renderer:gl.getParameter(gl.getExtension('WEBGL_debug_renderer_info')!.UNMASKED_RENDERER_WEBGL)}})
 expect(gpu.width).toBeGreaterThan(0);expect(gpu.renderer).toContain('SwiftShader');expect(errors).toEqual([])
 await testInfo.attach('backend.json',{body:JSON.stringify({gpu,errors,viewport:page.viewportSize()},null,2),contentType:'application/json'})
})
