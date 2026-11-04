import { useFrame, useThree } from '@react-three/fiber'
import type { RefObject } from 'react'
import * as THREE from 'three'
import type { NavigationRuntime } from './navigationRuntime'
import { shipVectorGuides } from './vectorGuides'
export type VectorElements=readonly [RefObject<SVGLineElement|null>,RefObject<SVGLineElement|null>]
/** Update the overlay after camera input and before this frame's WebGL render.
 * DOM writes avoid a delayed React publication for these moving endpoints. */
export function ShipVectorGuides({navigation,elements}:{navigation:NavigationRuntime;elements:VectorElements}) {
 const {camera}=useThree()
 useFrame(()=>{
  if(!(camera instanceof THREE.OrthographicCamera))return
  const guides=shipVectorGuides(camera,navigation.clock.state)
  ;[guides.heading,guides.velocity].forEach((line,i)=>{
   const element=elements[i].current
   if(!element)return
   element.style.display=line?'':'none'
   if(line)for(const [name,value] of Object.entries({x1:(line.start[0]+1)*50,y1:(1-line.start[1])*50,x2:(line.end[0]+1)*50,y2:(1-line.end[1])*50}))element.setAttribute(name,String(value))
  })
 },-1)
 return null
}
