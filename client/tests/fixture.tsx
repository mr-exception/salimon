// Development-only deterministic scenario. Uses the production runtime and App;
// this HTML is not an entry point in the distributed Vite build.
import ReactDOM from 'react-dom/client'
import { App } from '../src/App'
import { ORIGIN, translate, LIGHT_SECOND_M, radarPosition } from '../src/coordinates'
import { NavigationRuntime } from '../src/navigationRuntime'
import { physicalRadiusM } from '../src/contact'
import { type Body } from '../src/sampleUniverse'
import '../src/styles.css'
import { observeScene } from './sceneHarness'
const params=new URLSearchParams(location.search)
const scenario=params.get('scenario')
const seed=Number(params.get('seed')??750)
if(!Number.isSafeInteger(seed)||seed<0||seed>0xffffffff)throw Error('Expected uint32 seed')
let randomState=seed
const random=()=>{randomState=(Math.imul(randomState,1664525)+1013904223)>>>0;return randomState/2**32}
const seeded=scenario==='seeded'||scenario==='seeded-empty'
const body:Body={id:'fixture-body',name:'Contact fixture',kind:scenario==='asteroid'?'asteroid':'planet',position:translate(ORIGIN,[1e9,0,0]),radiusKm:1}
const dense=scenario==='dense'
const visuals=scenario==='visuals'
const edges=(scenario?.startsWith('edges')??false)||scenario==='expiry'
const points=[['right',1940,0],['left',-1940,0],['top',0,1600],['bottom',0,-1600],['top-right',1940,1600],['top-left',-1940,1600],['bottom-right',1940,-1600],['bottom-left',-1940,-1600]] as const
const edgeBodies:Body[]=scenario==='edges-dense'?Array.from({length:2000},(_,i)=>({...body,id:`edge-${String(i).padStart(4,'0')}`,name:`Edge contact ${String(i).padStart(4,'0')}`,position:translate(ORIGIN,[(1940+i/100)*LIGHT_SECOND_M,(i/100)*LIGHT_SECOND_M,0])})):points.map(([id,x,y])=>({...body,id,name:id,position:translate(ORIGIN,[x*LIGHT_SECOND_M,y*LIGHT_SECOND_M,0])}))
edgeBodies.push({...body,id:'out-of-range',name:'Out of range',position:translate(ORIGIN,[100001*LIGHT_SECOND_M,0,0])})
const visualBodies:Body[]=(['star','planet','moon','asteroid'] as const).map((kind,i)=>({...body,id:kind,name:kind,kind,position:translate(ORIGIN,[(i-1.5)*230*LIGHT_SECOND_M,0,-260*LIGHT_SECOND_M])}))
const seededBodies:Body[]=(['star','planet','moon','asteroid'] as const).map((kind,i)=>({...body,id:kind,name:kind,kind,position:translate(ORIGIN,[((i-1.5)*230+(random()-.5)*40)*LIGHT_SECOND_M,(random()-.5)*80*LIGHT_SECOND_M,-260*LIGHT_SECOND_M])}))
const catalog=seeded?seededBodies:edges?edgeBodies:visuals?visualBodies:dense?Array.from({length:2000},(_,i)=>({...body,id:`dense-${String(i).padStart(4,'0')}`,name:`Dense contact ${String(i).padStart(4,'0')}`,position:translate(ORIGIN,[100*LIGHT_SECOND_M+i,0,0])})):[body]
const navigation=new NavigationRuntime(catalog)
const warning=scenario==='warning'
if(scenario==='expiry')navigation.clock.state={...navigation.clock.state,position:translate(ORIGIN,[0,0,99000*LIGHT_SECOND_M]),velocityMps:[0,0,250*LIGHT_SECOND_M]}
if((edges&&scenario!=='edges-unscanned')||dense||visuals||scenario==='seeded'){navigation.clock.state={...navigation.clock.state,tick:300};navigation.scanner.update(300,navigation.clock.state.position)}
else if(!edges&&!seeded) {
 navigation.clock.state={...navigation.clock.state,position:translate(body.position,[physicalRadiusM(body)+(warning?2000:0),0,0])}
 if(warning)navigation.advance(5)
 navigation.clock.state={...navigation.clock.state,velocityMps:[warning?-150:scenario==='destroyed'?-100:-20,0,0]}
}
// All scenario writes happen before mounting. Browser hooks only return copies.
const vector=params.get('vector')
if(vector) {
 const directions:Record<string,[number,number]>={right:[1,0],left:[-1,0],top:[0,1],bottom:[0,-1],'top-right':[970,800],'top-left':[-970,800],'bottom-right':[970,-800],'bottom-left':[-970,-800]}
 const [x,y]=directions[vector]??[1,0]
 const yaw=x>0?-Math.PI/2:x<0?Math.PI/2:0,pitch=Math.atan2(y,Math.abs(x))
 const velocity:[number,number,number]=vector==='zero'?[0,0,0]:vector==='opposite'?[-1,0,0]:vector==='different'?[0,1,0]:vector==='near'?[1,.001,0]:[x,y,0]
 navigation.clock.state={...navigation.clock.state,yaw:vector==='away'?Math.PI/2:yaw,pitch,velocityMps:vector==='away'?[-1,0,0]:velocity,alignment:null}
}
window.navigationTest=Object.freeze({
 positions:()=>navigation.scanner.contacts.map(b=>({id:b.id,positionLs:radarPosition(b.position,navigation.clock.state.position)})),
 snapshot:()=>structuredClone({seed,scenario,ship:navigation.clock.state,pulse:navigation.scanner.pulse,selected:navigation.scanner.selected,catalog:catalog.map(b=>({id:b.id,position:b.position}))})
})
declare global {interface Window {navigationTest:{positions:()=>{id:string;positionLs:readonly [number,number,number]}[];snapshot:()=>{seed:number;scenario:string|null;ship:import('../src/simulation').ShipState;pulse:number;selected:string|null;catalog:{id:string;position:import('../src/coordinates').Position}[]}}}}
const view=params.get('view'),zoom=Number(params.get('zoom')??.5)
if(!Number.isFinite(zoom)||zoom<.001||zoom>30)throw Error('Invalid fixture zoom')
const target:[number,number,number]=[view==='panned'?-1250:view==='outside'?1200:0,0,0]
const position:[number,number,number]=[target[0],view==='tilted'?125000:0,200000]
ReactDOM.createRoot(document.getElementById('root')!).render(<App navigation={navigation} onSceneReady={state=>observeScene(state,view?{position,target,zoom}:undefined)}/> )
