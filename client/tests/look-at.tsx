import { useState } from 'react'
import { createRoot } from 'react-dom/client'
import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { ObservationCamera } from '../src/ObservationCamera.tsx'
import { ObservationCameraState } from '../src/observationCamera.ts'
import { OpticalSky } from '../src/OpticalSky.tsx'
import { PerspectiveContacts } from '../src/PerspectiveContacts.tsx'
import { perspectiveMarkers, currentContacts } from '../src/perspectiveContacts.ts'
import { setContactDirection } from '../src/contactCommands.ts'
import { ORIGIN, translate } from '../src/coordinates.ts'
import { SAMPLE_BODIES, type Body } from '../src/sampleUniverse.ts'
import { CatalogIndex, Scanner, SCANNER_RADIUS_M } from '../src/scanner.ts'
import { startingScoutLocation } from '../src/scoutLocation.ts'
import { initialShip, stepShip } from '../src/simulation.ts'
import { acceptsFlightShortcut, axisForCode, NavigationInput } from '../src/input.ts'
const grounded = new URLSearchParams(location.search).has('grounded')
let ship = grounded ? startingScoutLocation().ship : initialShip()
const make = (id: string, z: number, radiusKm: number): Body => ({ id, name: id, kind: 'moon', radiusKm, position: translate(ORIGIN, [0, 0, z]) })
const bodies = grounded ? SAMPLE_BODIES : [make('Foreground', 10000, 1), make('Behind', 20000, 0.5)]
const view = new ObservationCameraState({ yaw: 0, pitch: 0 }), scanner = new Scanner(new CatalogIndex(bodies)), input = new NavigationInput()
let tick = 0, observation: unknown, flying = false, debt = 0
window.addEventListener('keydown', e => {
 const axis = axisForCode(e.code)
 if (axis && acceptsFlightShortcut(e.target) && !e.altKey && !e.ctrlKey && !e.metaKey) {
  input.press(e.code, axis); ship = { ...ship, alignment: null }; e.preventDefault()
 }
})
window.addEventListener('keyup', e => input.release(e.code))
window.addEventListener('blur', () => input.clear())
window.addEventListener('focusin', () => input.clear())
Object.assign(window, { lookSnapshot: () => structuredClone(observation) })
function Scene({ omitHidden }: { omitHidden: boolean }) {
 const { gl, size } = useThree()
 useFrame((_, dt) => {
  if (flying && !document.hidden) {
   debt += Math.min(dt, 0.05)
   while (debt >= 1 / 60) { ship = stepShip(ship, input.controls); debt -= 1 / 60 }
  }
  const ctx = gl.getContext(), debug = ctx.getExtension('WEBGL_debug_renderer_info')
  observation = { ship, view: view.snapshot(), lookingAt: view.lookingAt, selected: scanner.selected,
   contacts: currentContacts(scanner, ship.position).map(b => ({ id: b.id, name: b.name })),
   markers: perspectiveMarkers(currentContacts(scanner, ship.position), bodies, ship.position, view.snapshot(), size.width, size.height),
   renderer: debug ? ctx.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null }
 }, 0)
 return <><ObservationCamera view={view}/><OpticalSky bodies={omitHidden ? bodies.filter(b => b.id !== 'Behind') : bodies} getOrigin={() => ship.position}/></>
}
function Fixture() {
 const [omitHidden, setOmitHidden] = useState(false)
 return <>
 <div style={{ height: '100vh', position: 'relative' }}><Canvas dpr={0.5} gl={{ preserveDrawingBuffer: true }}><Scene omitHidden={omitHidden}/></Canvas>
 <PerspectiveContacts scanner={scanner} geometry={bodies} view={view} getOrigin={() => ship.position} getShip={() => ship}
 onSetDirection={body => { ship = setContactDirection(scanner, ship, body.id); flying = true }}/></div>
 <nav style={{ position: 'absolute', left: 8, bottom: 8 }}>
 <button onClick={() => scanner.update(tick += 300, ship.position)}>Pulse</button>
 <button onClick={() => { ship = { ...ship, position: translate(ship.position, [SCANNER_RADIUS_M * 2, 0, 0]) }; scanner.update(++tick, ship.position) }}>Leave range</button>
 <button onClick={() => view.reset({ yaw: 0, pitch: 0 })}>Reset view</button>
 <button onClick={() => view.zoom(-1000)}>Zoom</button>
 <button onClick={() => setOmitHidden(value => !value)}>Toggle hidden optical control</button>
 </nav>
</>
}
createRoot(document.getElementById('root')!).render(<Fixture/>)
