import {createHash} from 'node:crypto'
import {PNG} from 'pngjs'
import pixelmatch from 'pixelmatch'

// No masks. Reject every pixel beyond the perceptual color threshold, including
// antialiasing differences. A single changed line cannot hide in a global ratio.
export const POLICY={threshold:0.1,maxDiffPixels:0,includeAA:true,masks:[]} as const
export const sha256=(data:Buffer|string)=>createHash('sha256').update(data).digest('hex')
export function compareImages(expected:Buffer,actual:Buffer) {
 const a=PNG.sync.read(expected),b=PNG.sync.read(actual)
 if(a.width!==b.width||a.height!==b.height)throw Error(`VISUAL_DIMENSIONS: ${a.width}x${a.height} vs ${b.width}x${b.height}`)
 const diff=new PNG({width:a.width,height:a.height})
 const changedPixels=pixelmatch(a.data,b.data,diff.data,a.width,a.height,{threshold:POLICY.threshold,includeAA:POLICY.includeAA})
 let rawChangedPixels=0
 for(let i=0;i<a.data.length;i+=4)if(!a.data.subarray(i,i+4).equals(b.data.subarray(i,i+4)))rawChangedPixels++
 return {pass:changedPixels<=POLICY.maxDiffPixels,changedPixels,rawChangedPixels,totalPixels:a.width*a.height,diff:PNG.sync.write(diff)}
}
export function verifyApproval(comments:{body?:string|null;user?:{type?:string}|null;author_association?:string}[],digest:string) {
 return comments.some(c=>c.body?.trim()===`Approve navigation baselines: ${digest}`&&c.user?.type==='User'&&['OWNER','MEMBER','COLLABORATOR'].includes(c.author_association??''))
}
