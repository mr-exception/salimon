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
const scenario=new URLSearchParams(location.search).get('scenario')
const body:Body={id:'fixture-body',name:'Contact fixture',kind:scenario==='asteroid'?'asteroid':'planet',position:translate(ORIGIN,[1e9,0,0]),radiusKm:1}
const dense=scenario==='dense'
const visuals=scenario==='visuals'
const edges=scenario?.startsWith('edges')??false
const points=[['right',1940,0],['left',-1940,0],['top',0,1600],['bottom',0,-1600],['top-right',1940,1600],['top-left',-1940,1600],['bottom-right',1940,-1600],['bottom-left',-1940,-1600]] as const
const edgeBodies:Body[]=scenario==='edges-dense'?Array.from({length:2000},(_,i)=>({...body,id:`edge-${String(i).padStart(4,'0')}`,name:`Edge contact ${String(i).padStart(4,'0')}`,position:translate(ORIGIN,[(1940+i/100)*LIGHT_SECOND_M,(i/100)*LIGHT_SECOND_M,0])})):points.map(([id,x,y])=>({...body,id,name:id,position:translate(ORIGIN,[x*LIGHT_SECOND_M,y*LIGHT_SECOND_M,0])}))
edgeBodies.push({...body,id:'out-of-range',name:'Out of range',position:translate(ORIGIN,[100001*LIGHT_SECOND_M,0,0])})
const visualBodies:Body[]=(['star','planet','moon','asteroid'] as const).map((kind,i)=>({...body,id:kind,name:kind,kind,position:translate(ORIGIN,[(i-1.5)*230*LIGHT_SECOND_M,0,-260*LIGHT_SECOND_M])}))
const catalog=edges?edgeBodies:visuals?visualBodies:dense?Array.from({length:2000},(_,i)=>({...body,id:`dense-${String(i).padStart(4,'0')}`,name:`Dense contact ${String(i).padStart(4,'0')}`,position:translate(ORIGIN,[100*LIGHT_SECOND_M+i,0,0])})):[body]
const navigation=new NavigationRuntime(catalog)
const warning=scenario==='warning'
if((edges&&scenario!=='edges-unscanned')||dense||visuals){navigation.clock.state={...navigation.clock.state,tick:300};navigation.scanner.update(300,navigation.clock.state.position)}
else if(!edges) {
 navigation.clock.state={...navigation.clock.state,position:translate(body.position,[physicalRadiusM(body)+(warning?2000:0),0,0])}
 if(warning)navigation.advance(5)
 navigation.clock.state={...navigation.clock.state,velocityMps:[warning?-150:scenario==='destroyed'?-100:-20,0,0]}
}
window.navigationTest={positions(){return navigation.scanner.contacts.map(b=>({id:b.id,positionLs:radarPosition(b.position,navigation.clock.state.position)}))},moveShipLs(offset){navigation.clock.state={...navigation.clock.state,position:translate(ORIGIN,offset.map(v=>v*LIGHT_SECOND_M) as [number,number,number])}}}
declare global {interface Window {navigationTest:{positions:()=>{id:string;positionLs:readonly [number,number,number]}[];moveShipLs:(offset:[number,number,number])=>void}}}
ReactDOM.createRoot(document.getElementById('root')!).render(<App navigation={navigation} onSceneReady={observeScene}/> )
