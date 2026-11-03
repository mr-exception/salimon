// Development-only deterministic scenario. Uses the production runtime and App;
// this HTML is not an entry point in the distributed Vite build.
import ReactDOM from 'react-dom/client'
import { App } from '../src/App'
import { ORIGIN, translate } from '../src/coordinates'
import { NavigationRuntime } from '../src/navigationRuntime'
import { physicalRadiusM } from '../src/contact'
import { type Body } from '../src/sampleUniverse'
import '../src/styles.css'
const scenario=new URLSearchParams(location.search).get('scenario')
const body:Body={id:'fixture-body',name:'Contact fixture',kind:scenario==='asteroid'?'asteroid':'planet',position:translate(ORIGIN,[1e9,0,0]),radiusKm:1}
const navigation=new NavigationRuntime([body])
const warning=scenario==='warning'
navigation.clock.state={...navigation.clock.state,position:translate(body.position,[physicalRadiusM(body)+(warning?2000:0),0,0])}
if(warning)navigation.advance(5)
navigation.clock.state={...navigation.clock.state,velocityMps:[warning?-150:scenario==='destroyed'?-100:-20,0,0]}
ReactDOM.createRoot(document.getElementById('root')!).render(<App navigation={navigation}/> )
