import * as THREE from 'three'
import type { BodyKind } from './sampleUniverse.ts'

// Shared presentation resources. Their screen sizes never enter simulation,
// scanner range, catalog positions, physical radii or contact handling.
export const VISUAL_RADIUS_PX: Record<BodyKind, number> = {star:18,planet:13,moon:9,asteroid:8}
export const planetGeometry = new THREE.IcosahedronGeometry(1,1)
export const moonGeometry = new THREE.IcosahedronGeometry(1,0)
export const starGeometry = new THREE.DodecahedronGeometry(1,0)
export const asteroidGeometry = new THREE.IcosahedronGeometry(1,0)
const rockPositions=asteroidGeometry.getAttribute('position')
for(let i=0;i<rockPositions.count;i++) {
  const x=rockPositions.getX(i),y=rockPositions.getY(i),z=rockPositions.getZ(i)
  // Equal vertices receive equal displacement: no cracks between flat faces.
  const radial=0.85+0.18*Math.sin(x*19+y*37+z*53)
  rockPositions.setXYZ(i,x*radial*1.3,y*radial*0.72,z*radial)
}
asteroidGeometry.computeVertexNormals()
export const bodyGeometries={star:starGeometry,planet:planetGeometry,moon:moonGeometry,asteroid:asteroidGeometry}
const material=(color:string,emissive='#000000')=>new THREE.MeshLambertMaterial({color,emissive,flatShading:true})
export const bodyMaterials={star:material('#ebc076','#664010'),planet:material('#69bdb8'),moon:material('#a3adbf'),asteroid:material('#ac849c')}
export const hullMaterial=material('#8695a4'),wingMaterial=material('#55616f'),glassMaterial=material('#69d3dd','#08262b')
export const engineMaterial=new THREE.MeshBasicMaterial({color:'#79e4ca'})
export const selectionMaterial=new THREE.MeshBasicMaterial({color:'#e5ffc2',depthTest:false})
export const selectionGeometry=new THREE.TorusGeometry(1.65,0.045,3,32)
export const engineGeometry=new THREE.BoxGeometry(0.19,0.16,0.05)
export const canopyGeometry=new THREE.OctahedronGeometry(0.36,0)
// Broad clipped hull facets, a long -Z nose, swept wings and paired aft engines.
export const shipGeometry=new THREE.BufferGeometry()
shipGeometry.setAttribute('position',new THREE.Float32BufferAttribute([
  0,0,-1.5, -0.43,0,-0.25, 0,0.3,-0.12, 0.43,0,-0.25,
  -0.38,0,0.85, 0,0.18,0.85, 0.38,0,0.85, 0,-0.22,0.45,
],3))
shipGeometry.setIndex([0,2,1,0,3,2,1,2,5,1,5,4,2,3,6,2,6,5,4,5,6,0,1,7,0,7,3,1,4,7,3,7,6,4,6,7])
shipGeometry.computeVertexNormals()
export const wingGeometry=new THREE.BufferGeometry()
wingGeometry.setAttribute('position',new THREE.Float32BufferAttribute([
  -0.3,0,-0.3, -1.12,-0.03,0.72, -0.3,0.08,0.5,
  -1.12,-0.03,0.72, -0.3,0,0.82, -0.3,0.08,0.5,
  0.3,0,-0.3, 0.3,0.08,0.5, 1.12,-0.03,0.72,
  1.12,-0.03,0.72, 0.3,0.08,0.5, 0.3,0,0.82,
],3))
wingGeometry.computeVertexNormals()
wingMaterial.side=THREE.DoubleSide

