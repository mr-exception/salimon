import {test} from 'node:test'
import assert from 'node:assert/strict'
import * as THREE from 'three'
import {clipViewportRay,projectedVectorRay,type Point2} from './vectorGuides.ts'
const near=(a:number,b:number)=>assert.ok(Math.abs(a-b)<1e-8,`${a} != ${b}`)
test('infinite rays reach all edges/corners independently of vector magnitude',()=>{
 for(const dir of [[1,0],[-1,0],[0,1],[0,-1],[1,1],[-1,1],[1,-1],[-1,-1],[4,1]] as Point2[])for(const scale of [1e-8,1,1e8]) {
  const line=clipViewportRay([0,0],[dir[0]*scale,dir[1]*scale])!
  assert.deepEqual(line.start,[0,0]);near(Math.max(...line.end.map(Math.abs)),1)
  near(line.end[0]*dir[1]-line.end[1]*dir[0],0)
 }
})
test('off-screen origins clip only forward visible portions; malformed, tangent and degenerate rays vanish',()=>{
 assert.deepEqual(clipViewportRay([2,0],[-1,0]),{start:[1,0],end:[-1,0]})
 assert.deepEqual(clipViewportRay([0,2],[0,-1]),{start:[0,1],end:[0,-1]})
 assert.deepEqual(clipViewportRay([-2,-2],[1,1]),{start:[-1,-1],end:[1,1]})
 for(const [origin,dir] of [[[2,0],[1,0]],[[2,2],[0,-1]],[[1,1],[1,1]],[[0,0],[0,0]],[[NaN,0],[1,0]],[[0,0],[Infinity,0]]] as [Point2,Point2][])assert.equal(clipViewportRay(origin,dir),null)
})
test('arbitrary rays preserve collinearity, direction and viewport clipping',()=>{
 for(let i=0;i<500;i++) {
  const origin:Point2=[Math.sin(i*17)*3,Math.cos(i*7)*3],dir:Point2=[Math.cos(i*13),Math.sin(i*11)]
  const line=clipViewportRay(origin,dir);if(!line)continue
  for(const p of [line.start,line.end]){assert.ok(p.every(v=>v>=-1&&v<=1));near((p[0]-origin[0])*dir[1]-(p[1]-origin[1])*dir[0],0)}
  assert.ok((line.end[0]-line.start[0])*dir[0]+(line.end[1]-line.start[1])*dir[1]>0)
 }
})
test('orthographic projection handles independent vectors, pan, zoom, stationary and view-axis directions',()=>{
 const camera=new THREE.OrthographicCamera(-500,500,400,-400,.1,1e6)
 camera.position.set(0,0,200000);camera.lookAt(0,0,0);camera.updateMatrixWorld();camera.updateProjectionMatrix()
 assert.deepEqual(projectedVectorRay(camera,[0,0,0]),null)
 assert.deepEqual(projectedVectorRay(camera,[1e-8,0,0]),null)
 assert.deepEqual(projectedVectorRay(camera,[0,0,-1]),null)
 assert.deepEqual(projectedVectorRay(camera,[1,0,0]),{start:[0,0],end:[1,0]})
 assert.deepEqual(projectedVectorRay(camera,[-1,0,0]),{start:[0,0],end:[-1,0]})
 camera.position.x=700;camera.updateMatrixWorld()
 const panned=projectedVectorRay(camera,[1,0,0])!;near(panned.start[0],-1);near(panned.end[0],1)
 camera.zoom=30;camera.updateProjectionMatrix();assert.ok(projectedVectorRay(camera,[1,0,0]))
})
