// Test-only controls call the actual session adapter. No fixture code enters normal dist.
import { useState } from 'react'
import ReactDOM from 'react-dom/client'
import { TopDownView } from '../src/TopDownView.tsx'
import { ResourceSession } from '../src/resourceSession.ts'
import type { GearCommand } from '../src/scoutEquipment.ts'
import '../src/styles.css'
let session: ResourceSession | undefined, fail = false
const suit = 'gear:0:starter-suit', module = 'gear:0:starter-temperature'
function Fixture() {
 const [message, setMessage] = useState('')
 const apply = (command: GearCommand, stale = false) => {
  if (!session) return
  const ok = session.changeEquipment(command, session.snapshot.revision - (stale ? 1 : 0))
  setMessage(ok ? 'Committed' : session.error || session.feedback)
 }
 return <><TopDownView gameplay createResourceSession={(storage, world) => session = new ResourceSession({ getItem: key => storage.getItem(key), setItem: (key, value) => { if (fail) throw Error('Fixture quota failure'); storage.setItem(key, value) } }, world)} />
 <nav aria-label="Non-shipping equipment adapter" style={{ position: 'relative', background: '#233437', padding: 8 }}>
 <button onClick={() => apply({ kind:'move', id:suit, from:{kind:'equipped'}, to:{kind:'drawer'} })}>Store suit fixture</button>
 <button onClick={() => apply({ kind:'move', id:suit, from:{kind:'drawer'}, to:{kind:'equipped'} })}>Equip suit fixture</button>
 <button onClick={() => apply({ kind:'move', id:module, from:{kind:'installed',suitId:suit,slot:0}, to:{kind:'module-storage'} })}>Store module fixture</button>
 <button onClick={() => apply({ kind:'move', id:module, from:{kind:'module-storage'}, to:{kind:'installed',suitId:suit,slot:0} })}>Install module fixture</button>
 <button onClick={() => apply({ kind:'move', id:suit, from:{kind:'equipped'}, to:{kind:'drawer'} }, true)}>Stale suit fixture</button>
 <button onClick={() => { fail = true; setMessage('Writes will fail') }}>Fail equipment saves fixture</button>
 <output aria-label="Equipment fixture result">{message}</output>
 </nav></>
}
ReactDOM.createRoot(document.getElementById('root')!).render(<Fixture />)
