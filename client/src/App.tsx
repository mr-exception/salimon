import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import * as THREE from 'three'
import { SAMPLE_BODIES, SCANNER_RADIUS_LS, SCAN_INTERVAL_SECONDS, length, normal, type Body, type Vec3 } from './sampleUniverse'

import { LIGHT_SECOND_M, radarPosition, relativeM } from './coordinates.ts'
import { NavigationClock, alignTo, forward, idleControls, setThrottle as changeThrottle, type Controls, type ShipState } from './simulation.ts'
type Pose = ShipState
type Edge = { id: string; name: string; x: number; y: number; distance: number }
const VISIBLE_RADIUS = 900
const radians = (v: number) => (v * 180 / Math.PI).toFixed(1)
const fmt = (v: number) => Math.abs(v) >= 1000 ? Math.round(v).toLocaleString() : v.toFixed(1)
function CameraControls({ onViewRadius }: { onViewRadius: (value: number) => void }) {
  const { camera, gl } = useThree()
  const controlsRef = useRef<OrbitControls|null>(null)
  useEffect(() => {
    camera.position.set(800, 500, 800)
    camera.lookAt(0,0,0)
    camera.updateProjectionMatrix()
    const controls = new OrbitControls(camera, gl.domElement)
    controls.enablePan = false
    controls.enableDamping = true
    controlsRef.current = controls
    controls.minZoom = 0.001
    controls.maxZoom = 30
    controls.target.set(0,0,0)
    const sync = () => {
      if (camera instanceof THREE.OrthographicCamera) {
        onViewRadius((camera.top - camera.bottom) / (2 * camera.zoom))
      }
    }
    controls.addEventListener('change',sync)
    sync()
    return () => { controls.removeEventListener('change',sync); controls.dispose();controlsRef.current=null }
  },[camera,gl,onViewRadius])
  useFrame(() => controlsRef.current?.update())
  return null
}
function Contact({ body, position, selected, onSelect }: { body: Body; position: Vec3; selected: boolean; onSelect: (id: string) => void }) {
  const color = selected ? '#dfffe0' : body.kind === 'star' ? '#f6cb79' : body.kind === 'planet' ? '#6cddd3' : body.kind === 'moon' ? '#adc3de' : '#db8bd5'
  const radius = body.kind === 'star' ? 18 : body.kind === 'planet' ? 12 : body.kind === 'moon' ? 8 : 6
  return <group position={position as [number,number,number]} onClick={(event) => {event.stopPropagation();onSelect(body.id)}}>
    <mesh>
      <sphereGeometry args={[radius, 12, 8]} />
      <meshBasicMaterial color={color} wireframe transparent opacity={selected ? 1 : 0.75} depthTest={false}/>
    </mesh>
    <mesh>
      <sphereGeometry args={[radius * 1.6, 8, 6]} />
      <meshBasicMaterial transparent opacity={0} depthWrite={false}/>
    </mesh>
    {selected && <mesh rotation={[Math.PI/2,0,0]}>
      <torusGeometry args={[radius*1.7,0.4,4,32]}/>
      <meshBasicMaterial color="#efffc3" depthTest={false}/>
    </mesh>}
  </group>
}
function RadarScene({ pose, contacts, selected, select, viewRadius, setViewRadius, setEdges }: { pose: Pose; contacts: readonly Body[]; selected: string|null; select:(id:string)=>void; viewRadius:number; setViewRadius:(radius:number)=>void; setEdges:(edges:Edge[])=>void }) {
  const { camera } = useThree()
  const elapsed = useRef(0)
  useFrame((_state,dt) => {
    elapsed.current += dt
    if (elapsed.current < 0.2) return
    elapsed.current = 0
    const next: Edge[] = []
    for (const body of contacts) {
      const relative = radarPosition(body.position, pose.position)
      const projected = new THREE.Vector3(...relative).project(camera)
      const outside = Math.abs(projected.x)>0.9 || Math.abs(projected.y)>0.88 || projected.z>1 || projected.z< -1 || length(relative)>viewRadius
      if (!outside) continue
      const x = THREE.MathUtils.clamp(projected.x,-0.9,0.9)
      const y = THREE.MathUtils.clamp(projected.y,-0.83,0.83)
      next.push({id:body.id,name:body.name,x:(x+1)*50,y:(1-y)*50,distance:length(relative)})
    }
    setEdges(next)
  })
  const heading = forward(pose.yaw,pose.pitch)
  return <>
    <color attach="background" args={['#060f19']}/>
    <CameraControls onViewRadius={setViewRadius} />
    <gridHelper args={[1800,18,'#2a6b79','#183645']} rotation={[0,0,0]} />
    <axesHelper args={[65]}/>
    <group rotation={[pose.pitch,pose.yaw,0]} rotation-order="YXZ">
      <mesh rotation={[-Math.PI/2,0,0]}><coneGeometry args={[12,36,5]} /><meshBasicMaterial color="#65efc6" wireframe /></mesh>
      <mesh position={[0,0,7]}><boxGeometry args={[20,3,10]}/><meshBasicMaterial color="#84a9bd" wireframe /></mesh>
    </group>
    <arrowHelper args={[new THREE.Vector3(...heading),new THREE.Vector3(),65,'#69f4c2',8,5]}/>
    {length(pose.velocityMps)>0.000001 && <arrowHelper args={[new THREE.Vector3(...normal(pose.velocityMps)),new THREE.Vector3(),85,'#f0b763',10,6]}/>}
    {contacts.map(body=><Contact key={body.id} body={body} position={radarPosition(body.position,pose.position)} selected={selected===body.id} onSelect={select}/>)}
  </>
}
export function App() {
  const clock=useRef(new NavigationClock())
  const [pose,setPose]=useState<Pose>(clock.current.state)
  const [throttle,setThrottle]=useState(0)
  const [contacts,setContacts]=useState<readonly Body[]>([])
  const [selected,setSelected]=useState<string|null>(null)
  const [edges,setEdges]=useState<Edge[]>([])
  const [viewRadius,setViewRadius]=useState(VISIBLE_RADIUS)
  const [pulse,setPulse]=useState(0)
  const [aligning,setAligning]=useState(false)
  const controls=useRef<Controls>(idleControls())

  const throttleRef=useRef(throttle)

  const scannerTimer=useRef(0)
  const setEdgesStable=useCallback((e:Edge[])=>setEdges(e),[])
  useEffect(()=>{throttleRef.current=throttle},[throttle])
  const select=useCallback((id:string)=>setSelected(id),[])
  useEffect(()=>{
    const onKeyDown=(e:KeyboardEvent)=>{
      const key=e.key.toLowerCase()
      const axis=({w:'up',s:'down',a:'left',d:'right'} as const)[key as 'w'|'s'|'a'|'d']
      if(axis && !e.repeat){controls.current[axis]=true;clock.current.state={...clock.current.state,alignment:null};setAligning(false);e.preventDefault()}
      if(e.code==='Space' && !e.repeat){setThrottle(0);e.preventDefault()}
    }
    const onKeyUp=(e:KeyboardEvent)=>{const axis=({w:'up',s:'down',a:'left',d:'right'} as const)[e.key.toLowerCase() as 'w'|'s'|'a'|'d'];if(axis) controls.current[axis]=false}
    const blur=()=>{controls.current={up:false,down:false,left:false,right:false}}
    window.addEventListener('keydown',onKeyDown)
    window.addEventListener('keyup',onKeyUp)
    window.addEventListener('blur',blur)
    return ()=>{window.removeEventListener('keydown',onKeyDown);window.removeEventListener('keyup',onKeyUp);window.removeEventListener('blur',blur)}
  },[])
  useEffect(()=>{
    let frame=0
    let previous=performance.now()
    let accumulator=0
    const resume=()=>{previous=performance.now()}
    document.addEventListener('visibilitychange',resume)
    const animate=(now:number)=>{
      const dt=document.hidden?0:(now-previous)/1000;previous=now
      clock.current.state=changeThrottle(clock.current.state,throttleRef.current)
      const p=clock.current.advance(dt,controls.current)
      setAligning(p.alignment!==null)
      scannerTimer.current+=dt
      if(scannerTimer.current>=SCAN_INTERVAL_SECONDS){scannerTimer.current %= SCAN_INTERVAL_SECONDS;setPulse(v=>v+1);setContacts(SAMPLE_BODIES.filter(body=>length(relativeM(body.position,p.position))/LIGHT_SECOND_M<=SCANNER_RADIUS_LS))}
      accumulator+=dt
      if(accumulator>=0.08){setPose(clock.current.state);accumulator=0}
      frame=requestAnimationFrame(animate)
    }
    setContacts(SAMPLE_BODIES.filter(body=>length(relativeM(body.position,clock.current.state.position))/LIGHT_SECOND_M<=SCANNER_RADIUS_LS))
    frame=requestAnimationFrame(animate)
    return ()=>{cancelAnimationFrame(frame);document.removeEventListener('visibilitychange',resume)}
  },[])
  useEffect(()=>{if(selected && !contacts.some(body=>body.id===selected))setSelected(null)},[contacts,selected])
  const target=useMemo(()=>contacts.find(body=>body.id===selected)??null,[contacts,selected])
  const setDirection=()=>{
    if(!target)return
    clock.current.state=alignTo(clock.current.state,relativeM(target.position,clock.current.state.position))
    setAligning(true)
  }
  const hold=(axis:keyof Controls)=>({
    onPointerDown:(event:import('react').PointerEvent<HTMLButtonElement>)=>{event.currentTarget.setPointerCapture(event.pointerId);controls.current[axis]=true;clock.current.state={...clock.current.state,alignment:null};setAligning(false)},
    onPointerUp:()=>{controls.current[axis]=false},
    onPointerCancel:()=>{controls.current[axis]=false},
    onLostPointerCapture:()=>{controls.current[axis]=false}
  })
  const distance=target?length(relativeM(target.position,pose.position))/LIGHT_SECOND_M:null
  return <div className="app">
    <div className="radar">
      <Canvas orthographic camera={{position:[800,500,800],zoom:0.5,near:0.1,far:300000}}>
        <RadarScene pose={pose} contacts={contacts} selected={selected} select={select} viewRadius={viewRadius} setViewRadius={setViewRadius} setEdges={setEdgesStable}/>
      </Canvas>
      <div className="top-label">SALIMON <span>3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE</span></div>
      <div className="edge-layer">{edges.map(e=><button key={e.id} className="edge" style={{left:e.x+'%',top:e.y+'%'}} onClick={()=>select(e.id)} title={e.name+' · '+fmt(e.distance)+' light-seconds'}>◇ {e.name}</button>)}</div>
      <div className="legend">▲ HEADING <span>◆ VELOCITY</span> · Scanner 100,000 light-seconds · pulse #{pulse}</div>
    </div>
    <aside>
      <h1>NAVIGATION</h1><p className="muted">Navigation-only prototype · unlimited energy</p>
      <div className="stat"><span>Relative position</span><strong>{pose.position.offsetM.map(fmt).join(' / ')} m (cell local)</strong></div>
      <div className="stat"><span>Speed (sample inertial frame)</span><strong>{fmt(length(pose.velocityMps))} m/s</strong></div>
      <div className="stat"><span>Yaw / pitch</span><strong>{radians(pose.yaw)}° / {radians(pose.pitch)}°</strong></div>
      <div className="stat"><span>Alignment</span><strong>{aligning?'Turning':'Manual'}</strong></div>
      <hr/>
      <h2>Contact</h2>
      {target?<><strong>{target.name}</strong><p className="muted">{target.kind} · {fmt(distance??0)} light-seconds</p><button className="action" onClick={setDirection}>Set direction</button></>:<p className="muted">Select a holographic contact or edge marker.</p>}
      <hr/>
      <h2>Manual orientation</h2>
      <div className="pad"><button {...hold('up')}>↑</button><div><button {...hold('left')}>←</button><button {...hold('down')}>↓</button><button {...hold('right')}>→</button></div></div>
      <p className="muted">Hold W/A/S/D or buttons. Camera drag is independent.</p>
      <hr/>
      <h2>Throttle <strong>{throttle>0?'+':''}{throttle}%</strong></h2>
      <input type="range" min="-100" max="100" step="1" value={throttle} onChange={e=>setThrottle(Number(e.target.value))}/>
      <button onClick={()=>setThrottle(0)}>Zero throttle (Space)</button>
      <p className="muted foot">SI fixed-step motion · split galactic coordinates · illustrative catalog. Gravity, landing, collisions, jump drive, Core limits and 2D gameplay are deferred.</p>
    </aside>
  </div>
}
