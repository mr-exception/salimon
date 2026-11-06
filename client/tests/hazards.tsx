// Separate test build only. No production query flag, module grant or planet sampling.
import ReactDOM from 'react-dom/client'
import { TopDownView } from '../src/TopDownView.tsx'
import { ResourceSession } from '../src/resourceSession.ts'
import '../src/styles.css'
let session: ResourceSession | undefined, fail = false
const sources = () => session?.setHazardSources([
  { id: 'fixture-radiation-a', type: 'radiation', intensity: 3.1 },
  { id: 'fixture-radiation-b', type: 'radiation', intensity: .2 },
  { id: 'fixture-poison', type: 'poison', intensity: .125 },
  { id: 'fixture-temperature', type: 'temperature', temperatureC: 25 },
])
ReactDOM.createRoot(document.getElementById('root')!).render(<>
  <TopDownView gameplay createResourceSession={(storage, world) => (session = new ResourceSession({
    getItem: key => storage.getItem(key), setItem: (key, value) => { if (fail) throw Error('Fixture quota failure'); storage.setItem(key, value) },
  }, world))} />
  <nav aria-label="Non-shipping hazard fixture" style={{ position: 'relative', background: '#233437', padding: 8 }}>
    <button onClick={() => session?.addProtection({ id: 'fixture-radiation-module', type: 'radiation', protection: 2 })}>Declare test module</button>
    <button onClick={sources}>Enter test hazards</button>
    <button onClick={() => session?.setHazardSources([])}>Exit test hazards</button>
    <button onClick={() => { fail = true }}>Fail test saves</button>
  </nav>
</>)
