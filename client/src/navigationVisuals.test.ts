import { test } from 'node:test'
import assert from 'node:assert/strict'
import { IcosahedronGeometry, Vector3 } from 'three'
import { bodyGeometries, shipGeometry, wingGeometry } from './visualGeometry.ts'
test('navigation assets have finite bounded flat facets and a directional ship silhouette',()=>{
 for(const geometry of [...Object.values(bodyGeometries),shipGeometry,wingGeometry]) {
  geometry.computeBoundingBox()
  assert.ok((geometry.index?.count??geometry.getAttribute('position').count)/3<=100)
  for(const value of geometry.getAttribute('position').array)assert.ok(Number.isFinite(value))
  const box=geometry.boundingBox!;assert.ok(box.max.distanceTo(box.min)>0)
 }
 const hull=shipGeometry.boundingBox!
 assert.ok(-hull.min.z>hull.max.z*1.5,'nose extends forward along simulation -Z')
 assert.ok(wingGeometry.boundingBox!.getSize(new Vector3()).x>hull.getSize(new Vector3()).x*2)
 const rock=bodyGeometries.asteroid.boundingBox!
 assert.ok(rock.getSize(new Vector3()).x>rock.getSize(new Vector3()).y*1.3)
 assert.notDeepEqual(Array.from(bodyGeometries.asteroid.getAttribute('position').array),Array.from(new IcosahedronGeometry(1,0).getAttribute('position').array))
})

test('closed ship hull triangles face outward so back-face culling preserves the upper hull',()=>{
 const positions=shipGeometry.getAttribute('position'),indices=shipGeometry.index!
 for(let i=0;i<indices.count;i+=3) {
  const a=new Vector3().fromBufferAttribute(positions,indices.getX(i)),b=new Vector3().fromBufferAttribute(positions,indices.getX(i+1)),c=new Vector3().fromBufferAttribute(positions,indices.getX(i+2))
  const outward=b.clone().sub(a).cross(c.clone().sub(a))
  assert.ok(outward.dot(a.clone().add(b).add(c).divideScalar(3))>0,'convex hull face points away from its interior')
 }
})
