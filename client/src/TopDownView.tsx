import { METER, survivalPolicy } from './survival.ts'
import { DRY_MASS_KG, setMass } from './simulation.ts'
import { App } from './App'
import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { isCockpitTarget, leaveCockpit, setCockpitPower } from './cockpit.ts'
import { useEffect, useRef, useState } from 'react'
import { acceptsFlightShortcut, FrameTimer, NavigationInput, panAxisForCode } from './input.ts'
import { cancelInteraction, createTopDownState, drawList, LAYERS, PLAYER_ID, requestInteraction, stepTopDown, TopDownClock, type TopDownState } from './topDownRuntime.ts'
import { useWorldSprites, WorldSprite } from './topDownSprites.tsx'
import { RESOURCE_CATALOG, RESOURCE_SAVE_KEY, ResourceSession, type ResourceStorage } from './resourceSession.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import { ResourceContainerPanel } from './ResourceContainerPanel.tsx'
import { SCOUT_RESOURCE_CONTAINER_ID } from './scoutResourceContainer.ts'
import './topDown.css'

// Presentation receives immutable scene state. It has no radar or Three imports.
export function TopDownScene({ state, session }: { state: TopDownState; session?: ResourceSession | null }) {
  const items = drawList(state)
  const sprites = useWorldSprites(state.entities)
  return <svg className="top-down-scene" viewBox="-600 -320 1200 640" shapeRendering="geometricPrecision" textRendering="geometricPrecision" role="img" aria-label="Strict overhead Scout and planetary test site" data-render-ready={String(sprites.ready)} aria-busy={!sprites.ready}>
    {LAYERS.map(layer => <g key={layer} data-layer={layer}>
      {items.filter(item => item.entity.layer === layer).map(({ entity, positionM: [x, y], angle }) =>
        <g key={entity.id} data-entity={entity.id} data-source-kind={entity.source?.kind} data-source-site={entity.source?.siteId} data-remaining-grams={session?.snapshot.sources[entity.id]?.remainingGrams} data-depleted={session?.snapshot.sources[entity.id]?.remainingGrams === 0 || undefined} data-frame={entity.anchor.frame} data-position={entity.anchor.point.offsetM.slice(0, 2).join(',')} data-posture={entity.id === PLAYER_ID ? state.player.mode : undefined} data-facing={entity.id === PLAYER_ID ? state.player.facing : undefined} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`}>
          <WorldSprite entity={entity} />
        </g>)}
      {layer === 'cues' && items.filter(item => item.entity.id === state.selectedId).map(({ entity, positionM: [x, y], angle }) => <g key={entity.id} data-selection={entity.id} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`} fill="none" stroke="#bce9df" strokeWidth=".08"><WorldSprite entity={entity} selected /></g>)}
    </g>)}
  </svg>
}

