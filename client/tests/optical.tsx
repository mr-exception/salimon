import { createRoot } from 'react-dom/client'
import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { ObservationCamera } from '../src/ObservationCamera.tsx'
import { ObservationCameraState } from '../src/observationCamera.ts'
import { OpticalSky } from '../src/OpticalSky.tsx'
import { opticalSpheres } from '../src/opticalSky.ts'
import { ORIGIN, translate } from '../src/coordinates.ts'
import { startingScoutLocation } from '../src/scoutLocation.ts'
import { SAMPLE_BODIES, type Body } from '../src/sampleUniverse.ts'
import { CatalogIndex, Scanner } from '../src/scanner.ts'
const scenario = new URLSearchParams(location.search).get('scenario') ?? 'wide'
const origin = scenario === 'translated' ? { cell: [10n ** 24n, -(10n ** 24n), 10n ** 30n] as const, offsetM: [0, 0, 0] as const } : ORIGIN
const make = (id: string, position: readonly [number, number, number], radiusKm: number, kind: Body['kind'] = 'moon'): Body =>
 ({ id, name: `Secret ${id}`, kind, radiusKm, position: translate(origin, position) })
const calibration = [make('foreground', [0, 0, -10000], 1), make('light', [-1e8, 1e8, 0], 500, 'star'),
 make('tiny', [60000, 0, -1e8], 1.4), make('hidden', [0, 0, -1e8], 40)]
const surface = scenario === 'surface' || scenario === 'below', site = startingScoutLocation()
const bodies = surface ? SAMPLE_BODIES : scenario === 'telescope' ? calibration.slice(1) : scenario === 'clear' ? calibration.filter(b => b.id !== 'hidden') : calibration
const shipOrigin = surface ? site.ship.position : origin
const view = new ObservationCameraState({ yaw: 0, pitch: 0 })
if (scenario === 'telescope') view.restore({ quaternion: [0, 0, 0, 1], fov: 0.1 })
if (scenario === 'below') view.reset({ yaw: 0, pitch: -Math.PI / 2 })
const scanner = new Scanner(new CatalogIndex(bodies))
let observation: unknown
Object.assign(window, { opticalSnapshot: () => structuredClone(observation) })
function Scene() {
 const { gl } = useThree()
 useFrame(({ camera }) => {
  const ctx = gl.getContext(), debug = ctx.getExtension('WEBGL_debug_renderer_info')
  observation = { scenario, contacts: scanner.contacts.length, selected: scanner.selected,
   spheres: opticalSpheres(bodies, shipOrigin), origin: shipOrigin, view: view.snapshot(),
   renderer: debug ? ctx.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null,
   calls: gl.info.render.calls, triangles: gl.info.render.triangles, camera: camera.position.toArray() }
 }, 0)
 return <><ObservationCamera view={view}/><OpticalSky bodies={bodies} getOrigin={() => shipOrigin}/></>
}
createRoot(document.getElementById('root')!).render(<>
 <div style={{ height: '100vh' }}><Canvas dpr={1} gl={{ preserveDrawingBuffer: true }}><Scene/></Canvas></div>
 <button style={{ position: 'absolute', top: 0 }} onClick={() => scanner.update(300, shipOrigin)}>Pulse</button>
</>)
