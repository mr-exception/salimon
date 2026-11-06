import { createRoot } from 'react-dom/client'
import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { ObservationCamera } from '../src/ObservationCamera.tsx'
import { ObservationCameraState } from '../src/observationCamera.ts'
import { OpticalSky } from '../src/OpticalSky.tsx'
import { PerspectiveContacts } from '../src/PerspectiveContacts.tsx'
import { currentContacts, perspectiveMarkers } from '../src/perspectiveContacts.ts'
import { initialShip } from '../src/simulation.ts'
import { ORIGIN, translate } from '../src/coordinates.ts'
import type { Body } from '../src/sampleUniverse.ts'
import { CatalogIndex, Scanner, SCANNER_RADIUS_M } from '../src/scanner.ts'
const lowResolution = new URLSearchParams(location.search).has('low-resolution')
const base = new URLSearchParams(location.search).has('translated') ? { cell: [10n ** 24n, -(10n ** 24n), 10n ** 30n] as const, offsetM: [0, 0, 0] as const } : ORIGIN
let origin = base, tick = 0, actions = 0
const make = (id: string, x: number, y: number, z: number, radiusKm = 0.00001): Body => ({ id, name: `Detected ${id}`, kind: 'moon', radiusKm, position: translate(base, [x, y, z]) })
const bodies = [make('planet', 0, 0, -10000, 1), make('hidden', 0, 0, -20000), make('tiny', 4000, 0, -10000), make('back', 0, 0, 10000), make('outside', 20000, 0, -10000),
 ...Array.from({ length: 18 }, (_, i) => make(`dense-${String(i).padStart(2, '0')}`, -4000, 2000, -10000))]
const scanner = new Scanner(new CatalogIndex(bodies)), view = new ObservationCameraState({ yaw: 0, pitch: 0 })
let observation: unknown
Object.assign(window, { sensorSnapshot: () => structuredClone(observation) })
function Scene() {
 const { gl, size } = useThree()
 useFrame(() => {
  const ctx = gl.getContext(), debug = ctx.getExtension('WEBGL_debug_renderer_info')
  observation = { tick, contacts: scanner.contacts.map(b => b.id), selected: scanner.selected, actions,
   markers: perspectiveMarkers(currentContacts(scanner, origin), bodies, origin, view.snapshot(), size.width, size.height),
   view: view.snapshot(), canvas: { width: gl.domElement.width, height: gl.domElement.height }, renderer: debug ? ctx.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null }
 }, 0)
 return <><ObservationCamera view={view}/><OpticalSky bodies={bodies} getOrigin={() => origin}/></>
}
createRoot(document.getElementById('root')!).render(<>
 <div style={{ height: '100vh', position: 'relative' }}><Canvas dpr={lowResolution ? 0.5 : 1} gl={{ preserveDrawingBuffer: true }}><Scene/></Canvas>
 <PerspectiveContacts scanner={scanner} geometry={bodies} view={view} getOrigin={() => origin} getShip={() => ({ ...initialShip(), position: origin })} onSetDirection={() => actions++}/></div>
 <nav style={{ position: 'absolute', bottom: 8, left: 8 }}>
 <button onClick={() => scanner.update(++tick, origin)}>Tick</button>
 <button onClick={() => { tick = Math.ceil((tick + 1) / 300) * 300; scanner.update(tick, origin) }}>Pulse</button>
 <button onClick={() => { origin = translate(base, [SCANNER_RADIUS_M * 2, 0, 0]); scanner.update(++tick, origin) }}>Leave range</button>
 <button onClick={() => { view.restore({ quaternion: [0, -Math.sin(Math.atan(0.4) / 2), 0, Math.cos(Math.atan(0.4) / 2)], fov: 0.1 }) }}>Telescope</button>
 <button onClick={() => view.reset({ yaw: 0, pitch: 0 })}>Reset view</button>
 </nav>
</>)
