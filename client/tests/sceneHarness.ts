// Development fixture only; Vite's production entry imports none of this module.
import * as THREE from 'three'
import type { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { addAfterEffect, type RootState } from '@react-three/fiber'
export function observeScene({gl,scene,camera,invalidate,get}:RootState,initialView?:{position:[number,number,number];target:[number,number,number];zoom:number}) {
  if(!(camera instanceof THREE.OrthographicCamera))throw Error('Expected orthographic radar')
  // Initialize privately after the normal reset effect, then observe completed
  // production frames. snapshot() neither renders nor changes the scene.
  let initialized=false
  let lastSnapshot:ReturnType<typeof capture>
  addAfterEffect(()=>{
    if(!initialized){
      initialized=true
      if(initialView){
        const controls=get().controls as OrbitControls|null
        if(controls){controls.enableDamping=false;controls.update();controls.target.set(...initialView.target)}
        camera.position.set(...initialView.position);camera.up.set(0,1,0);camera.lookAt(...initialView.target);camera.zoom=initialView.zoom
        camera.updateProjectionMatrix();controls?.update();camera.updateMatrixWorld();if(controls)controls.enableDamping=true
        invalidate();return
      }
    }
    lastSnapshot=capture()
    window.radarTest=observer
  })
  const observer=Object.freeze({
    projectLs(point:[number,number,number]) {
      const v=new THREE.Vector3(...point).project(camera)
      return [(v.x+1)*50,(1-v.y)*50]
    },
    recordFrames(count:number) {
      if(!Number.isInteger(count)||count<1||count>120)throw Error('Expected 1..120 frames')
      return new Promise<PanRecording>(resolve=>{
        const frames:PanFrame[]=[],captures:Record<string,string>={}
        const stop=addAfterEffect(()=>{
          const snapshot=window.radarTest.snapshot(),context=gl.getContext()
          const expected=window.navigationTest.positions().map(body=>{
            const ndc=new THREE.Vector3(...body.positionLs).project(camera)
            return {id:body.id,ndc:[ndc.x,ndc.y,ndc.z],pixel:[(ndc.x+1)*gl.domElement.width/2,(1-ndc.y)*gl.domElement.height/2]}
          }).filter(body=>Math.abs(body.ndc[0])<.96&&Math.abs(body.ndc[1])<.96&&Math.abs(body.ndc[2])<=1)
          const bodies=expected.map(body=>{
            const rgba=new Uint8Array(32*32*4)
            context.readPixels(Math.round(body.pixel[0]-16),Math.round(gl.domElement.height-body.pixel[1]-16),32,32,context.RGBA,context.UNSIGNED_BYTE,rgba)
            let bright=0;for(let i=0;i<rgba.length;i+=4)if(Math.max(rgba[i],rgba[i+1],rgba[i+2])>65)bright++
            const actual=snapshot.objects.find(o=>o.name.endsWith(`:${body.id}`))
            return {...body,present:!!actual,actualPixel:actual?.pixel??null,bright}
          })
          const frame={bodies,rendered:snapshot.objects.filter(o=>o.name.startsWith('nav-body:')).map(o=>o.name),ship:snapshot.objects.find(o=>o.name==='nav-ship-hull')?.pixel??null,vectors:[...document.querySelectorAll('.ship-vectors line')].map(n=>({kind:n.getAttribute('data-vector'),display:(n as SVGElement).style.display,start:[Number(n.getAttribute('x1')),Number(n.getAttribute('y1'))],end:[Number(n.getAttribute('x2')),Number(n.getAttribute('y2'))]})),pulse:document.querySelector('.scan-status')?.textContent??''}
          frames.push(frame)
          const key=bodies.some(b=>!b.present||b.bright<10)?'missing':bodies.length>0?'visible':'empty'
          if(!captures[key])captures[key]=gl.domElement.toDataURL('image/png')
          if(frames.length>=count){stop();resolve({frames,captures,renderer:snapshot.renderer})}
        })
      })
    },
    snapshot:()=>structuredClone(lastSnapshot)
  })
  function capture() {
      const context=gl.getContext(),debug=context.getExtension('WEBGL_debug_renderer_info')
      let surfaces=0,arrows=0
      const objects:{name:string;type:string;triangles:number;pixel:number[];pixels:number;colors:number}[]=[]
      scene.traverse(object=>{
        if(object.type==='ArrowHelper')arrows++
        if(object.type==='GridHelper'||(object instanceof THREE.Mesh && object.geometry.type==='PlaneGeometry'))surfaces++
        if(!object.name.startsWith('nav-'))return
        for(let parent:THREE.Object3D|null=object;parent;parent=parent.parent)if(!parent.visible)return
        const center=object.getWorldPosition(new THREE.Vector3()).project(camera)
        const pixel=[(center.x+1)*gl.domElement.width/2,(1-center.y)*gl.domElement.height/2]
        const side=64,x=Math.round(pixel[0]-side/2),y=Math.round(gl.domElement.height-pixel[1]-side/2)
        let pixels=0;const colors=new Set<string>()
        if(x>=0&&y>=0&&x+side<=gl.domElement.width&&y+side<=gl.domElement.height) {
          const rgba=new Uint8Array(side*side*4);context.readPixels(x,y,side,side,context.RGBA,context.UNSIGNED_BYTE,rgba)
          for(let i=0;i<rgba.length;i+=4)if(Math.max(rgba[i],rgba[i+1],rgba[i+2])>65){pixels++;colors.add(`${rgba[i]},${rgba[i+1]},${rgba[i+2]}`)}
        }
        const geometry=object instanceof THREE.Mesh?object.geometry:null
        objects.push({name:object.name,type:object.type,triangles:geometry?(geometry.index?.count??geometry.getAttribute('position').count)/3:0,pixel,pixels,colors:colors.size})
      })
      return {objects,surfaces,arrows,calls:gl.info.render.calls,triangles:gl.info.render.triangles,geometries:gl.info.memory.geometries,renderer:debug?context.getParameter(debug.UNMASKED_RENDERER_WEBGL):context.getParameter(context.RENDERER),zoom:camera.zoom}
  }
}
declare global {interface Window {radarTest:{recordFrames:(count:number)=>Promise<PanRecording>;projectLs:(point:[number,number,number])=>number[];snapshot:()=>{objects:{name:string;type:string;triangles:number;pixel:number[];pixels:number;colors:number}[];surfaces:number;arrows:number;calls:number;triangles:number;geometries:number;renderer:string;zoom:number}}}}

export interface PanFrame {bodies:{id:string;ndc:number[];pixel:number[];present:boolean;actualPixel:number[]|null;bright:number}[];rendered:string[];ship:number[]|null;vectors:{kind:string|null;display:string;start:number[];end:number[]}[];pulse:string}
export interface PanRecording {frames:PanFrame[];captures:Record<string,string>;renderer:string}
