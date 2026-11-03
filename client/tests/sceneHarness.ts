// Development fixture only; Vite's production entry imports none of this module.
import * as THREE from 'three'
import type { RootState } from '@react-three/fiber'
export function observeScene({gl,scene,camera,invalidate}:RootState) {
  if(!(camera instanceof THREE.OrthographicCamera))throw Error('Expected orthographic radar')
  window.radarTest={
    camera(position:[number,number,number],target:[number,number,number],zoom:number) {
      camera.position.set(...position);camera.up.set(0,1,0);camera.lookAt(...target);camera.zoom=zoom;camera.updateProjectionMatrix();camera.updateMatrixWorld();invalidate()
    },
    snapshot() {
      gl.render(scene,camera)
      const context=gl.getContext(),debug=context.getExtension('WEBGL_debug_renderer_info')
      let surfaces=0
      const objects:{name:string;type:string;triangles:number;pixel:number[];pixels:number;colors:number}[]=[]
      scene.traverse(object=>{
        if(object.type==='GridHelper'||(object instanceof THREE.Mesh && object.geometry.type==='PlaneGeometry'))surfaces++
        if(!object.name.startsWith('nav-'))return
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
      return {objects,surfaces,calls:gl.info.render.calls,triangles:gl.info.render.triangles,geometries:gl.info.memory.geometries,renderer:debug?context.getParameter(debug.UNMASKED_RENDERER_WEBGL):context.getParameter(context.RENDERER),zoom:camera.zoom}
    }
  }
}
declare global {interface Window {radarTest:{camera:(position:[number,number,number],target:[number,number,number],zoom:number)=>void;snapshot:()=>{objects:{name:string;type:string;triangles:number;pixel:number[];pixels:number;colors:number}[];surfaces:number;calls:number;triangles:number;geometries:number;renderer:string;zoom:number}}}}
