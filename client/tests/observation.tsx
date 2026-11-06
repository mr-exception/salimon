import { useEffect, useRef, useState } from 'react'
import { createRoot } from 'react-dom/client'
import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { Mesh, PerspectiveCamera } from 'three'
import { ObservationCamera } from '../src/ObservationCamera.tsx'
import { observationFrame, observationRelativePosition, ObservationCameraState } from '../src/observationCamera.ts'
import { initialShip, NavigationClock } from '../src/simulation.ts'
import { acceptsFlightShortcut, axisForCode, FrameTimer, NavigationInput } from '../src/input.ts'
import { translate } from '../src/coordinates.ts'

// Development-only: actual simulation and adapter, synthetic colored optical
// calibration targets. No catalog/scanner/HUD claims or production route.
const clock = new NavigationClock(initialShip())
const view = new ObservationCameraState(clock.state)
const flight = new NavigationInput()
let observation: unknown = null
Object.assign(window, { observationSnapshot: () => structuredClone(observation) })
const target = translate(clock.state.position, [0, 0, -1000])

function Scene() {
  const marker = useRef<Mesh>(null!)
  const { gl } = useThree()
  const timer = useRef(new FrameTimer(performance.now()))
  useEffect(() => {
    const clear = () => { flight.clear(); timer.current.resume(performance.now()) }
    const down = (event: KeyboardEvent) => {
      if (!acceptsFlightShortcut(event.target) || event.altKey || event.ctrlKey || event.metaKey) return
      const axis = axisForCode(event.code)
      if (axis) { event.preventDefault(); flight.press(event.code, axis) }
    }
    const up = (event: KeyboardEvent) => flight.release(event.code)
    window.addEventListener('keydown', down); window.addEventListener('keyup', up)
    window.addEventListener('blur', clear); window.addEventListener('resize', clear)
    document.addEventListener('focusin', clear); document.addEventListener('visibilitychange', clear)
    return () => {
      clear(); window.removeEventListener('keydown', down); window.removeEventListener('keyup', up)
      window.removeEventListener('blur', clear); window.removeEventListener('resize', clear)
      document.removeEventListener('focusin', clear); document.removeEventListener('visibilitychange', clear)
    }
  }, [])
  useFrame(() => {
    clock.advance(timer.current.sample(performance.now(), document.hidden), flight.controls)
    marker.current.position.fromArray(observationRelativePosition(target, clock.state))
  }, -3)
  useFrame(({ camera, size }) => {
    const context = gl.getContext(), debug = context.getExtension('WEBGL_debug_renderer_info')
    observation = {
      view: view.snapshot(), origin: observationFrame(view, clock.state).origin,
      ship: clock.state, held: flight.controls,
      render: { position: camera.position.toArray(), quaternion: camera.quaternion.toArray(),
        fov: (camera as PerspectiveCamera).fov, aspect: (camera as PerspectiveCamera).aspect,
        width: size.width, height: size.height, dpr: gl.getPixelRatio(),
        framebuffer: [gl.domElement.width, gl.domElement.height],
        renderer: debug ? context.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null },
    }
  }, -1)
  return <><ObservationCamera view={view}/>
    <mesh ref={marker}><sphereGeometry args={[0.25, 24, 16]}/><meshBasicMaterial color="cyan" toneMapped={false}/></mesh>
    {[[0, 0, -1000], [1000, 0, 0], [-1000, 0, 0], [0, 1000, 0], [0, -1000, 0], [0, 0, 1000]].map((p, i) =>
      <mesh key={i} position={p as [number, number, number]}><sphereGeometry args={[40, 16, 12]}/><meshBasicMaterial color={['#243e70', '#802020', '#208020', '#806020', '#602080', '#208080'][i]} wireframe/></mesh>)}
  </>
}
function Fixture() {
  const [mounted, setMounted] = useState(true)
  return <main style={{ color: 'white', background: '#080d19', height: '100vh', fontFamily: 'sans-serif' }}>
    <h1 style={{ margin: 0 }}>Observation camera fixture</h1>
    <p>WASD / left-drag: look · wheel: optical zoom · arrows: ship steering</p>
    <button onClick={() => view.command({ type: 'reset', heading: clock.state })}>Reset view</button>
    <button onClick={() => setMounted(!mounted)}>Toggle scene</button>
    <label>Notes <input aria-label="Notes"/></label>
    <label>Throttle <input aria-label="Throttle" type="range" min="-100" max="100" defaultValue="0"
      onChange={event => { clock.state = { ...clock.state, throttle: Number(event.target.value) } }}/></label>
    <div style={{ height: '70vh' }}>{mounted && <Canvas dpr={[1, 2]}><Scene/></Canvas>}</div>
  </main>
}
createRoot(document.getElementById('root')!).render(<Fixture/>)
