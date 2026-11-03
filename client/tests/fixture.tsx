// Development-only deterministic scenario. Uses the production runtime and App;
// this HTML is not an entry point in the distributed Vite build.
import ReactDOM from 'react-dom/client'
import { App } from '../src/App'
import { ORIGIN, translate, LIGHT_SECOND_M } from '../src/coordinates'
import { NavigationRuntime } from '../src/navigationRuntime'
import { physicalRadiusM } from '../src/contact'
import { type Body } from '../src/sampleUniverse'
import '../src/styles.css'
import { observeScene } from './sceneHarness'
const scenario=new URLSearchParams(location.search).get('scenario')
const body:Body={id:'fixture-body',name:'Contact fixture',kind:scenario==='asteroid'?'asteroid':'planet',position:translate(ORIGIN,[1e9,0,0]),radiusKm:1}
const dense=scenario==='dense'
const visuals=scenario==='visuals'
const visualBodies:Body[]=(['star','planet','moon','asteroid'] as const).map((kind,i)=>({...body,id:kind,name:kind,kind,position:translate(ORIGIN,[(i-1.5)*230*LIGHT_SECOND_M,0,-260*LIGHT_SECOND_M])}))
const catalog=visuals?visualBodies:dense?Array.from({length:2000},(_,i)=>({...body,id:`dense-${String(i).padStart(4,'0')}`,name:`Dense contact ${String(i).padStart(4,'0')}`,position:translate(ORIGIN,[100*LIGHT_SECOND_M+i,0,0])})):[body]
const navigation=new NavigationRuntime(catalog)
const warning=scenario==='warning'
if(dense||visuals){navigation.clock.state={...navigation.clock.state,tick:300};navigation.scanner.update(300,navigation.clock.state.position)}
else {
 navigation.clock.state={...navigation.clock.state,position:translate(body.position,[physicalRadiusM(body)+(warning?2000:0),0,0])}
 if(warning)navigation.advance(5)
 navigation.clock.state={...navigation.clock.state,velocityMps:[warning?-150:scenario==='destroyed'?-100:-20,0,0]}
}
ReactDOM.createRoot(document.getElementById('root')!).render(<App navigation={navigation} onSceneReady={observeScene}/> )
