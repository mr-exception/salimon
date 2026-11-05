// Non-shipping HTML entry point. No query parameter or hook enables fixtures in the production game.
import ReactDOM from 'react-dom/client'
import { TopDownView } from '../src/TopDownView.tsx'
import { ResourceSession } from '../src/resourceSession.ts'
import { TEST_ITEMS } from './itemFixtures.ts'
import '../src/styles.css'
let session: ResourceSession | undefined
const change = (action: 'add' | 'remove', id: string, count: number) => session?.changeItem(action, id, count, session.snapshot.revision)
ReactDOM.createRoot(document.getElementById('root')!).render(<>
  <TopDownView gameplay createResourceSession={(storage, world) => (session = new ResourceSession(storage, world, undefined, TEST_ITEMS))} />
  <nav aria-label="Non-shipping item fixture" style={{ position: 'relative', background: '#233437', padding: 8 }}>
    <button onClick={() => change('add', 'fixture-single', 2)}>Add two singles</button>
    <button onClick={() => change('add', 'fixture-stack', 5)}>Add five stacked</button>
    <button onClick={() => change('remove', 'fixture-stack', 1)}>Remove one stacked</button>
    <button onClick={() => change('add', 'fixture-stack', 40)}>Try overweight</button>
  </nav>
</>)
