import { useEffect, useRef } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import * as THREE from 'three'
import type { NavigationInput } from './input'

/** Camera input never enters the authoritative flight controls. */
export function CameraControls({onViewRadius,resetSequence,input}:{onViewRadius:(value:number)=>void;resetSequence:number;input:NavigationInput}) {
  const {camera,gl,set}=useThree()
  const controlsRef=useRef<OrbitControls|null>(null)
  const interacting=useRef(false)
  useEffect(()=>{
    const controls=new OrbitControls(camera,gl.domElement)
    controls.enablePan=true
    controls.screenSpacePanning=true
    controls.enableDamping=true
    controls.minZoom=0.001
    controls.maxZoom=30
    controlsRef.current=controls
    set({controls})
    const sync=()=>{
      camera.updateMatrixWorld()
      if(camera instanceof THREE.OrthographicCamera)onViewRadius((camera.top-camera.bottom)/(2*camera.zoom))
    }
    const start=()=>{interacting.current=true}
    const finish=()=>{interacting.current=false;controls.enableDamping=false;controls.update();controls.enableDamping=true}
    controls.addEventListener('start',start)
    controls.addEventListener('end',finish)
    controls.addEventListener('change',sync)
    sync()
    return ()=>{controls.removeEventListener('change',sync);controls.removeEventListener('end',finish);controls.removeEventListener('start',start);interacting.current=false;controls.dispose();controlsRef.current=null;set({controls:null})}
  },[camera,gl,onViewRadius,set])
  useEffect(()=>{
    const controls=controlsRef.current
    if(!controls||!(camera instanceof THREE.OrthographicCamera))return
    // Drain residual pointer damping before resetting both position and target.
    controls.enableDamping=false;controls.update()
    controls.target.set(0,0,0)
    camera.position.set(200000,125000,200000);camera.lookAt(0,0,0);camera.zoom=0.5
    camera.updateProjectionMatrix();controls.update();camera.updateMatrixWorld()
    controls.enableDamping=true
    onViewRadius((camera.top-camera.bottom)/(2*camera.zoom))
  },[camera,resetSequence,onViewRadius])
  useFrame((_state,dt)=>{
    const controls=controlsRef.current
    if(!controls)return
    let panned=false
    if(camera instanceof THREE.OrthographicCamera) {
      const held=input.controls,x=Number(held.right)-Number(held.left),y=Number(held.up)-Number(held.down)
      if(x||y) {
        const step=(camera.top-camera.bottom)/camera.zoom*Math.min(dt,0.05)*0.65/Math.hypot(x,y)
        const shift=new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld,0).multiplyScalar(x*step)
          .addScaledVector(new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld,1),y*step)
        camera.position.add(shift);controls.target.add(shift);panned=true
      }
    }
    // OrbitControls reconstitutes spherical coordinates on update. Leave an
    // idle pose untouched rather than accumulating floating-point roundoff.
    if(panned||interacting.current)controls.update()
    camera.updateMatrixWorld()
  },-2)
  return null
}
