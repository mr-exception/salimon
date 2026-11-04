import { useRef } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import * as THREE from 'three'
import { BodyVisual, ShipVisual } from './navigationVisuals'
import { radarGroups, type ContactGroup } from './radar'
import type { NavigationRuntime } from './navigationRuntime'
import type { Body } from './sampleUniverse'

/** Mesh identity follows scanned IDs, never camera bins. Per-frame projection
 * controls visibility/position before rendering; React only publishes menus. */
export function ContinuousContacts({navigation,contacts,selected,selectGroup,setGroups}:{navigation:NavigationRuntime;contacts:readonly Body[];selected:string|null;selectGroup:(group:ContactGroup)=>void;setGroups:(groups:ContactGroup[])=>void}) {
 const {camera,size}=useThree()
 const nodes=useRef(new Map<string,THREE.Group>())
 const activeGroups=useRef(new Map<string,ContactGroup>())
 const ship=useRef<THREE.Group>(null)
 const elapsed=useRef(0)
 const projection=useRef(new THREE.Vector3())
 useFrame((_state,dt)=>{
  const pose=navigation.clock.state
  const groups=radarGroups(navigation.scanner.contacts,pose.position,p=>{
   const v=projection.current.set(...p).project(camera);return [v.x,v.y,v.z]
  },size.width,size.height,navigation.scanner.selected)
  const active=new Map(groups.map(group=>[group.representative,group]))
  activeGroups.current=active
  for(const [id,node] of nodes.current) {
   const group=active.get(id)
   node.visible=!!group
   if(group)node.position.set(...group.position)
  }
  if(ship.current)ship.current.rotation.set(pose.pitch,pose.yaw,0,'YXZ')
  elapsed.current+=dt
  if(elapsed.current>=.1){setGroups(groups);elapsed.current=0}
 },-1)
 return <>
  <group ref={ship}><ShipVisual pitch={0} yaw={0}/></group>
  {contacts.map(body=><group key={body.id} ref={node=>{if(node)nodes.current.set(body.id,node);else nodes.current.delete(body.id)}} visible={false} onClick={event=>{
   const group=activeGroups.current.get(body.id)
   if(group){event.stopPropagation();selectGroup(group)}
  }}>
   <BodyVisual kind={body.kind} id={body.id} selected={selected===body.id}/>
  </group>)}
 </>
}
