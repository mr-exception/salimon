import { TopDownHelp } from './TopDownHelp.tsx'
import { TopDownCockpit } from './TopDownCockpit.tsx'
import { TopDownScene } from './TopDownScene.tsx'
export { TopDownScene } from './TopDownScene.tsx'
import { SurvivalHud, InventoryHud } from './TopDownHud.tsx'
import { useTopDownLoop } from './useTopDownLoop.ts'
import { GearNoticeTracker } from './equipmentHud.ts'
import { EquipmentPanel } from './EquipmentPanel.tsx'
import { commandFeedback } from './commandFeedback.ts'
import { selectObjectCommand } from './topDownCommands.ts'
import { ObjectContextMenu, type ObjectMenuSelection } from './ObjectContextMenu.tsx'
import { objectActions, type ObjectActionId } from './topDownActions.ts'
import { ExplorationHud } from './ExplorationHud.tsx'
import { BOARDING_DOOR_ID } from './scoutDoor.ts'
import { SCOUT_ANCHORS } from './scoutLayout.ts'
import { useExpeditionAudio } from './audio/ExpeditionAudio.tsx'
import { cancelGroundMovement, requestGroundMovement } from './topDownNavigation.ts'
import { relativeAnchor, worldToView } from './topDownFrames.ts'
import { METER, survivalPolicy } from './survival.ts'
import { ObservationCameraState } from './observationCamera.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { isCockpitTarget, leaveCockpit, setCockpitPower } from './cockpit.ts'
import { useCallback, useEffect, useRef, useState } from 'react'
import { cancelInteraction, createTopDownState, PLAYER_ID, requestInteraction, type TopDownState } from './topDownRuntime.ts'
import { useWorldGraphics } from './rendering/WorldGraphicsSettings.tsx'
import { RESOURCE_CATALOG, RESOURCE_SAVE_KEY, ResourceSession, type ResourceStorage } from './resourceSession.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import { ResourceContainerPanel } from './ResourceContainerPanel.tsx'
import { SCOUT_RESOURCE_CONTAINER_ID } from './scoutResourceContainer.ts'
import { Button, Progress } from './ui/primitives.tsx'
import { FeedbackScope, StatusMessage } from './ui/Feedback.tsx'
import { useDismissLayer } from './ui/interaction.ts'
import './topDown.css'

