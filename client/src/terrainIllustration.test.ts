import {test} from 'node:test'
import assert from 'node:assert/strict'
import {contourTriangle,terrainPatch,terrainPigment} from './terrainIllustration.ts'
import {sampleGeology} from './planetGeology.ts'
import {geologicalDefinition} from './planetDefinition.ts'
import {surfaceRegion,localToSurface,surfaceToLocal,neighboringRegions} from './surfaceCoordinates.ts'

test('P04 physical pigments survive region edges, ordering and visual resolution',()=>{
  const d=geologicalDefinition('vesper-moon'),location={bodyId:d.bodyId,radialM:[0,d.radiusM+5,0] as const}
  const region=surfaceRegion(location,d),physical=localToSurface(region,[31,20])
  const paint=(p:typeof physical)=>terrainPigment(sampleGeology(p,d),p.radialM,d.domains.visual.seed)
  const expected=paint(physical)
  for(const neighbor of neighboringRegions(region))assert.deepEqual(paint(localToSurface(neighbor,surfaceToLocal(physical,neighbor))),expected)
  assert.deepEqual(paint(physical),expected)
  const warm=geologicalDefinition('caligo'),bare={bodyId:warm.bodyId,radialM:[warm.radiusM+5,0,0] as const}
  assert.ok(expected[2]!>expected[0]!)
  assert.notDeepEqual(expected,terrainPigment(sampleGeology(bare,warm),bare.radialM,warm.domains.visual.seed))
})

test('P04 contours interpolate the same physical edge in either winding, with no saddle gap',()=>{
  assert.equal(contourTriangle([[0,0],[2,0],[2,2]],[0,2,2],1),'M1,0L1,1')
  assert.equal(contourTriangle([[0,0],[2,2],[0,2]],[0,2,0],1),'M1,1L1,2')
  assert.equal(contourTriangle([[0,0],[2,0],[0,2]],[0,0,0],1),'')
})

test('P04 all aspect/zoom rotations stay covered by a bounded 1681-sample patch',()=>{
  for(const [width,height] of [[1200,640],[1200,2400],[6000,640]])for(const zoom of [8,26,64])for(const [cx,cy] of [[0,0],[7.99,-.01],[500,-500]]) {
    const p=terrainPatch(cx!,cy!,width!,height!,zoom),radius=Math.hypot(width!,height!)/zoom/2
    assert.equal(p.count*p.count,1681)
    assert.ok(cx!-radius>=p.x && cx!+radius<=p.x+40*p.step)
    assert.ok(cy!-radius>=p.y && cy!+radius<=p.y+40*p.step)
  }
})

test('P04 physically clipped rock footprint has no offset shadow or detached cap',async()=>{
  const {gradeFootprint}=await import('./terrainIllustration.ts')
  assert.equal(gradeFootprint([[0,0],[2,0],[0,2]],[0,2,0],1),'M1,0L2,0L1,1Z')
  assert.equal(gradeFootprint([[0,0],[2,0],[0,2]],[0,0,0],1),'')
})
