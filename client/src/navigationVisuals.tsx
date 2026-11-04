import { useRef } from 'react'
import { useFrame } from '@react-three/fiber'
import * as THREE from 'three'
import type { BodyKind } from './sampleUniverse.ts'

import { VISUAL_RADIUS_PX,bodyGeometries,bodyMaterials,shipGeometry,wingGeometry,hullMaterial,wingMaterial,glassMaterial,engineMaterial,engineGeometry,canopyGeometry,selectionGeometry,selectionMaterial } from './visualGeometry.ts'

/** Screen-readable proxy, with real linear position and independent physical size. */
export function ScreenSize({pixels,children}:{pixels:number;children:React.ReactNode}) {
  const ref=useRef<THREE.Group>(null)
  useFrame(({camera,size})=>{
    if(ref.current && camera instanceof THREE.OrthographicCamera) {
      ref.current.scale.setScalar(pixels*(camera.top-camera.bottom)/(camera.zoom*Math.max(1,size.height)))
    }
  })
  return <group ref={ref}>{children}</group>
}
export function BodyVisual({kind,id,selected}:{kind:BodyKind;id:string;selected:boolean}) {
  const seed=Array.from(id).reduce((n,c)=>(n*31+c.charCodeAt(0))>>>0,0)
  return <ScreenSize pixels={VISUAL_RADIUS_PX[kind]}>
    <mesh name={`nav-body:${kind}:${id}`} geometry={bodyGeometries[kind]} material={bodyMaterials[kind]} rotation={[seed%7,seed%11,seed%5]} dispose={null}/>
    {selected && <mesh name="nav-selection" rotation={[Math.PI/2,0,0]} geometry={selectionGeometry} material={selectionMaterial} dispose={null}/>}
  </ScreenSize>
}
export function ShipVisual({pitch,yaw}:{pitch:number;yaw:number}) {
  return <group rotation={[pitch,yaw,0]} rotation-order="YXZ" name="nav-ship-heading">
    <ScreenSize pixels={18}>
      <mesh name="nav-ship-hull" geometry={shipGeometry} material={hullMaterial} dispose={null}/>
      <mesh name="nav-ship-wings" geometry={wingGeometry} material={wingMaterial} dispose={null}/>
      <mesh name="nav-ship-canopy" position={[0,0.28,-0.17]} scale={[0.55,0.35,1.3]} geometry={canopyGeometry} material={glassMaterial} dispose={null}/>
      {[-0.25,0.25].map(x=><mesh key={x} name="nav-ship-engine" position={[x,0.05,0.88]} geometry={engineGeometry} material={engineMaterial} dispose={null}/>)}
    </ScreenSize>
  </group>
}
// A sparse volume of galactic reference points, without a surface or boundary.
const latticeGeometry=new THREE.BufferGeometry()
const lattice:number[]=[]
for(let x=-4;x<=4;x++)for(let y=-1;y<=1;y++)for(let z=-4;z<=4;z++)lattice.push(x,y,z)
latticeGeometry.setAttribute('position',new THREE.Float32BufferAttribute(lattice,3))
const latticeMaterial=new THREE.PointsMaterial({color:'#587e91',size:1.5,sizeAttenuation:false,transparent:true,opacity:0.28,depthWrite:false})
export function GalacticLattice({spacing,offset}:{spacing:number;offset:readonly [number,number,number]}) {
  return <points name="nav-galactic-lattice" position={[...offset]} scale={spacing} geometry={latticeGeometry} material={latticeMaterial} dispose={null}/>
}
