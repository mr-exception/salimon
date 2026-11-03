import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import * as THREE from 'three'
import { SAMPLE_BODIES, length, normal, type Body, type Vec3 } from './sampleUniverse'

import { LIGHT_SECOND_M, relativeM } from './coordinates.ts'
import { alignTo, forward, setThrottle as changeThrottle, type ShipState } from './simulation.ts'
type Pose = ShipState
import { NavigationInput, FrameTimer, acceptsFlightShortcut, axisForCode, type Axis } from './input.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { distances, navigationWarnings } from './contact.ts'
import { radarGroups, gridOffsetLs, type ContactGroup } from './radar.ts'
const VISIBLE_RADIUS = 900
const RADAR_CAMERA = {position:[200000,125000,200000] as [number,number,number],zoom:0.5,near:0.1,far:1000000}
const radians = (v: number) => (v * 180 / Math.PI).toFixed(1)
const fmt = (v: number) => Math.abs(v) >= 1000 ? Math.round(v).toLocaleString() : v.toFixed(1)
type ViewCommand={action:'reset'|'in'|'out';sequence:number}
function CameraControls({ onViewRadius, command }: { onViewRadius: (value: number) => void; command:ViewCommand }) {
  const { camera, gl } = useThree()
  const controlsRef = useRef<OrbitControls|null>(null)
  useEffect(() => {
    camera.position.set(200000, 125000, 200000)
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
  useEffect(()=>{
    if(!(camera instanceof THREE.OrthographicCamera))return
    if(command.action==='reset'){camera.position.set(200000,125000,200000);camera.lookAt(0,0,0);camera.zoom=0.5}
    else camera.zoom=THREE.MathUtils.clamp(camera.zoom*(command.action==='in'?1.5:1/1.5),0.001,30)
    camera.updateProjectionMatrix();controlsRef.current?.update()
    onViewRadius((camera.top-camera.bottom)/(2*camera.zoom))
  },[camera,command,onViewRadius])
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
function RadarScene({ pose, contacts, selected, selectGroup, viewRadius, setViewRadius, setGroups, viewCommand }: { pose: Pose; contacts: readonly Body[]; selected: string|null; selectGroup:(group:ContactGroup)=>void; viewRadius:number; setViewRadius:(radius:number)=>void; setGroups:(groups:ContactGroup[])=>void; viewCommand:ViewCommand }) {
  const { camera, size } = useThree()
  const [groups,updateGroups]=useState<ContactGroup[]>([])
  const elapsed = useRef(0)
  useFrame((_state,dt) => {
    elapsed.current += dt
    if (elapsed.current < 0.1) return
    elapsed.current = 0
    const next=radarGroups(contacts,pose.position,p=>{
      const v=new THREE.Vector3(...p).project(camera);return [v.x,v.y,v.z]
    },size.width,size.height)
    updateGroups(next);setGroups(next)
  })
  const spacing=10**Math.floor(Math.log10(Math.max(1,viewRadius/5)))
  const gridOffset=gridOffsetLs(pose.position,spacing)
  const heading = forward(pose.yaw,pose.pitch)
  return <>
    <color attach="background" args={['#060f19']}/>
    <CameraControls onViewRadius={setViewRadius} command={viewCommand} />
    <gridHelper args={[spacing*100,100,'#2a6b79','#183645']} position={[gridOffset[0],0,gridOffset[2]]} />
    <axesHelper args={[65]}/>
    <group rotation={[pose.pitch,pose.yaw,0]} rotation-order="YXZ">
      <mesh rotation={[-Math.PI/2,0,0]}><coneGeometry args={[12,36,5]} /><meshBasicMaterial color="#65efc6" wireframe /></mesh>
      <mesh position={[0,0,7]}><boxGeometry args={[20,3,10]}/><meshBasicMaterial color="#84a9bd" wireframe /></mesh>
    </group>
    <arrowHelper args={[new THREE.Vector3(...heading),new THREE.Vector3(),65,'#69f4c2',8,5]}/>
    {length(pose.velocityMps)>0.000001 && <arrowHelper args={[new THREE.Vector3(...normal(pose.velocityMps)),new THREE.Vector3(),85,'#f0b763',10,6]}/>}
    {groups.filter(g=>!g.edge).map(g=><Contact key={g.id} body={g.bodies[0]} position={g.position} selected={g.bodies.some(b=>selected===b.id)} onSelect={()=>selectGroup(g)}/>)}
  </>
}
export function App({navigation}:{navigation?:NavigationRuntime}={}) {
  const [runtimeInstance]=useState(()=>navigation??new NavigationRuntime(SAMPLE_BODIES))
  const runtime=useRef(runtimeInstance)
  const [pose,setPose]=useState<Pose>(runtime.current.clock.state)
  const [throttle,setThrottle]=useState(0)
  const [contacts,setContacts]=useState<readonly Body[]>([])
  const [selected,setSelected]=useState<string|null>(null)
  const [groups,setGroups]=useState<ContactGroup[]>([])
  const [menu,setMenu]=useState<readonly string[]>([])
  const [menuPage,setMenuPage]=useState(0)
  const [viewRadius,setViewRadius]=useState(VISIBLE_RADIUS)
  const [pulse,setPulse]=useState(0)
  const [aligning,setAligning]=useState(false)
  const input=useRef(new NavigationInput())
  const [held,setHeld]=useState(input.current.controls)
  const [paused,setPaused]=useState(document.hidden)
  const [viewCommand,setViewCommand]=useState<ViewCommand>({action:'reset',sequence:0})
  const [contactPage,setContactPage]=useState(0)
  const [contactQuery,setContactQuery]=useState('')

  const throttleRef=useRef(throttle)

  const setGroupsStable=useCallback((g:ContactGroup[])=>setGroups(g),[])
  useEffect(()=>{throttleRef.current=throttle},[throttle])
  const select=useCallback((id:string)=>{runtime.current.scanner.select(id);setSelected(runtime.current.scanner.selected);setMenu([])},[])
  const selectGroup=useCallback((g:ContactGroup)=>{
    if(g.bodies.length===1){runtime.current.scanner.select(g.bodies[0].id);setSelected(runtime.current.scanner.selected);setMenu([])}
    else {setMenu(g.bodies.map(b=>b.id));setMenuPage(0)}
  },[])
  const cancelAlignment=()=>{runtime.current.clock.state={...runtime.current.clock.state,alignment:null};setAligning(false)}
  useEffect(()=>{
    const clear=()=>{input.current.clear();setHeld(input.current.controls);cancelAlignment()}
    const onKeyDown=(e:KeyboardEvent)=>{
      if(!acceptsFlightShortcut(e.target)||e.altKey||e.ctrlKey||e.metaKey)return
      const axis=axisForCode(e.code)
      if(axis){input.current.press(`key:${e.code}`,axis);cancelAlignment();e.preventDefault()}
      if(e.code==='Space' && !e.repeat){throttleRef.current=0;setThrottle(0);e.preventDefault()}
    }
    const onKeyUp=(e:KeyboardEvent)=>input.current.release(`key:${e.code}`)
    const visibility=()=>{clear();setPaused(document.hidden)}
    const focus=(e:FocusEvent)=>{if(!acceptsFlightShortcut(e.target))input.current.releasePrefix('key:')}
    window.addEventListener('keydown',onKeyDown);window.addEventListener('keyup',onKeyUp)
    window.addEventListener('blur',clear);window.addEventListener('resize',clear);window.addEventListener('focusin',focus)
    document.addEventListener('visibilitychange',visibility)
    return ()=>{window.removeEventListener('keydown',onKeyDown);window.removeEventListener('keyup',onKeyUp);window.removeEventListener('blur',clear);window.removeEventListener('resize',clear);window.removeEventListener('focusin',focus);document.removeEventListener('visibilitychange',visibility)}
  },[])
  useEffect(()=>{
    let frame=0
    const timer=new FrameTimer(performance.now())
    let accumulator=0
    const resume=()=>timer.resume(performance.now())
    document.addEventListener('visibilitychange',resume)
    const animate=(now:number)=>{
      const dt=timer.sample(now,document.hidden)
      if(document.hidden){frame=requestAnimationFrame(animate);return}
      runtime.current.clock.state=changeThrottle(runtime.current.clock.state,throttleRef.current)
      const p=runtime.current.advance(dt,input.current.controls)
      if(p.mode!=='flying'){throttleRef.current=0;setThrottle(0)}
      setAligning(p.alignment!==null)
      accumulator+=dt
      if(accumulator>=0.08){
        setHeld(input.current.controls);setPose(p);setContacts(runtime.current.scanner.contacts);setSelected(runtime.current.scanner.selected);setPulse(runtime.current.scanner.pulse);accumulator=0
      }
      frame=requestAnimationFrame(animate)
    }
    frame=requestAnimationFrame(animate)
    return ()=>{cancelAnimationFrame(frame);document.removeEventListener('visibilitychange',resume)}
  },[])
  useEffect(()=>{if(selected && !contacts.some(body=>body.id===selected))setSelected(null)},[contacts,selected])
  const menuIds=new Set(menu)
  const menuBodies=contacts.filter(b=>menuIds.has(b.id))
  const target=useMemo(()=>contacts.find(body=>body.id===selected)??null,[contacts,selected])
  const setDirection=()=>{
    if(!target)return
    runtime.current.clock.state=alignTo(runtime.current.clock.state,relativeM(target.position,runtime.current.clock.state.position))
    setAligning(true)
  }
  const hold=(axis:Axis)=>({
    disabled:pose.mode!=='flying',
    'aria-pressed':held[axis],
    onPointerDown:(event:import('react').PointerEvent<HTMLButtonElement>)=>{
      if(event.button!==0)return
      event.currentTarget.setPointerCapture(event.pointerId);input.current.press(`pointer:${event.pointerId}`,axis);cancelAlignment()
    },
    onPointerUp:(event:import('react').PointerEvent<HTMLButtonElement>)=>input.current.release(`pointer:${event.pointerId}`),
    onPointerCancel:(event:import('react').PointerEvent<HTMLButtonElement>)=>input.current.release(`pointer:${event.pointerId}`),
    onLostPointerCapture:(event:import('react').PointerEvent<HTMLButtonElement>)=>input.current.release(`pointer:${event.pointerId}`),
    onKeyDown:(event:import('react').KeyboardEvent<HTMLButtonElement>)=>{if(event.code==='Space'||event.code==='Enter'){event.preventDefault();input.current.press(`button:${axis}:${event.code}`,axis);cancelAlignment()}},
    onKeyUp:(event:import('react').KeyboardEvent<HTMLButtonElement>)=>{if(event.code==='Space'||event.code==='Enter'){event.preventDefault();input.current.release(`button:${axis}:${event.code}`)}},
    onBlur:()=>input.current.releasePrefix(`button:${axis}:`),
    onClick:(event:import('react').MouseEvent<HTMLButtonElement>)=>{if(event.detail===0){input.current.press(`activate:${axis}`,axis);cancelAlignment();setTimeout(()=>input.current.release(`activate:${axis}`),150)}}
  })
  const restart=()=>{runtime.current.reset();throttleRef.current=0;setThrottle(0);input.current.clear();setHeld(input.current.controls);setPose(runtime.current.clock.state);setContacts([]);setSelected(null);setGroups([]);setMenu([]);setMenuPage(0);setContactQuery('');setContactPage(0);setAligning(false);setPulse(0)}
  const filteredContacts=contacts.filter(b=>b.name.toLowerCase().includes(contactQuery.toLowerCase()))
  const contactPages=Math.max(1,Math.ceil(filteredContacts.length/32))
  const page=Math.min(contactPage,contactPages-1)
  const pageContacts=filteredContacts.slice(page*32,(page+1)*32)
  const view=(action:ViewCommand['action'])=>setViewCommand(c=>({action,sequence:c.sequence+1}))
  const warnings=navigationWarnings(pose,contacts)
  const targetDistances=target?distances(pose,target):null
  const distance=target?length(relativeM(target.position,pose.position))/LIGHT_SECOND_M:null
  return <div className="app" data-mode={pose.mode}>
    <div className="radar" role="region" aria-label="Navigation radar" aria-describedby="navigation-help" tabIndex={0} onPointerDown={e=>{if(e.target instanceof HTMLCanvasElement)e.currentTarget.focus({preventScroll:true})}}>
      <Canvas orthographic camera={RADAR_CAMERA} onCreated={({gl})=>{gl.domElement.setAttribute('aria-label','Ship-centered three-dimensional radar');gl.domElement.setAttribute('role','img')}} fallback={<p role="alert">WebGL is unavailable. A WebGL 2 browser is required.</p>}>
        <RadarScene pose={pose} contacts={contacts} selected={selected} selectGroup={selectGroup} viewRadius={viewRadius} setViewRadius={setViewRadius} setGroups={setGroupsStable} viewCommand={viewCommand}/>
      </Canvas>
      <div className="top-label">SALIMON <span>3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE</span></div>
      <div className="edge-layer">{groups.filter(g=>g.edge||g.bodies.length>1).map(g=><button key={g.id} className="edge" style={{left:`clamp(var(--edge-inset), ${g.x}%, calc(100% - var(--edge-inset)))`,top:g.y+'%'}} onClick={()=>selectGroup(g)} title={g.bodies.map(b=>b.name).slice(0,3).join(', ')}>{g.edge?'◇':'◉'} {g.bodies.length>1?`${g.bodies.length} contacts`:g.bodies[0].name}</button>)}</div>
      <div className="legend">▲ HEADING <span>◆ VELOCITY</span> · Scanner 100,000 light-seconds · pulse #{pulse}</div>
    </div>
    <aside>
      <h1>NAVIGATION</h1><p className="muted">Navigation-only prototype · unlimited energy</p>
      <p className="scan-status" role="status" aria-live="polite">{paused?'Navigation paused while hidden':pulse===0?'Scanner: awaiting first five-second pulse':`Scanner pulse ${pulse}: ${contacts.length} temporary contacts`}</p>
      <div className="stat"><span>Total ship mass</span><strong>{fmt(pose.massKg)} kg</strong></div>
      <div className="stat"><span>Relative position</span><strong>{pose.position.offsetM.map(fmt).join(' / ')} m (cell local)</strong></div>
      <div className="stat"><span>Speed (sample inertial frame)</span><strong>{fmt(length(pose.velocityMps))} m/s</strong></div>
      <div className="stat"><span>Yaw / pitch</span><strong>{radians(pose.yaw)}° / {radians(pose.pitch)}°</strong></div>
      <div className="stat"><span>Alignment</span><strong>{aligning?'Turning':'Manual'}</strong></div>
      <div className="stat"><span>Flight status</span><strong>{pose.mode}</strong></div>
      {pose.contact && <div role="status" className="contact-status">
        {pose.mode==='destroyed'?'Journey ended — ship destroyed':pose.contact.outcome==='safe-contact'?'Safe surface contact':'Crash recovery'} · impact {fmt(pose.contact.impactSpeedMps)} m/s
        {(pose.mode==='destroyed'||pose.mode==='grounded') && <button onClick={restart}>Restart navigation</button>}
      </div>}
      <div className="warnings" role="status" aria-label="Navigation warnings">{warnings.length===0?<p>No navigation warnings</p>:warnings.map(w=><p key={w.id}>{w.message}</p>)}</div>
      <hr/>
      <h2>Contact</h2>
      <p role="status" aria-live="polite" className="target-status">{target?`Selected target: ${target.name}`:'No selected target'}</p>
      <label htmlFor="contact-search">Find scanned contacts</label>
      <input id="contact-search" type="search" value={contactQuery} onChange={e=>{setContactQuery(e.target.value);setContactPage(0)}}/>
      <label htmlFor="contact-select">Scanned contacts ({contacts.length})</label>
      <select id="contact-select" value={pageContacts.some(b=>b.id===selected)?selected??'':''} onChange={e=>{if(e.target.value)select(e.target.value)}}>
        <option value="">Choose a contact</option>
        {pageContacts.map(b=><option key={b.id} value={b.id}>{b.name}</option>)}
      </select>
      {contactPages>1 && <div className="contact-pages"><button disabled={page===0} onClick={()=>setContactPage(page-1)}>Previous scanned contacts</button><span>{page+1}/{contactPages}</span><button disabled={page+1===contactPages} onClick={()=>setContactPage(page+1)}>Next scanned contacts</button></div>}
      {menuBodies.length>0 && <div className="contact-menu" aria-label="Grouped contacts">
        <p>{menuBodies.length} contacts · page {menuPage+1}</p>
        {menuBodies.slice(menuPage*32,(menuPage+1)*32).map(b=><button key={b.id} onClick={()=>select(b.id)}>{b.name}</button>)}
        <button disabled={menuPage===0} onClick={()=>setMenuPage(p=>p-1)}>Previous contacts</button>
        <button disabled={(menuPage+1)*32>=menuBodies.length} onClick={()=>setMenuPage(p=>p+1)}>Next contacts</button>
      </div>}
      {target?<><strong>{target.name}</strong><p className="muted">{target.kind} · {fmt(distance??0)} light-seconds</p><p className="muted">Surface clearance {fmt(targetDistances?.surfaceDistanceM??0)} m</p><button className="action" disabled={pose.mode!=='flying'} onClick={setDirection}>Set direction</button></>:<p className="muted">Select a holographic contact or edge marker.</p>}
      <hr/>
      <h2>Manual orientation</h2>
      <div className="pad"><button aria-label="Pitch up" {...hold('up')}>↑</button><div><button aria-label="Yaw left" {...hold('left')}>←</button><button aria-label="Pitch down" {...hold('down')}>↓</button><button aria-label="Yaw right" {...hold('right')}>→</button></div></div>
      <p className="muted" id="navigation-help">Focus the radar for W/A/S/D and Space. Hold buttons with pointer, Space or Enter. Camera drag is independent.</p>
      <div className="camera-actions"><button onClick={()=>view('in')}>Zoom in</button><button onClick={()=>view('out')}>Zoom out</button><button onClick={()=>view('reset')}>Reset camera</button></div>
      <hr/>
      <h2><label htmlFor="throttle">Throttle</label> <strong>{throttle>0?'+':''}{throttle}%</strong></h2>
      <input id="throttle" aria-valuetext={`${throttle}% ${throttle>0?'forward':throttle<0?'reverse':'coast'}`} type="range" min="-100" max="100" step="1" value={throttle} disabled={pose.mode!=='flying'} onChange={e=>{const value=Number(e.target.value);throttleRef.current=value;setThrottle(value)}}/>
      <button disabled={pose.mode!=='flying'} onClick={()=>{throttleRef.current=0;setThrottle(0)}}>Zero throttle (Space)</button>
      <p className="muted foot">Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run. Lower-speed contact settles the ship.</p>
    </aside>
  </div>
}