const openResourceSession = (storage: ResourceStorage, world: TopDownState) => new ResourceSession(storage, world)
export function TopDownView({ gameplay = false, createResourceSession = openResourceSession }: { gameplay?: boolean; createResourceSession?: typeof openResourceSession }) {
  const graphics = useWorldGraphics()
  const flightFixture = import.meta.env.DEV && gameplay && new URLSearchParams(location.search).has('flightFixture')
  const powerFixture = import.meta.env.DEV && gameplay && new URLSearchParams(location.search).has('powerFixture')
  const [state, setState] = useState(() => createTopDownState(import.meta.env.DEV && new URLSearchParams(location.search).get('view') === 'interior' ? 'interior' : 'exterior', gameplay))
  const [navigation] = useState(() => new NavigationRuntime(SAMPLE_BODIES, gameplay ? state.scoutLocation : undefined))
  const [observation] = useState(() => new ObservationCameraState(navigation.clock.state))
  const current = useRef(state)
  const session = useRef<ResourceSession | null>(null)
  const lockAttempt = useRef<Promise<unknown>>(Promise.resolve())
  const gearNotices=useRef(new GearNoticeTracker()).current
  const [gearOpen,setGearOpen]=useState(false)
  const container = useRef<ResourceContainerInteraction | null>(null)
  const openEquipment = (world: TopDownState) => {
    if (!session.current || ![SCOUT_RESOURCE_CONTAINER_ID, 'scout:core', 'scout:fabricator', 'scout:gear-bench'].includes(world.selectedId ?? '')) return false
    if(world.selectedId===SCOUT_RESOURCE_CONTAINER_ID&&world.objectCommand?.actionId==='equipment'){
      container.current?.close();session.current.cancel();session.current.world=requestInteraction(world,SCOUT_RESOURCE_CONTAINER_ID);session.current.checkpoint();setGearOpen(true);return true
    }
    const equipment = world.selectedId === 'scout:gear-bench' ? 'bench' : world.selectedId === 'scout:fabricator' ? 'fabricator' : world.selectedId === 'scout:core' ? 'core' : 'container'
    if (container.current?.equipment !== equipment) {
      container.current?.close()
      container.current = new ResourceContainerInteraction(session.current, equipment)
    }
    return container.current!.open(world)
  }
  const menuSequence = useRef(0)
  const [objectMenu, setObjectMenu] = useState<ObjectMenuSelection | null>(null)
  const closeObjectMenu = useCallback(() => setObjectMenu(null), [])
  const paused = useRef(false)
  const [inventoryOpen, setInventoryOpen] = useState(false)
  const inventoryExpanded = useRef(false)
  const inventoryTrigger = useRef<HTMLButtonElement>(null)
  const inventoryPanel = useRef<HTMLDivElement>(null)
  const [helpOpen, setHelpOpen] = useState(false)
  const [notebookOpen, setNotebookOpen] = useState(false)
  const [scoutGuiding, setScoutGuiding] = useState(false)
  const notebookWasOpen = useRef(false)
  useEffect(() => {
    if (notebookWasOpen.current && !notebookOpen) document.querySelector<HTMLButtonElement>('[data-field-help]')?.focus()
    notebookWasOpen.current = notebookOpen
  }, [notebookOpen])
  const toggleInventory = (open: boolean, restoreFocus = false) => {
    inventoryExpanded.current = open; setInventoryOpen(open)
    if (open) session.current?.cancel()
    else if (restoreFocus) inventoryTrigger.current?.focus()
  }
  useDismissLayer(inventoryOpen, () => toggleInventory(false, true))
  useEffect(() => { if (inventoryOpen) inventoryPanel.current?.focus({ preventScroll: true }) }, [inventoryOpen])
  useEffect(() => { if (state.cockpit.view !== 'world') toggleInventory(false) }, [state.cockpit.view])
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
        navigation.surfaceDefinitionForBody = id => session.current!.bodyDefinition(id)
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
      navigation.surfaceDefinitionForBody = undefined
      session.current = null
      unlock?.()
    }
  }, [gameplay])
  useTopDownLoop({ gameplay, current, session, container, navigation, paused, inventoryExpanded, closeObjectMenu, setGearOpen, toggleInventory, setNotebookOpen, setScoutGuiding, setState, openEquipment })
  const selected = state.entities.find(e => e.id === state.selectedId)
  const update = (next: TopDownState) => { current.current = next; setState(next) }
  const cockpitTarget = gameplay && isCockpitTarget(selected?.id ?? null)

  const resourceSession = session.current
  const usage = resourceSession && resourceInventoryUsage(resourceSession.snapshot.inventory, RESOURCE_CATALOG, resourceSession.itemCatalog)
  const selectedContainer = resourceSession && [SCOUT_RESOURCE_CONTAINER_ID, 'scout:core', 'scout:fabricator', 'scout:gear-bench'].includes(selected?.id ?? '')
  const closeContainer = () => { container.current?.close(); current.current = session.current?.world ?? current.current; (document.activeElement as HTMLElement | null)?.blur(); setState({ ...current.current }) }
  const selectedSource = resourceSession && selected && resourceSession.source(selected.id)
  const interact = () => {
    if (current.current.scoutLocation.ship.mode === 'destroyed' || paused.current || resourceSession?.survival.health === 0 || cockpitTarget && resourceSession && (!!resourceSession.error || !survivalPolicy(resourceSession.survival).canWork)) return
    current.current = cancelGroundMovement(current.current)
    if (resourceSession) resourceSession.world = current.current
    if (selectedContainer && openEquipment(current.current)) current.current = container.current!.session.world
    else if (selected?.id === BOARDING_DOOR_ID && resourceSession) { resourceSession.world = current.current; resourceSession.toggleBoardingDoor(resourceSession.snapshot.revision); current.current = resourceSession.world }
    else if (resourceSession?.equipped && selectedSource) { resourceSession.start(selected!.id); current.current = resourceSession.world }
    else { if (resourceSession) resourceSession.feedback = ''; current.current = current.current.player.mode === 'interacting' ? cancelInteraction(current.current) : requestInteraction(current.current) }
    setState(current.current)
  }
  const chooseObjectAction = (action: ObjectActionId) => {
    const id = objectMenu?.targetId
    if (!id) return
    const next = selectObjectCommand({ world: current.current, session: session.current, paused: paused.current }, id, action, objectMenu.sequence)
    if (session.current) { session.current.world = next; session.current.feedback = '' }
    update(next)
  }

  const menuActions = objectMenu ? objectActions({ world: state, session: resourceSession, paused: paused.current }, objectMenu.targetId) : []
  useEffect(() => { if (objectMenu && !menuActions.length) closeObjectMenu() }, [objectMenu, menuActions.length, closeObjectMenu])
  const vital = resourceSession?.survival
  const warning = state.scoutLocation.ship.mode === 'destroyed' ? 'Journey ended — whole ship destroyed. No surface access.' : vital?.health === 0 ? 'Run ended · zero health. Start a fresh character to continue.' : [
    vital && vital.health <= 40 ? 'Low health · critical condition.' : '',
    vital?.fatigue === 0 ? 'Exhausted · gathering and piloting unavailable.' : '',
    vital && vital.hunger < METER * .2 ? 'Low hunger · starvation at zero. Movement and mining reduced.' : '',
    vital && vital.fatigue > 0 && vital.fatigue <= METER * .5 ? 'Low fatigue · movement reduced; mining reduced at 20 or below.' : '',
  ].filter(Boolean).join(' ')
  const ended = state.scoutLocation.ship.mode === 'destroyed' || vital?.health === 0
  const audio = useExpeditionAudio({
    enabled: gameplay,
    active: !!resourceSession && !resourceSession.error && !paused.current && !ended,
    view: ended ? 'ended' : state.cockpit.view,
    warning,
    door: state.scoutLocation.boardingDoor,
    doorPosition: () => {
      const playerAnchor = state.entities.find(entity => entity.id === PLAYER_ID)?.anchor
      if (!playerAnchor || (state.frames.ship.parent === null && playerAnchor.frame !== state.frames.ship.id)) return undefined
      const offset = worldToView(relativeAnchor(SCOUT_ANCHORS.entry, playerAnchor, state.frames), state.frames)
      return { rightM: offset[0], distanceM: Math.hypot(...offset) }
    },
  })
  const gatherBlocked = selectedSource && resourceSession?.equipped && usage && !resourceSession.activeSourceId ? selectedSource.remainingGrams === 0 ? 'Source depleted. No material remains.' : usage.grams >= resourceSession.snapshot.inventory.capacityGrams ? 'Inventory full: weight limit.' : !resourceSession.snapshot.inventory.quantities[selectedSource.resourceId] && usage.slots >= resourceSession.snapshot.inventory.capacitySlots ? 'Inventory full: slot limit.' : '' : ''
  const canAct = state.scoutLocation.ship.mode !== 'destroyed' && !(cockpitTarget && !state.cockpit.powered) && !gatherBlocked && !!selected?.interaction && !paused.current && !resourceSession?.error && vital?.health !== 0 && !(cockpitTarget && vital && !survivalPolicy(vital).canWork) && !(selectedSource && resourceSession?.equipped && vital && !survivalPolicy(vital).canWork)
  const commandStatus = commandFeedback(state, resourceSession?.activeSourceId, resourceSession?.feedback)
  const cancelCommand = () => {
    let next = cancelInteraction(cancelGroundMovement(current.current))
    if (resourceSession?.activeSourceId && next.objectCommand?.actionId === 'gather') next = { ...next, objectCommand: { ...next.objectCommand, status: 'cancelled', message: 'Mining cancelled. Partial work saved.' } }
    if (resourceSession) { resourceSession.world = next; resourceSession.cancel() }
    update(next)
  }
  const persistentMessages = [
    { id: 'survival', text: warning, priority: 'critical' as const },
    { id: 'power', text: state.player.mode === 'seated' && !state.cockpit.powered ? 'Cockpit unpowered · observation unavailable. Stand up or restore power.' : '', priority: 'critical' as const },
  ]
  return <FeedbackScope messages={persistentMessages}><div className="expedition-ui">
    {gameplay && <SurvivalHud state={state} resourceSession={resourceSession} paused={paused.current}
      warning={warning} saveStatus={saveStatus} gearNotices={gearNotices} powerMessage={persistentMessages[1].text}
      onFreshRun={() => { if (resourceSession!.freshRun()) observation.reset(navigation.clock.state); setState({ ...current.current }) }}
      onTogglePause={() => { paused.current = !paused.current; current.current = cancelGroundMovement(current.current); resourceSession!.world = current.current; resourceSession!.cancel(); resourceSession!.checkpoint(); setState({ ...current.current }) }}
      onHelp={() => setHelpOpen(true)} />}
    <TopDownHelp open={helpOpen} gameplay={gameplay} resourceSession={resourceSession} audio={audio} graphics={graphics}
      onDismiss={() => setHelpOpen(false)}
      onNotebook={() => { setHelpOpen(false); session.current?.cancel(); setNotebookOpen(true) }}
      onGuidance={() => { setHelpOpen(false); setScoutGuiding(value => !value) }} />
    <TopDownCockpit state={state} preload={cockpitTarget} graphics={graphics} warning={warning}
      navigation={navigation} observation={observation} current={current} session={session} paused={paused} update={update} />
    {flightFixture && <div className="power-fixture">{(['begin-takeoff', 'finish-takeoff', 'begin-landing', 'finish-landing'] as const).map(kind => <button key={kind} onClick={event => { event.currentTarget.blur(); if (session.current) { session.current.world = current.current; session.current.transitionScout(kind === 'begin-landing' ? { kind, siteId: current.current.scoutLocation.siteId, bodyId: current.current.scoutLocation.body.id } : { kind }, session.current.snapshot.revision, current.current.cockpit); update({ ...session.current.world }) } }}>{kind}</button>)}</div>}
    {powerFixture && <div className="power-fixture" style={flightFixture ? { transform: 'translateY(-50px)' } : undefined}><button onClick={event => { update(setCockpitPower(current.current, !current.current.cockpit.powered)); event.currentTarget.blur() }}>{state.cockpit.powered ? 'Test cockpit power loss' : 'Restore cockpit power'}</button></div>}
    {gearOpen && resourceSession && <EquipmentPanel viewKey={JSON.stringify([resourceSession.snapshot.inventory,state.entities.find(e=>e.id===PLAYER_ID)?.anchor,state.player.mode,state.cockpit.view,state.scoutLocation.ship.mode,resourceSession.survival.health===0,resourceSession.error],(_,value)=>typeof value==='bigint'?String(value):value)} session={resourceSession} onClose={()=>setGearOpen(false)} onChange={()=>setState({...current.current})} />}
    {container.current?.opened && <ResourceContainerPanel interaction={container.current} onClose={closeContainer} onChange={() => setState({ ...current.current })} />}
    {objectMenu && menuActions.length > 0 && <ObjectContextMenu key={objectMenu.targetId} selection={objectMenu}
      label={state.entities.find(e => e.id === objectMenu.targetId)!.interaction!.label}
      actions={menuActions} onClose={closeObjectMenu} onChoose={chooseObjectAction} />}
    <main data-surface-bound={String(state.scoutLocation.binding !== null)} data-tick={state.tick} hidden={state.cockpit.view === 'radar'} data-cockpit-view={state.cockpit.view} data-powered={String(state.cockpit.powered)} data-occupant={state.cockpit.occupantId ?? ''} data-scout-location={gameplay ? JSON.stringify(state.scoutLocation, (_, v) => typeof v === 'bigint' ? v.toString() : v) : undefined} data-ship={gameplay ? JSON.stringify(navigation.clock.state, (_, v) => typeof v === 'bigint' ? v.toString() : v) : undefined} className="top-down-app" data-player-mode={state.player.mode} data-target={state.player.targetId ?? ''}>
    <header className="top-down-sr-only"><h1>Scout · Surface study</h1></header>
    <section className="top-down-stage"><TopDownScene state={state} quality={graphics.quality} paused={paused.current}
        rawAccess={!!container.current?.opened && container.current.storeId === SCOUT_RESOURCE_CONTAINER_ID}
        equipmentAccess={gearOpen && state.player.targetId === SCOUT_RESOURCE_CONTAINER_ID} activeCues={audio.activeCueIds} session={resourceSession} onCamera={operation => {
        if (paused.current || current.current.cockpit.view !== 'world' || container.current?.opened || document.querySelector('dialog[open]')) return
        const next = operation(current.current)
        if (resourceSession) resourceSession.world = next
        update(next)
      }} onCancelGround={() => update(cancelGroundMovement(current.current))} onGround={(anchor, objectId, pointer) => {
        if (current.current.cockpit.view !== 'world' || container.current?.opened || document.querySelector('dialog[open]')) return
        if (!objectId && (paused.current || gameplay && (!resourceSession || resourceSession.error || resourceSession.survival.health === 0))) return
        if (objectId) setObjectMenu({ targetId: objectId, ...pointer, sequence: ++menuSequence.current })
        else { closeObjectMenu(); resourceSession?.cancel() }
        if (resourceSession && !objectId) resourceSession.feedback = ''
        const next = objectId ? cancelGroundMovement(current.current) : requestGroundMovement(current.current, anchor)
        if (resourceSession) resourceSession.world = next
        update(next)
      }} />
      {gameplay && resourceSession && <ExplorationHud state={state} session={resourceSession} blocked={paused.current || !!resourceSession.error || resourceSession.survival.health === 0 || state.scoutLocation.ship.mode === 'destroyed'} onChange={() => setState({...current.current})} open={notebookOpen} guiding={scoutGuiding} onDismiss={() => setNotebookOpen(false)} onGuidanceChange={() => setScoutGuiding(value => !value)} />}
      {gameplay && <InventoryHud resourceSession={resourceSession} paused={paused.current} saveStatus={saveStatus}
        inventoryOpen={inventoryOpen} inventoryTrigger={inventoryTrigger} inventoryPanel={inventoryPanel} toggleInventory={toggleInventory}
        onEquipment={() => { resourceSession!.cancel(); container.current?.close(); setGearOpen(true) }}
        onChange={() => setState({ ...current.current })} />}
    </section>
    <div className="top-down-bottom">
    <aside className="top-down-interaction" aria-label="Nearby interaction">
      <div><p data-prompt hidden={!canAct}>{selected?.interaction ? `${selected.interaction.label} · ${cockpitTarget ? 'Navigate' : selectedContainer ? 'Use' : resourceSession?.activeSourceId ? 'Stop gathering' : selectedSource && resourceSession?.equipped ? 'Gather' : selected.interaction.action} [E]` : ''}</p><StatusMessage announce={!resourceSession?.error} priority={resourceSession?.error ? 'error' : resourceSession?.activeSourceId ? 'pending' : 'status'}>{resourceSession?.error || resourceSession?.feedback || state.feedback}</StatusMessage></div>
      {canAct && <button onClick={event => { interact(); event.currentTarget.blur() }}>{selected?.id === BOARDING_DOOR_ID ? selected.interaction?.action : selectedContainer ? selected?.id === 'scout:gear-bench' ? 'Use Gear Crafting Bench' : selected?.id === 'scout:fabricator' ? 'Use Fabricator' : selected?.id === 'scout:core' ? 'Use Energy Core' : 'Use container' : resourceSession?.activeSourceId ? 'Stop gathering' : selectedSource && resourceSession?.equipped ? 'Gather' : state.player.mode === 'interacting' ? 'Cancel inspection' : cockpitTarget ? 'Use cockpit' : 'Inspect fixture'}</button>}
    </aside>
    {gatherBlocked && <p className="source-summary">{gatherBlocked}</p>}
    {selectedSource && <p className="source-summary" data-source-summary>{RESOURCE_CATALOG[selectedSource.resourceId as keyof typeof RESOURCE_CATALOG].name} · {selectedSource.remainingGrams / 1000} kg remaining</p>}
    {!!resourceSession?.activeSourceId && <div className="gathering-status">
      <Progress label="Next kilogram extraction interval" value={resourceSession.progressTicks} max={60} unit="ticks" />
      <p>Mining rate: {survivalPolicy(resourceSession.survival).miningUnits / 16} kg/s · partial work saved per source</p>
      <p>Capacity: {usage!.grams / 1000} / {resourceSession.snapshot.inventory.capacityGrams / 1000} kg · {usage!.slots} / {resourceSession.snapshot.inventory.capacitySlots} slots</p>
    </div>}
    {state.player.mode === 'seated' && <button onClick={() => update(leaveCockpit(current.current))}>Stand up</button>}
    {commandStatus && <div className="command-feedback" data-command-status={state.objectCommand?.status} data-command-target={state.objectCommand?.targetId} data-movement-status={state.groundCommand?.status}>
      <StatusMessage priority={commandStatus.failed ? 'error' : commandStatus.cancellable ? 'pending' : 'status'}>{commandStatus.text}</StatusMessage>
      {commandStatus.cancellable && <Button variant="secondary" onClick={event => { cancelCommand(); event.currentTarget.blur() }}>Cancel command</Button>}
    </div>}
    <footer><p>Right-click ground to walk · <strong>W A S D</strong> Walk · <strong>E</strong> Interact · <strong>Esc</strong> Cancel · <strong>P</strong> Pause</p></footer>
    </div>
  </main></div></FeedbackScope>
}
