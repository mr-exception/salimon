import {test} from 'node:test'
import assert from 'node:assert/strict'
import {ORIGIN,translate,LIGHT_SECOND_M} from './coordinates.ts'
import {clipViewportSegment,radarGroups,type Point2} from './radar.ts'
import type {Body} from './sampleUniverse.ts'
const body=(id:string,x:number,y:number):Body=>({id,name:id,kind:'planet',radiusKm:1,position:translate(ORIGIN,[x*LIGHT_SECOND_M,0,y*LIGHT_SECOND_M])})
const project=(p:readonly number[])=>[p[0]/100,p[2]/100,0] as const
const near=(a:number,b:number)=>assert.ok(Math.abs(a-b)<1e-9,`${a} != ${b}`)
test('directional segments reach all viewport edges and corners without a false interior endpoint',()=>{
 for(const [end,expected] of [ [[2,0],[1,0]],[[-2,0],[-1,0]],[[0,2],[0,1]],[[0,-2],[0,-1]],[[2,2],[1,1]],[[-2,2],[-1,1]],[[2,-2],[1,-1]],[[-2,-2],[-1,-1]],[[2,.5],[1,.25]] ] as [Point2,Point2][]) {
  const clipped=clipViewportSegment([0,0],end)!
  assert.deepEqual(clipped.start,[0,0]);near(clipped.end[0],expected[0]);near(clipped.end[1],expected[1])
 }
})
test('panned ship clips entering and exiting rays; misses, parallel, coincident and nonfinite projections are safe',()=>{
 assert.deepEqual(clipViewportSegment([2,0],[-2,0]),{start:[1,0],end:[-1,0]})
 assert.deepEqual(clipViewportSegment([0,2],[0,-2]),{start:[0,1],end:[0,-1]})
 assert.deepEqual(clipViewportSegment([2,1],[-2,-1]),{start:[1,.5],end:[-1,-.5]})
 for(const [a,b] of [[[2,0],[3,0]],[[2,2],[2,-2]],[[1,1],[2,2]],[[0,0],[0,0]],[[NaN,0],[2,0]],[[0,0],[Infinity,0]]] as [Point2,Point2][])assert.equal(clipViewportSegment(a,b),null)
})
test('clipping preserves direction and collinearity under arbitrary camera projections',()=>{
 for(let i=0;i<200;i++) {
  const start:Point2=[Math.sin(i*17)*3,Math.cos(i*7)*3],end:Point2=[Math.cos(i*13)*4,Math.sin(i*11)*4]
  const line=clipViewportSegment(start,end)
  if(!line)continue
  const dx=end[0]-start[0],dy=end[1]-start[1]
  for(const p of [line.start,line.end]) {
   assert.ok(p.every(v=>v>=-1&&v<=1))
   near((p[0]-start[0])*dy-(p[1]-start[1])*dx,0)
  }
  assert.ok((line.end[0]-line.start[0])*dx+(line.end[1]-line.start[1])*dy>0)
 }
})
test('only actual viewport exits have lines; projection pan/rotation and ship-relative origin affect their direction',()=>{
 const b=body('target',200,50)
 const [g]=radarGroups([b],ORIGIN,project,800,600)
 assert.deepEqual(g.line,{start:[50,50],end:[100,37.5]})
 const pan=(p:readonly number[])=>[p[0]/100+2,p[2]/100,0] as const
 assert.equal(radarGroups([b],ORIGIN,pan,800,600)[0].line,null,'ship and target on same side have no visible segment')
 const rotated=(p:readonly number[])=>[-p[2]/100,p[0]/100,0] as const
 assert.deepEqual(radarGroups([b],ORIGIN,rotated,800,600)[0].line,{start:[50,50],end:[37.5,0]})
 assert.equal(radarGroups([body('inside',95,90)],ORIGIN,project,800,600)[0].edge,false,'inside viewport is not an edge contact')
 const depth=radarGroups([b],ORIGIN,()=>[0,0,2],800,600)[0]
 assert.equal(depth.line,null);assert.equal(depth.depthOnly,true)
 assert.deepEqual(radarGroups([b],ORIGIN,project,0,600),[])
})
test('deterministic overlap grouping preserves every ID and prioritizes the selected ray',()=>{
 const contacts=Array.from({length:2000},(_,i)=>body(`contact-${String(i).padStart(4,'0')}`,200+i/100,15+i/1000))
 const a=radarGroups(contacts,ORIGIN,project,800,600,'contact-1999')
 const b=radarGroups([...contacts].reverse(),ORIGIN,project,800,600,'contact-1999')
 assert.deepEqual(a,b);assert.equal(a.length,1);assert.equal(a[0].bodies.length,2000);assert.equal(a[0].representative,'contact-1999')
 assert.notDeepEqual(a[0].line,radarGroups(contacts,ORIGIN,project,800,600)[0].line)
 const range=radarGroups([body('outside',100001,0),body('eligible',200,0)],ORIGIN,project,800,600)
 assert.deepEqual(range.flatMap(g=>g.bodies.map(b=>b.id)),['eligible'])
})

test('near-collinear cardinal rays share a midpoint slot instead of splitting at its boundary',()=>{
 for(const axis of ['horizontal','vertical']) {
  const contacts=[-1,0,1].map((v,i)=>body(String(i),axis==='horizontal'?200:v/100,axis==='vertical'?200:v/100))
  const groups=radarGroups(contacts,ORIGIN,project,970,800)
  assert.equal(groups.length,1);assert.equal(groups[0].bodies.length,3)
 }
})

test('an unselected group prefers a crossing ray over a nearer path missing the panned viewport',()=>{
 const pan=(p:readonly number[])=>[p[0]/100-1.237,p[2]/100,0] as const
 const contacts=[body('near-miss',0,200),body('crossing',200,200)]
 const [g]=radarGroups(contacts,ORIGIN,pan,970,800)
 assert.equal(g.bodies.length,2);assert.equal(g.representative,'crossing');assert.ok(g.line)
 assert.equal(radarGroups(contacts,ORIGIN,pan,970,800,'near-miss')[0].line,null,'selected path must remain exact')
})