const openResourceSession = (storage: ResourceStorage, world: TopDownState) => new ResourceSession(storage, world)
export function TopDownView({ gameplay = false, createResourceSession = openResourceSession }: { gameplay?: boolean; createResourceSession?: typeof openResourceSession }) {
  const powerFixture = import.meta.env.DEV && gameplay && new URLSearchParams(location.search).has('powerFixture')
  const [state, setState] = useState(() => createTopDownState(import.meta.env.DEV && new URLSearchParams(location.search).get('view') === 'interior' ? 'interior' : 'exterior', gameplay))
  const [navigation] = useState(() => new NavigationRuntime(SAMPLE_BODIES, gameplay ? state.scoutLocation : undefined))
  const current = useRef(state)
  const session = useRef<ResourceSession | null>(null)
  const lockAttempt = useRef<Promise<unknown>>(Promise.resolve())
  const container = useRef<ResourceContainerInteraction | null>(null)
  const paused = useRef(false)
  const [saveStatus, setSaveStatus] = useState('Opening saved expedition…')
  useEffect(() => {
    if (!gameplay) return
    let disposed = false, unlock: (() => void) | undefined
    if (!navigator.locks) { setSaveStatus('Saving needs HTTPS or localhost. Gathering is paused.'); return }
    lockAttempt.current = lockAttempt.current.then(() => navigator.locks.request(RESOURCE_SAVE_KEY, { mode: 'exclusive', ifAvailable: true }, async lock => {
      if (disposed) return
      if (!lock) { setSaveStatus('Another tab has this expedition open. Close it and reload to gather.'); return }
      try {
        session.current = createResourceSession(localStorage, current.current)
        container.current = new ResourceContainerInteraction(session.current)
        current.current = session.current.world
        setState(current.current)
        setSaveStatus('Expedition saves automatically on this browser.')
        await new Promise<void>(resolve => { unlock = resolve })
      } catch { setSaveStatus('Browser storage is unavailable. Gathering is paused.') }
    })).catch(() => { if (!disposed) setSaveStatus('Could not open saved expedition. Gathering is paused.') })
    return () => {
      disposed = true
      session.current?.cancel()
      session.current?.checkpoint()
      container.current?.close()
      container.current = null
      session.current = null
      unlock?.()
    }
  }, [gameplay])
  useEffect(() => {
    const input = new NavigationInput(), clock = new TopDownClock(), timer = new FrameTimer(performance.now())
    let raf = 0
    const release = () => { input.clear(); clock.reset(); timer.resume(performance.now()) }
    const clear = () => { release(); session.current?.cancel(); container.current?.close(); current.current = cancelInteraction(current.current); if (session.current) session.current.world = current.current; session.current?.checkpoint(); setState(current.current) }
    const keydown = (event: KeyboardEvent) => {
      if (event.code === 'KeyP' && !event.repeat && acceptsFlightShortcut(event.target)) {
        paused.current = !paused.current; clear(); return
      }
      if (paused.current || session.current?.survival.health === 0) return
      if (event.code === 'KeyE' && isCockpitTarget(current.current.selectedId) && session.current && (!!session.current.error || !survivalPolicy(session.current.survival).canWork)) return
      if (current.current.cockpit.view === 'radar') return
      if (container.current?.opened && event.code === 'KeyE') return
      if (!event.repeat && (event.code === 'Escape' || (event.code === 'KeyE' && acceptsFlightShortcut(event.target)))) {
        event.preventDefault(); input.clear()
        if (container.current?.opened && event.code === 'Escape') {
          container.current.close(); current.current = container.current.session.world
        } else if (event.code === 'KeyE' && current.current.selectedId === SCOUT_RESOURCE_CONTAINER_ID && container.current?.open(current.current)) {
          current.current = container.current.session.world
        } else if (event.code === 'KeyE' && session.current?.equipped && current.current.entities.find(e => e.id === current.current.selectedId)?.source) {
          session.current.start(current.current.selectedId)
          current.current = session.current.world
        } else {
          const wasGathering = !!session.current?.activeSourceId
          session.current?.cancel()
          current.current = event.code === 'KeyE' ? requestInteraction(current.current) : current.current.player.mode === 'seated' ? leaveCockpit(current.current) : cancelInteraction(current.current)
          if (session.current && !wasGathering) session.current.feedback = ''
        }
        setState(current.current)
      }
      const axis = panAxisForCode(event.code)
      if (axis && acceptsFlightShortcut(event.target)) { event.preventDefault(); input.press(event.code, axis) }
    }
    const keyup = (event: KeyboardEvent) => input.release(event.code)
    const frame = (now: number) => {
      const dt = timer.sample(now, document.hidden || paused.current || gameplay && (!session.current || !!session.current.error || session.current.survival.health === 0))
      const inWorld = current.current.cockpit.view === 'world'
      if (!inWorld) input.clear()
      if (paused.current || document.hidden) { input.clear(); clock.reset() }
      const controls = input.controls
      current.current = clock.advance(current.current, dt, [Number(controls.right) - Number(controls.left), Number(controls.down) - Number(controls.up)], (world, direction) => {
        if (!session.current) return stepTopDown(world, direction)
        if (session.current.error || session.current.survival.health === 0) return world
        return session.current.advance(stepTopDown(world, direction, survivalPolicy(session.current.survival).movement))
      })
      if (session.current) {
        session.current.world = current.current
        container.current?.synchronize(current.current)
        current.current = session.current.world
        navigation.clock.state = setMass(navigation.clock.state, DRY_MASS_KG + session.current.payload().shipGrams / 1000)
      }
      if (gameplay && inWorld) navigation.advance(dt)
      setState(current.current)
      raf = requestAnimationFrame(frame)
    }
    window.addEventListener('keydown', keydown); window.addEventListener('keyup', keyup)
    window.addEventListener('blur', clear); window.addEventListener('resize', clear)
    document.addEventListener('visibilitychange', clear); window.addEventListener('focusin', release)
    window.addEventListener('pagehide', clear)
    raf = requestAnimationFrame(frame)
    return () => {
      cancelAnimationFrame(raf)
      window.removeEventListener('keydown', keydown); window.removeEventListener('keyup', keyup)
      window.removeEventListener('blur', clear); window.removeEventListener('resize', clear)
      document.removeEventListener('visibilitychange', clear); window.removeEventListener('focusin', release)
      window.removeEventListener('pagehide', clear)
    }
  }, [])
  const selected = state.entities.find(e => e.id === state.selectedId)
  const update = (next: TopDownState) => { current.current = next; setState(next) }
  const cockpitTarget = gameplay && isCockpitTarget(selected?.id ?? null)
  const resourceSession = session.current
  const usage = resourceSession && resourceInventoryUsage(resourceSession.snapshot.inventory, RESOURCE_CATALOG, resourceSession.itemCatalog)
  const selectedContainer = resourceSession && selected?.id === SCOUT_RESOURCE_CONTAINER_ID
  const closeContainer = () => { container.current?.close(); current.current = session.current?.world ?? current.current; (document.activeElement as HTMLElement | null)?.blur(); setState({ ...current.current }) }
  const selectedSource = resourceSession && selected && resourceSession.snapshot.sources[selected.id]
  const interact = () => {
    if (paused.current || resourceSession?.survival.health === 0 || cockpitTarget && resourceSession && (!!resourceSession.error || !survivalPolicy(resourceSession.survival).canWork)) return
    if (selectedContainer && container.current?.open(current.current)) current.current = container.current.session.world
    else if (resourceSession?.equipped && selectedSource) { resourceSession.start(selected!.id); current.current = resourceSession.world }
    else { if (resourceSession) resourceSession.feedback = ''; current.current = current.current.player.mode === 'interacting' ? cancelInteraction(current.current) : requestInteraction(current.current) }
    setState(current.current)
  }
  const vital = resourceSession?.survival
  const warning = vital?.health === 0 ? 'Run ended · zero health. Start a fresh character to continue.' : vital?.fatigue === 0 ? 'Exhausted · gathering and piloting unavailable.' : vital && vital.hunger < METER * .2 ? 'Low hunger · starvation at zero. Movement and mining reduced.' : vital && vital.fatigue <= METER * .5 ? 'Low fatigue · movement reduced; mining reduced at 20 or below.' : ''
  return <>
    {gameplay && <aside className="survival-hud" aria-label="Survival condition">
      {vital && !resourceSession.error ? <>
        <label>Health <span>{Math.ceil(vital.health)} / 200 HP</span><meter aria-label="Health" min={0} max={200} value={vital.health} /></label>
        <label>Hunger <span>{Math.ceil(vital.hunger / METER * 100)} / 100</span><meter aria-label="Hunger" min={0} max={100} value={Math.ceil(vital.hunger / METER * 100)} /></label>
        <label>Fatigue <span>{Math.ceil(vital.fatigue / METER * 100)} / 100</span><meter aria-label="Fatigue" min={0} max={100} value={Math.ceil(vital.fatigue / METER * 100)} /></label>
        <p aria-live="polite">{paused.current ? 'Paused · active time stopped' : warning}</p>
        {vital.health === 0 ? <button onClick={event => { event.currentTarget.blur(); resourceSession.freshRun(); setState({ ...current.current }) }}>Start fresh character · keep world resources</button> : <button onClick={event => { event.currentTarget.blur(); paused.current = !paused.current; resourceSession.cancel(); resourceSession.checkpoint(); setState({ ...current.current }) }}>{paused.current ? 'Resume' : 'Pause'} [P]</button>}
      </> : <p>{resourceSession?.error || saveStatus}</p>}
    </aside>}
    {state.cockpit.view === 'radar' && <><div inert={paused.current || !!resourceSession?.error}><App navigation={navigation} isPaused={() => paused.current || !!session.current?.error} canPilot={() => !session.current || survivalPolicy(session.current.survival).canWork} /></div><button className="cockpit-exit" onClick={() => update(leaveCockpit(current.current))}>Exit navigation · Stand up</button></>}
    {powerFixture && <div className="power-fixture"><button onClick={event => { update(setCockpitPower(current.current, !current.current.cockpit.powered)); event.currentTarget.blur() }}>{state.cockpit.powered ? 'Test cockpit power loss' : 'Restore cockpit power'}</button></div>}
    {container.current?.opened && <ResourceContainerPanel interaction={container.current} onClose={closeContainer} onChange={() => setState({ ...current.current })} />}
    <main data-tick={state.tick} hidden={state.cockpit.view === 'radar'} data-cockpit-view={state.cockpit.view} data-powered={String(state.cockpit.powered)} data-occupant={state.cockpit.occupantId ?? ''} data-scout-location={gameplay ? JSON.stringify(state.scoutLocation, (_, v) => typeof v === 'bigint' ? v.toString() : v) : undefined} data-ship={gameplay ? JSON.stringify(navigation.clock.state, (_, v) => typeof v === 'bigint' ? v.toString() : v) : undefined} className="top-down-app" data-player-mode={state.player.mode} data-target={state.player.targetId ?? ''}>
    <header><div><p className="top-down-eyebrow">SALIMON / WORLD FOUNDATION</p><h1>Scout · Surface study</h1></div><span className="top-down-tag">STRICT OVERHEAD / 2D</span></header>
    <section className="top-down-stage"><TopDownScene state={state} session={resourceSession} />
      {gameplay && <aside className="resource-hud" aria-label="Resource inventory" data-gathering={resourceSession?.activeSourceId ?? ''}>
        <h2>Field inventory</h2>
        {resourceSession && usage && <>
          <p data-inventory-weight><strong>{usage.grams / 1000}</strong> / {resourceSession.snapshot.inventory.capacityGrams / 1000} kg</p>
          <p data-inventory-slots><strong>{usage.slots}</strong> / {resourceSession.snapshot.inventory.capacitySlots} slots</p>
          <ul aria-label="Raw resources">{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <li key={id}>{entry.name} <span data-resource={id}>{(resourceSession.snapshot.inventory.quantities[id] ?? 0) / 1000} kg</span></li>)}</ul>
          <p className="resource-note">Each carried raw type uses 1 slot.</p>
          <ul aria-label="Count items">{Object.entries(resourceSession.snapshot.inventory.items).map(([id, count]) => {
            const meta = resourceSession.itemCatalog[id]!
            return <li className="count-item" key={id} data-item={id}><span>{meta.name ?? id} × {count}</span><span>{count * meta.gramsPerItem / 1000} kg · {Math.ceil(count / meta.stackSize)} slots</span></li>
          })}</ul>
          {!Object.keys(resourceSession.snapshot.inventory.items).length && <p data-empty-items>No items carried.</p>}
        </>}
        <button disabled={!resourceSession || !!resourceSession.error || resourceSession.survival.health === 0 || paused.current} aria-pressed={resourceSession?.equipped ?? false} onClick={event => { resourceSession?.toggleTool(); event.currentTarget.blur(); setState({ ...current.current }) }}>{resourceSession?.equipped ? 'Stow gathering tool' : 'Equip gathering tool'}</button>
        <p className="resource-note">{resourceSession?.error || saveStatus}</p>
        {selectedSource && <p data-source-summary>{RESOURCE_CATALOG[selectedSource.resourceId as keyof typeof RESOURCE_CATALOG].name} · {selectedSource.remainingGrams / 1000} kg remaining</p>}
        {!!resourceSession?.activeSourceId && <p>Mining rate: {survivalPolicy(resourceSession.survival).miningUnits / 16} kg/s · partial work saved per source</p>}
        {!!resourceSession?.activeSourceId && <progress aria-label="Next kilogram extraction interval" max={60} value={resourceSession.progressTicks} />}
      </aside>}
      <div className="top-down-caption"><span>01 / QUIET LANDING SITE</span><span>SCOUT — INTERIOR CUTAWAY</span></div></section>
    <aside className="top-down-interaction" aria-label="Nearby interaction">
      <div><p data-prompt>{selected?.interaction ? `${selected.interaction.label} · ${cockpitTarget ? 'Navigate' : selectedContainer ? 'Use' : selectedSource && resourceSession?.equipped ? 'Gather' : selected.interaction.action} [E]` : 'No fixture in reach · Move closer'}</p><p role="status" aria-live="polite">{resourceSession?.error || resourceSession?.feedback || state.feedback}</p></div>
      <button onClick={event => { interact(); event.currentTarget.blur() }} disabled={paused.current || cockpitTarget && !!resourceSession?.error || resourceSession?.survival.health === 0 || !selected && state.player.mode !== 'interacting'}>{selectedContainer ? 'Use container' : resourceSession?.activeSourceId ? 'Stop gathering' : selectedSource && resourceSession?.equipped ? 'Gather' : state.player.mode === 'interacting' ? 'Cancel inspection' : cockpitTarget ? 'Use cockpit' : 'Inspect fixture'}</button>
    </aside>
    {state.player.mode === 'seated' && <button onClick={() => update(leaveCockpit(current.current))}>Stand up</button>}
    <footer><p><strong>W A S D</strong> Walk · <strong>E</strong> Interact · <strong>Esc</strong> Cancel</p><p>{gameplay ? 'Gathering · up to 1 kg/s · Low hunger/fatigue reduce rate · Partial work saved' : 'Console inspections are a test · Systems inactive'}</p></footer>
  </main></>
}
