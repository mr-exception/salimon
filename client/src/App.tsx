import { Canvas, type RootState } from '@react-three/fiber'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { CameraControls } from './CameraControls'
import { HelpDialog } from './HelpDialog'
import { ShipVectorGuides, type VectorElements } from './ShipVectorGuides'
import { STATIONARY_SPEED_MPS } from './vectorGuides'
import { SAMPLE_BODIES, length, type Body } from './sampleUniverse'

import { LIGHT_SECOND_M, relativeM } from './coordinates.ts'
import { alignTo, setThrottle as changeThrottle, type ShipState } from './simulation.ts'
type Pose = ShipState
import { NavigationInput, FrameTimer, acceptsFlightShortcut, axisForCode, panAxisForCode } from './input.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { distances, navigationWarnings } from './contact.ts'
import { gridOffsetLs, type ContactGroup } from './radar.ts'
import { GalacticLattice } from './navigationVisuals.tsx'
import { ContinuousContacts } from './ContinuousContacts'
const VISIBLE_RADIUS = 900
const RADAR_CAMERA = {position:[200000,125000,200000] as [number,number,number],zoom:0.5,near:0.1,far:1000000}
const radians = (v: number) => (v * 180 / Math.PI).toFixed(1)
const fmt = (v: number) => Math.abs(v) >= 1000 ? Math.round(v).toLocaleString() : v.toFixed(1)
function RadarScene({ pose, contacts, selected, selectGroup, viewRadius, setViewRadius, setGroups, viewCommand, panInput, navigation, vectorElements }: { pose: Pose; contacts: readonly Body[]; selected: string|null; selectGroup:(group:ContactGroup)=>void; viewRadius:number; setViewRadius:(radius:number)=>void; setGroups:(groups:ContactGroup[])=>void; viewCommand:number; panInput:NavigationInput;navigation:NavigationRuntime;vectorElements:VectorElements }) {
  const spacing=10**Math.floor(Math.log10(Math.max(1,viewRadius/5)))
  const gridOffset=gridOffsetLs(pose.position,spacing)
  return <>
    <color attach="background" args={['#060f19']}/>
    <CameraControls onViewRadius={setViewRadius} resetSequence={viewCommand} input={panInput} />
    <ambientLight intensity={0.65}/>
    <directionalLight position={[100,180,-80]} intensity={2}/>
    <GalacticLattice spacing={spacing} offset={gridOffset}/>
    <ShipVectorGuides navigation={navigation} elements={vectorElements}/>
    <ContinuousContacts navigation={navigation} contacts={contacts} selected={selected} selectGroup={selectGroup} setGroups={setGroups}/>

  </>
}
export function App({navigation,onSceneReady}:{navigation?:NavigationRuntime;onSceneReady?:(state:RootState)=>void}={}) {
  const [runtimeInstance]=useState(()=>navigation??new NavigationRuntime(SAMPLE_BODIES))
  const runtime=useRef(runtimeInstance)
  const [pose,setPose]=useState<Pose>(runtime.current.clock.state)
  const [throttle,setThrottle]=useState(runtime.current.clock.state.throttle)
  const [contacts,setContacts]=useState<readonly Body[]>(runtime.current.scanner.contacts)
  const [selected,setSelected]=useState<string|null>(runtime.current.scanner.selected)
  const [groups,setGroups]=useState<ContactGroup[]>([])
  const [menu,setMenu]=useState<readonly string[]>([])
  const [menuPage,setMenuPage]=useState(0)
  const [viewRadius,setViewRadius]=useState(VISIBLE_RADIUS)
  const [pulse,setPulse]=useState(0)
  const [aligning,setAligning]=useState(false)
  const input=useRef(new NavigationInput())
  const panInput=useRef(new NavigationInput())
  const headingLine=useRef<SVGLineElement>(null),velocityLine=useRef<SVGLineElement>(null)
  const vectorElements:VectorElements=[headingLine,velocityLine]
  const [helpOpen,setHelpOpen]=useState(false)
  const [paused,setPaused]=useState(document.hidden)
  const [viewCommand,setViewCommand]=useState(0)
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
    const clear=()=>{input.current.clear();panInput.current.clear()}
    const onKeyDown=(e:KeyboardEvent)=>{
      if(!acceptsFlightShortcut(e.target)||e.altKey||e.ctrlKey||e.metaKey)return
      const pan=panAxisForCode(e.code)
      if(pan){panInput.current.press(`key:${e.code}`,pan);e.preventDefault()}
      const axis=axisForCode(e.code)
      if(axis){input.current.press(`key:${e.code}`,axis);cancelAlignment();e.preventDefault()}
      if(e.code==='Space' && !e.repeat){throttleRef.current=0;setThrottle(0);e.preventDefault()}
    }
    const onKeyUp=(e:KeyboardEvent)=>{input.current.release(`key:${e.code}`);panInput.current.release(`key:${e.code}`)}
    const visibility=()=>{clear();setPaused(document.hidden)}
    const focus=(e:FocusEvent)=>{if(!acceptsFlightShortcut(e.target)){input.current.releasePrefix('key:');panInput.current.clear()}}
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
        setPose(p);setContacts(runtime.current.scanner.contacts);setSelected(runtime.current.scanner.selected);setPulse(runtime.current.scanner.pulse);accumulator=0
      }
      frame=requestAnimationFrame(animate)
    }
    frame=requestAnimationFrame(animate)
    return ()=>{runtime.current.clock.state=changeThrottle(runtime.current.clock.state,throttleRef.current);cancelAnimationFrame(frame);document.removeEventListener('visibilitychange',resume)}
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
  const restart=()=>{runtime.current.reset();throttleRef.current=0;setThrottle(0);input.current.clear();panInput.current.clear();setPose(runtime.current.clock.state);setContacts([]);setSelected(null);setGroups([]);setMenu([]);setMenuPage(0);setContactQuery('');setContactPage(0);setAligning(false);setPulse(0)}
  const filteredContacts=contacts.filter(b=>b.name.toLowerCase().includes(contactQuery.toLowerCase()))
  const contactPages=Math.max(1,Math.ceil(filteredContacts.length/32))
  const page=Math.min(contactPage,contactPages-1)
  const pageContacts=filteredContacts.slice(page*32,(page+1)*32)
  const resetView=()=>{panInput.current.clear();setViewCommand(c=>c+1)}
  const warnings=navigationWarnings(pose,contacts)
  const targetDistances=target?distances(pose,target):null
  const distance=target?length(relativeM(target.position,pose.position))/LIGHT_SECOND_M:null
  return <div className="app" data-mode={pose.mode}>
    <div className="radar" role="region" aria-label="Navigation radar" aria-describedby="navigation-help" tabIndex={0} onPointerDown={e=>{if(e.target instanceof HTMLCanvasElement)e.currentTarget.focus({preventScroll:true})}}>
      <Canvas orthographic camera={RADAR_CAMERA} onCreated={(state)=>{onSceneReady?.(state);const {gl}=state;gl.domElement.setAttribute('aria-label','Ship-centered three-dimensional radar');gl.domElement.setAttribute('role','img')}} fallback={<p role="alert">WebGL is unavailable. A WebGL 2 browser is required.</p>}>
        <RadarScene pose={pose} contacts={contacts} selected={selected} selectGroup={selectGroup} viewRadius={viewRadius} setViewRadius={setViewRadius} setGroups={setGroupsStable} viewCommand={viewCommand} panInput={panInput.current} navigation={runtime.current} vectorElements={vectorElements}/>
      </Canvas>
      <div className="top-label">SALIMON <span>3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE</span></div>
      <svg className="ship-vectors" viewBox="0 0 100 100" preserveAspectRatio="none" role="img" aria-label="Ship heading and velocity vector lines">
        <line ref={headingLine} data-vector="heading" vectorEffect="non-scaling-stroke" style={{display:'none'}}/>
        <line ref={velocityLine} data-vector="velocity" vectorEffect="non-scaling-stroke" style={{display:'none'}}/>
      </svg>
      <div className="edge-layer">{groups.filter(g=>g.bodies.length>1).map(g=><button key={g.id} className="edge" style={{left:g.x+'%',top:g.y+'%'}} onClick={()=>selectGroup(g)}>◉ {g.bodies.length} contacts</button>)}</div>
      <div className="legend"><b>HEADING</b> <span>{length(pose.velocityMps)<=STATIONARY_SPEED_MPS?'STATIONARY':'VELOCITY · dashed'}</span> · Scanner 100,000 light-seconds · pulse #{pulse}</div>
    </div>
    <aside>
      <h1>NAVIGATION</h1><p className="muted">Navigation-only prototype · unlimited energy</p>
      <p className="scan-status" role="status" aria-live="polite">{paused?'Navigation paused while hidden':pulse===0?'Scanner: awaiting first five-second pulse':`Scanner pulse ${pulse}: ${contacts.length} temporary contacts`}</p>
      <div className="stat"><span>Total ship mass</span><strong>{pose.massKg.toLocaleString(undefined, { maximumFractionDigits: 3 })} kg</strong></div>
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
      {target?<><strong>{target.name}</strong><p className="muted">{target.kind} · {fmt(distance??0)} light-seconds</p><p className="muted">Surface clearance {fmt(targetDistances?.surfaceDistanceM??0)} m</p><button className="action" disabled={pose.mode!=='flying'} onClick={setDirection}>Set direction</button></>:<p className="muted">Select a visible contact or use the scanned-contact list.</p>}
      <hr/>
      <h2>Camera and orientation</h2>
      <p className="muted" id="navigation-help">WASD pans the view · wheel zooms · arrow keys turn the ship.</p>
      <div className="camera-actions"><button onClick={()=>setHelpOpen(true)}>Help</button><button onClick={resetView}>Reset camera</button></div>
      <hr/>
      <h2><label htmlFor="throttle">Throttle</label> <strong>{throttle>0?'+':''}{throttle}%</strong></h2>
      <input id="throttle" aria-valuetext={`${throttle}% ${throttle>0?'forward':throttle<0?'reverse':'coast'}`} type="range" min="-100" max="100" step="1" value={throttle} disabled={pose.mode!=='flying'} onChange={e=>{const value=Number(e.target.value);throttleRef.current=value;setThrottle(value)}}/>
      <button disabled={pose.mode!=='flying'} onClick={()=>{throttleRef.current=0;setThrottle(0)}}>Zero throttle (Space)</button>
      <p className="muted foot">Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run. Lower-speed contact settles the ship.</p>
    </aside>
    {helpOpen && <HelpDialog onClose={()=>setHelpOpen(false)}/>}
  </div>
}
