import {test} from 'node:test'
import assert from 'node:assert/strict'
import {PNG} from 'pngjs'
import {compareImages,verifyApproval} from './visual-comparison.ts'

test('VIS-1 comparison passes identical images and detects a changed frame with a retained diff',()=>{
 const a=new PNG({width:128,height:80});a.data.fill(255)
 const original=PNG.sync.write(a)
 assert.equal(compareImages(original,original).changedPixels,0)
 for(let y=20;y<40;y++)for(let x=30;x<70;x++)a.data.set([255,0,0,255],(y*128+x)*4)
 const result=compareImages(original,PNG.sync.write(a))
 assert.equal(result.pass,false);assert.equal(result.changedPixels,800)
 assert.equal(PNG.sync.read(result.diff).width,128)
 assert.throws(()=>compareImages(original,PNG.sync.write(new PNG({width:1,height:1}))),/VISUAL_DIMENSIONS/)
 assert.throws(()=>compareImages(original,Buffer.from('invalid PNG')))
})
test('VIS-3 approval is exact, digest-bound and requires a human collaborator',()=>{
 const comment={body:'Approve navigation baselines: abc',user:{type:'User'},author_association:'OWNER'}
 assert.equal(verifyApproval([comment],'abc'),true)
 assert.equal(verifyApproval([comment],'changed'),false)
 assert.equal(verifyApproval([{...comment,user:{type:'Bot'}}],'abc'),false)
 assert.equal(verifyApproval([{...comment,author_association:'NONE'}],'abc'),false)
 assert.equal(verifyApproval([{...comment,body:'Looks good abc'}],'abc'),false)
})
