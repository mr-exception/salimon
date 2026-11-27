import { EquipmentHud } from './EquipmentHud.tsx'
import { GearNoticeTracker, equipmentHudReadout } from './equipmentHud.ts'
import { EquipmentIcon } from './rendering/EquipmentIcon.tsx'
import { EquipmentPanel } from './EquipmentPanel.tsx'
import { commandFeedback } from './commandFeedback.ts'
import { advanceObjectCommand, selectObjectCommand } from './topDownCommands.ts'
import { ObjectContextMenu, type ObjectMenuSelection } from './ObjectContextMenu.tsx'
import { objectActions, type ObjectActionId } from './topDownActions.ts'
import { CameraDrag, followWorldCamera, panWorldCamera, zoomWorldCamera } from './topDownCamera.ts'
import { MarkerDeployment } from './MarkerDeployment.tsx'
import { ExplorationHud, FieldMarkCues } from './ExplorationHud.tsx'
import { GeneratedGround } from './GeneratedGround.tsx'
import { BOARDING_DOOR_ID } from './scoutDoor.ts'
import { SCOUT_ANCHORS } from './scoutLayout.ts'
import { ExpeditionAudioSettings, useExpeditionAudio } from './audio/ExpeditionAudio.tsx'
import { cancelGroundMovement, groundFromView, interactiveAt, requestGroundMovement } from './topDownNavigation.ts'
import { relativeAnchor, rotate2, worldToView, type Anchor, viewToWorld } from './topDownFrames.ts'
import { METER, survivalPolicy } from './survival.ts'
import { DRY_MASS_KG, setMass } from './simulation.ts'
import type { PanoramicNavigation } from './PanoramicNavigation.tsx'
import { ObservationCameraState } from './observationCamera.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { controlsCockpit, LOCAL_PLAYER, isCockpitTarget, leaveCockpit, setCockpitPower } from './cockpit.ts'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { acceptsFlightShortcut, FrameTimer, NavigationInput, panAxisForCode } from './input.ts'
import { cancelInteraction, createTopDownState, drawList, PLAYER_ID, type DrawItem, requestInteraction, stepTopDown, TopDownClock, type TopDownState } from './topDownRuntime.ts'
import { scoutWallRequests, spriteBounds, spriteRequest, usePresentationVisibility, useWorldSprites, WorldSprite } from './topDownSprites.tsx'
import { QUALITY, renderDpr, type WorldQuality } from './rendering/graphicsPreferences.ts'
import { useWorldGraphics, WorldGraphicsSettings } from './rendering/WorldGraphicsSettings.tsx'
import { diagnosticsEnabled, FrameDiagnostics, RenderDiagnostics } from './rendering/RenderDiagnostics.tsx'
import { isScoutModule, scoutArtObstructs, scoutWallJoins } from './rendering/scoutArt.ts'
import { characterArt, ScoutCharacter } from './rendering/ScoutCharacter.tsx'
import { scoutFixture, ScoutFixtureSignal } from './rendering/ScoutFixtures.tsx'
import { machineState, scoutMachine, ScoutMachineSignal } from './rendering/ScoutMachine.tsx'
import { ScoutModule, ScoutWall } from './rendering/ScoutModules.tsx'
import { svgArtIntersects, svgLayers, svgSpriteTransform } from './rendering/svgWorld.ts'
import { RESOURCE_CATALOG, RESOURCE_SAVE_KEY, ResourceSession, type ResourceStorage } from './resourceSession.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import { ResourceContainerPanel } from './ResourceContainerPanel.tsx'
import { SCOUT_RESOURCE_CONTAINER_ID } from './scoutResourceContainer.ts'
import { Button, Dialog, KeyHint, Meter, Progress, UtilityPanel } from './ui/primitives.tsx'
import { FeedbackScope, StatusMessage } from './ui/Feedback.tsx'
import { useDismissLayer } from './ui/interaction.ts'
import './topDown.css'

// Presentation receives immutable scene state. It has no radar or Three imports.
export function TopDownScene({ state, quality = QUALITY.Medium, paused = false, rawAccess = false, equipmentAccess = false, activeCues = () => [], session, onGround, onCancelGround, onCamera }: { state: TopDownState; quality?: WorldQuality; paused?: boolean; rawAccess?: boolean; equipmentAccess?: boolean; activeCues?: () => readonly string[]; session?: ResourceSession | null; onGround?: (anchor: Anchor, objectId: string | null, pointer: { x: number; y: number }) => void; onCancelGround?: () => void; onCamera?: (operation: (world: TopDownState) => TopDownState) => void }) {
  const viewport = useRef<SVGSVGElement>(null)
  const cameraInput = useRef({ onCamera, onCancelGround })
  cameraInput.current = { onCamera, onCancelGround }
  useEffect(() => {
    if (state.cockpit.view !== 'world') return
    const svg = viewport.current!, drag = new CameraDrag()
    const point = (event: PointerEvent) => {
      const matrix = svg.getScreenCTM()
      if (!matrix) return null
      const p = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse())
      return [p.x, p.y] as const
    }
    const cancel = () => {
      const id = drag.cancel()
      if (id !== null && svg.hasPointerCapture(id)) svg.releasePointerCapture(id)
    }
    const down = (event: PointerEvent) => {
      if (event.button === 0 || event.button === 1) svg.focus({ preventScroll: true })
      if (event.button !== 1 || !cameraInput.current.onCamera) return
      const p = point(event)
      if (!p || !drag.begin(event.pointerId, p)) return
      event.preventDefault()
      cameraInput.current.onCancelGround?.()
      svg.setPointerCapture(event.pointerId)
    }
    const move = (event: PointerEvent) => {
      if (drag.id !== event.pointerId) return
      if (!(event.buttons & 4)) { cancel(); return }
      const p = point(event), delta = p && drag.move(event.pointerId, p)
      if (delta) cameraInput.current.onCamera?.(world => panWorldCamera(world, delta))
    }
    const end = (event: PointerEvent) => { if (drag.id === event.pointerId) cancel() }
    const wheel = (event: WheelEvent) => {
      if (!cameraInput.current.onCamera || event.ctrlKey) return
      event.preventDefault()
      const pixels = event.deltaY * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? svg.clientHeight : 1)
      cameraInput.current.onCamera(world => zoomWorldCamera(world, pixels))
    }
    svg.addEventListener('pointerdown', down)
    svg.addEventListener('pointermove', move)
    svg.addEventListener('pointerup', end)
    svg.addEventListener('pointercancel', end)
    svg.addEventListener('lostpointercapture', end)
    svg.addEventListener('focusout', cancel)
    svg.addEventListener('wheel', wheel, { passive: false })
    window.addEventListener('blur', cancel)
    window.addEventListener('resize', cancel)
    document.addEventListener('visibilitychange', cancel)
    return () => {
      cancel()
      svg.removeEventListener('pointerdown', down)
      svg.removeEventListener('pointermove', move)
      svg.removeEventListener('pointerup', end)
      svg.removeEventListener('pointercancel', end)
      svg.removeEventListener('lostpointercapture', end)
      svg.removeEventListener('focusout', cancel)
      svg.removeEventListener('wheel', wheel)
      window.removeEventListener('blur', cancel)
      window.removeEventListener('resize', cancel)
      document.removeEventListener('visibilitychange', cancel)
    }
  }, [state.cockpit.view])
  const [size, setSize] = useState({ width: 1200, height: 640 })
  useEffect(() => {
    const observer = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect
      if (width > 0 && height > 0) setSize({ width, height })
    })
    observer.observe(viewport.current!)
    return () => observer.disconnect()
  }, [])
  // Expand the visible world to the viewport aspect; never stretch or crop the
  // reference scene. SVG's screen CTM remains the world-to-screen authority.
  const width = Math.max(1200, 640 * size.width / size.height)
  const height = Math.max(640, 1200 * size.height / size.width)
  const { entities, frames, camera: { anchor, pixelsPerM }, player: { mode, facing }, selectedId } = state
  const items = useMemo(() => drawList({ entities, frames, camera: { anchor }, player: { mode, facing } }),
    [entities, frames, anchor, mode, facing])
  // The boarding assembly lies over the deck within the interior pass.
  const boardingDoor = items.find(item => item.entity.id === BOARDING_DOOR_ID)
  const passes = useMemo(() => svgLayers(items.filter(item => item.entity.id !== BOARDING_DOOR_ID)), [items])
  const wallJoins = useMemo(() => scoutWallJoins(entities), [entities])
  const playerArt = items.find(item => item.entity.id === PLAYER_ID && item.entity.anchor.frame === 'ship:scout')
  const selection = items.find(item => item.entity.id === selectedId)
  const visible = usePresentationVisibility()
  const showWorld = visible && state.cockpit.view === 'world'
  const rasterScale = Math.max(.5, renderDpr(window.devicePixelRatio, quality) / window.devicePixelRatio)
  const pilot = characterArt(session?.snapshot.inventory.equipment, !!session?.equipped, mode, quality.shadows === 'dynamic')
  const stride = mode === 'walking' && !window.matchMedia('(prefers-reduced-motion: reduce)').matches
    ? .13 * Math.sin(state.tick * Math.PI / 12) : 0
  const request = (entity: DrawItem['entity'], selected = false) => ({
    ...spriteRequest(entity, selected, rasterScale),
    character: entity.id === PLAYER_ID && !selected ? pilot : undefined,
  })
  const intersects = (item: DrawItem, margin = 0, selected = false) => svgArtIntersects(item,
    spriteBounds({ entity: item.entity, selected }), width / pixelsPerM, height / pixelsPerM, margin)
  const requests = showWorld ? items.filter(item => intersects(item, 4)).flatMap<ReturnType<typeof spriteRequest>>(({ entity }) => {
    const wall = wallJoins.get(entity.id)
    if (wall) return scoutWallRequests(entity, wall)
    return isScoutModule(entity) ? [] : [request(entity)]
  }) : []
  if (showWorld && selection && intersects(selection, 4, true)) requests.push(request(selection.entity, true))
  const artwork = useWorldSprites(requests)
  const renderItems = (list: readonly DrawItem[]) => list.map(item => {
    const { entity } = item
    const sprite = request(entity)
    const wall = wallJoins.get(entity.id)
    const modular = isScoutModule(entity)
    const machine = scoutMachine(entity)
    const fixture = scoutFixture(entity)
    const fade = playerArt && scoutArtObstructs(entity, rotate2([
      playerArt.positionM[0] - item.positionM[0], playerArt.positionM[1] - item.positionM[1],
    ], -item.angle))
    const illustration = entity.id === PLAYER_ID
      ? <g data-character="" data-suit-condition={pilot.condition} data-suit-equipped={String(pilot.suited)}>
          {pilot.tool && <g data-held-tool="" />}
          {pilot.modules.map(item => <g key={item.slot} data-character-module={item.definitionId}
            data-module-condition={item.condition} />)}
          {stride !== 0 ? <ScoutCharacter art={pilot} stride={stride} />
            : <WorldSprite request={sprite} outcome={artwork.get(sprite)} pending={<ScoutCharacter art={pilot} />} />}
        </g>
      : <>
          <WorldSprite request={sprite} outcome={artwork.get(sprite)} />
          {fixture && <ScoutFixtureSignal entity={entity} fixture={fixture} rawAccess={rawAccess}
            equipmentAccess={equipmentAccess} occupied={state.cockpit.occupantId !== null} powered={state.cockpit.powered}
            gearStocked={session?.snapshot.inventory.equipment?.instances.some(item =>
              item.location.kind === 'drawer' || item.location.kind === 'module-storage') ?? false}
            stocked={Object.values(session?.snapshot.stores[SCOUT_RESOURCE_CONTAINER_ID]?.quantities ?? {}).some(grams => grams > 0)} />}
          {machine && <ScoutMachineSignal entity={entity} machine={machine} state={machineState(machine, session, paused)} />}
        </>
    const wallSprites = wall && scoutWallRequests(entity, wall).map(wallSprite =>
      <WorldSprite key={wallSprite.wall.part} request={wallSprite} outcome={artwork.get(wallSprite)} />)
    return <g key={entity.id} data-entity={entity.id} data-door-state={entity.id === BOARDING_DOOR_ID ? state.scoutLocation.boardingDoor : undefined} data-source-kind={entity.source?.kind} data-source-site={entity.source?.siteId} data-remaining-grams={session?.source(entity.id)?.remainingGrams} data-depleted={session?.source(entity.id)?.remainingGrams === 0 || undefined} data-frame={entity.anchor.frame} data-surface-chart={entity.anchor.chart ? JSON.stringify(entity.anchor.chart) : undefined} data-position={entity.anchor.point.offsetM.slice(0, 2).join(',')} data-posture={entity.id === PLAYER_ID ? mode : undefined} data-facing={entity.id === PLAYER_ID ? facing : undefined} transform={svgSpriteTransform(item, pixelsPerM)}>
      {showWorld && intersects(item) && (wall
        ? <ScoutWall entity={entity} joins={wall} fade={!!fade} base={wallSprites?.[0]} cap={wallSprites?.[1]} />
        : modular ? <ScoutModule entity={entity} />
        : entity.footprint && entity.anchor.frame === 'ship:scout'
          ? <g className="scout-obstruction" data-scout-faded={String(!!fade)}>{illustration}</g>
          : illustration)}
    </g>
  })
  const destination = state.groundCommand && ['moving', 'arrived', 'failed'].includes(state.groundCommand.status) && (state.frames.ship.parent !== null || state.groundCommand.destination.frame === 'ship:scout') && worldToView(relativeAnchor(state.groundCommand.destination, state.camera.anchor, state.frames), state.frames)
  return <>
  {artwork.error && <div className="world-art-error" role="alert">
    Some world artwork could not load. <button onClick={artwork.retry}>Retry artwork</button>
  </div>}
  <svg onContextMenu={event => {
    event.preventDefault()
    const matrix = event.currentTarget.getScreenCTM()
    if (!matrix) return
    const svg = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse())
    const anchor = groundFromView(state, [svg.x, svg.y])
    const id = (event.target as Element).closest('[data-entity]')?.getAttribute('data-entity')
    const objectId = interactiveAt(state, anchor, id)
    event.currentTarget.focus({ preventScroll: true })
    onGround?.(anchor, objectId, { x: event.clientX, y: event.clientY })
  }} onPointerLeave={onCancelGround} onPointerCancel={onCancelGround} onLostPointerCapture={onCancelGround} tabIndex={0} ref={viewport} className="top-down-scene" data-camera-scale={state.camera.pixelsPerM} data-camera-panned={String(!!state.camera.panned)} viewBox={`${-width / 2} ${-height / 2} ${width} ${height}`} shapeRendering="geometricPrecision" textRendering="geometricPrecision" role="img" aria-label={state.frames.ship.parent === null ? "Strict overhead Scout interior in flight" : "Strict overhead Scout and planetary test site"} data-world-raster-scale={rasterScale} data-render-ready={String(artwork.ready)} data-art-resources={JSON.stringify(artwork.stats)} aria-busy={!artwork.ready && !artwork.error}>
    {passes.map(({ id: layer, beforeTerrain, sprites }) => <g key={layer} data-layer={layer}>
      {layer === 'terrain' && state.scoutLocation.binding && <rect x={-width/2} y={-height/2} width={width} height={height} fill="#b5a58b" />}
      {renderItems(beforeTerrain)}
      {layer === 'terrain' && showWorld && <GeneratedGround state={state} width={width} height={height} rasterScale={rasterScale} detail={quality.environment / 100} diagnostics={diagnosticsEnabled} />}
      {layer === 'people' && showWorld && quality.shadows === 'dynamic' && <g data-world-shadows="dynamic" pointerEvents="none">
        {items.filter(item => ['core', 'fabricator', 'storage', 'bed'].includes(item.entity.appearance) && intersects(item))
          .map(item => <ellipse key={item.entity.id} transform={svgSpriteTransform(item, pixelsPerM)}
            cx=".14" cy=".18" rx={(item.entity.footprint?.halfWidth ?? .28) + .17}
            ry={(item.entity.footprint?.halfHeight ?? .24) + .09} fill="#18242a" opacity=".23" />)}
      </g>}
      {renderItems(sprites)}
      {layer === 'interior' && boardingDoor && renderItems([boardingDoor])}
      {layer === 'cues' && destination && <g pointerEvents="none" data-destination={state.groundCommand!.status} transform={`translate(${destination[0] * state.camera.pixelsPerM} ${destination[1] * state.camera.pixelsPerM})`} className="command-destination" fill="none" stroke={state.groundCommand!.status === 'failed' ? 'var(--ui-danger)' : 'var(--ui-active)'} strokeWidth="2">
        <circle r="9" /><path d="M -14 0 H -5 M 5 0 H 14 M 0 -14 V -5 M 0 5 V 14" />
      </g>}
      {layer === 'cues' && <FieldMarkCues state={state} session={session} />}
      {layer === 'cues' && showWorld && selection && intersects(selection, 0, true) && <g data-selection={selection.entity.id} transform={svgSpriteTransform(selection, pixelsPerM)} fill="none" stroke="#bce9df" strokeWidth=".08"><WorldSprite request={request(selection.entity, true)} outcome={artwork.get(request(selection.entity, true))} /></g>}
    </g>)}
    {diagnosticsEnabled && showWorld && <RenderDiagnostics state={state} items={items} art={artwork.stats} activeCues={activeCues} />}
  </svg>
  {diagnosticsEnabled && showWorld && <FrameDiagnostics backend="SVG CPU"
    details={`${artwork.stats.count} images · ${artwork.stats.decodedBytes} estimated decoded RGBA8 bytes · ${state.frames.surface?.chart?.id ?? 'authored terrain'}`}
    measure={() => `audio ${activeCues().join(', ') || 'idle'}`} />}
  </>
}

let navigationPresentation: Promise<typeof PanoramicNavigation> | undefined
function loadNavigationPresentation() {
  navigationPresentation ??= import('./PanoramicNavigation.tsx').then(module => module.PanoramicNavigation)
    .catch(error => { navigationPresentation = undefined; throw error })
  return navigationPresentation
}

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
  useEffect(() => {
    const input = new NavigationInput(), clock = new TopDownClock(), timer = new FrameTimer(performance.now())
    let raf = 0, publishedError = ''
    const release = () => { input.clear(); clock.reset(); timer.resume(performance.now()) }
    const clear = () => { setGearOpen(false);closeObjectMenu(); release(); session.current?.cancel(); container.current?.close(); current.current = cancelInteraction(current.current); if (session.current) session.current.world = current.current; session.current?.checkpoint(); setState(current.current) }
    const keydown = (event: KeyboardEvent) => {
      if (event.defaultPrevented) return
      if (event.code === 'Escape' && inventoryExpanded.current && !container.current?.opened) {
        event.preventDefault(); release(); toggleInventory(false, true); return
      }
      if (event.code === 'KeyI' && !event.repeat && acceptsFlightShortcut(event.target) && current.current.cockpit.view === 'world' && !container.current?.opened) {
        event.preventDefault(); release(); toggleInventory(!inventoryExpanded.current); return
      }
      if (gameplay && !event.repeat && (event.code === 'KeyM' || event.code === 'KeyG') && acceptsFlightShortcut(event.target) && current.current.cockpit.view === 'world') {
        event.preventDefault(); release()
        if (event.code === 'KeyM') { session.current?.cancel(); setNotebookOpen(true) }
        else setScoutGuiding(value => !value)
        return
      }
      if (event.code === 'KeyP' && !event.repeat && acceptsFlightShortcut(event.target)) {
        paused.current = !paused.current; clear(); current.current = { ...current.current }; setState(current.current); return
      }
      if (paused.current || session.current?.survival.health === 0 || current.current.scoutLocation.ship.mode === 'destroyed' && event.code !== 'Escape') return
      if (event.code === 'KeyE' && isCockpitTarget(current.current.selectedId) && session.current && (!!session.current.error || !survivalPolicy(session.current.survival).canWork)) return
      if (current.current.cockpit.view === 'radar') return
      if (event.code === 'Space' && event.target instanceof SVGSVGElement && event.target.classList.contains('top-down-scene') && !document.querySelector('dialog[open]')) {
        event.preventDefault(); current.current = followWorldCamera(current.current)
        if (session.current) session.current.world = current.current
        setState(current.current); return
      }
      if (container.current?.opened && event.code === 'KeyE') return
      if (!event.repeat && (event.code === 'Escape' || (event.code === 'KeyE' && acceptsFlightShortcut(event.target)))) {
        event.preventDefault(); input.clear(); current.current = cancelGroundMovement(current.current); if (session.current) session.current.world = current.current
        if (container.current?.opened && event.code === 'Escape') {
          container.current.close(); current.current = container.current.session.world
        } else if (event.code === 'KeyE' && openEquipment(current.current)) {
          current.current = container.current!.session.world
        } else if (event.code === 'KeyE' && current.current.selectedId === BOARDING_DOOR_ID && session.current) {
          session.current.world = current.current
          session.current.toggleBoardingDoor(session.current.snapshot.revision)
          current.current = session.current.world
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
      if (axis && acceptsFlightShortcut(event.target)) { event.preventDefault(); current.current = cancelGroundMovement(current.current); input.press(event.code, axis) }
    }
    const keyup = (event: KeyboardEvent) => input.release(event.code)
    const frame = (now: number) => {
      const dt = timer.sample(now, document.hidden || paused.current || gameplay && (!session.current || !!session.current.error || session.current.survival.health === 0))
      const inWorld = current.current.cockpit.view === 'world'
      if (!inWorld) { input.clear(); current.current = cancelGroundMovement(current.current) }
      if (paused.current || document.hidden) { input.clear(); clock.reset() }
      const controls = input.controls
      current.current = clock.advance(current.current, dt, viewToWorld([Number(controls.right) - Number(controls.left), Number(controls.down) - Number(controls.up)], current.current.frames), (world, direction) => {
        if (!session.current) return advanceObjectCommand({ world: stepTopDown(world, direction), paused: paused.current })
        if (session.current.error || session.current.survival.health === 0) return world
        const locationBefore = { ...world.scoutLocation }
        if (gameplay) navigation.advance(1 / 60, controlsCockpit(world.cockpit, LOCAL_PLAYER) && survivalPolicy(session.current.survival).canWork ? navigation.pilotInput.controls : undefined)
        const next = session.current.advance(stepTopDown(world, direction, survivalPolicy(session.current.survival).movement))
        if (next.tick === world.tick) Object.assign(world.scoutLocation, locationBefore)
        const commanded = advanceObjectCommand({ world: next, session: session.current, paused: paused.current }, openEquipment)
        session.current.world = commanded
        return commanded
      })
      if (session.current) {
        session.current.world = current.current
        container.current?.synchronize(current.current)
        current.current = session.current.world
        navigation.clock.state = setMass(navigation.clock.state, DRY_MASS_KG + session.current.payload().shipGrams / 1000)
      }
      // Failed atomic saves roll the domain world back to the same object.
      // Publish the changed error once even when React would bail out on identity.
      const error = session.current?.error ?? ''
      if (error !== publishedError) { current.current = { ...current.current }; publishedError = error }
      setState(current.current)
      raf = requestAnimationFrame(frame)
    }
    const focus = () => { release(); current.current = cancelGroundMovement(current.current); if (session.current) session.current.world = current.current; if (document.activeElement?.closest('.hud-utility, dialog')) session.current?.cancel(); if (container.current?.opened && !document.activeElement?.closest('dialog')) { container.current.close(); current.current = session.current!.world; setState({ ...current.current }) } }
    window.addEventListener('keydown', keydown); window.addEventListener('keyup', keyup)
    window.addEventListener('blur', clear); window.addEventListener('resize', clear)
    document.addEventListener('visibilitychange', clear); window.addEventListener('focusin', focus)
    window.addEventListener('pagehide', clear)
    raf = requestAnimationFrame(frame)
    return () => {
      cancelAnimationFrame(raf)
      window.removeEventListener('keydown', keydown); window.removeEventListener('keyup', keyup)
      window.removeEventListener('blur', clear); window.removeEventListener('resize', clear)
      document.removeEventListener('visibilitychange', clear); window.removeEventListener('focusin', focus)
      window.removeEventListener('pagehide', clear)
    }
  }, [])
  const selected = state.entities.find(e => e.id === state.selectedId)
  const update = (next: TopDownState) => { current.current = next; setState(next) }
  const cockpitTarget = gameplay && isCockpitTarget(selected?.id ?? null)
  const [panoramic, setPanoramic] = useState<{ component?: typeof PanoramicNavigation; error?: boolean }>({})
  const [navigationRetry, setNavigationRetry] = useState(0)
  const needsPanoramic = cockpitTarget || state.cockpit.view === 'radar'
  useEffect(() => {
    if (!needsPanoramic) return
    let active = true
    void loadNavigationPresentation().then(
      component => { if (active) setPanoramic({ component }) },
      () => { if (active) setPanoramic({ error: true }) },
    )
    return () => { active = false }
  }, [needsPanoramic, navigationRetry])
  const NavigationPresentation = panoramic.component
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
    {gameplay && <aside className="survival-hud" aria-label="Survival condition">
      {vital ? <>
        <StatusMessage priority={warning ? 'critical' : 'status'}>{[paused.current ? 'Paused · active time stopped' : '', warning].filter(Boolean).join(' · ')}</StatusMessage>
        <div className="vital-meters">
        <div className="ui-meter"><span>Health: {Math.ceil(vital.health)} / 200 HP</span><meter aria-label="Health" min={0} max={200} value={vital.health} aria-valuetext={`${vital.health} / 200 HP`} /></div>
        <Meter label="Hunger" value={Math.ceil(vital.hunger / METER * 100)} max={100} unit="" />
        <Meter label="Fatigue" value={Math.ceil(vital.fatigue / METER * 100)} max={100} unit="" />
        </div>
        {vital.health === 0 ? <button onClick={event => { event.currentTarget.blur(); if (resourceSession.freshRun()) observation.reset(navigation.clock.state); setState({ ...current.current }) }}>Start fresh character · keep world resources</button> : <button onClick={event => { event.currentTarget.blur(); paused.current = !paused.current; current.current = cancelGroundMovement(current.current); resourceSession.world = current.current; resourceSession.cancel(); resourceSession.checkpoint(); setState({ ...current.current }) }}>{paused.current ? 'Resume' : 'Pause'} [P]</button>}
      </> : <p>{resourceSession?.error || saveStatus}</p>}
      {resourceSession && <EquipmentHud readout={equipmentHudReadout(resourceSession,gearNotices)} />}
      {resourceSession?.error && <StatusMessage priority="error">{resourceSession.error}</StatusMessage>}
      {persistentMessages[1].text && <StatusMessage priority="critical">{persistentMessages[1].text}</StatusMessage>}
      {state.scoutLocation.boardingDoor === 'closed' && <p data-boarding-door-status>Aft door: closed{state.scoutLocation.binding === null ? ' · flight lock' : ''}</p>}
      {state.cockpit.view === 'radar' && resourceSession?.feedback && <StatusMessage>{resourceSession.feedback}</StatusMessage>}
      <Button data-field-help variant="secondary" onClick={() => setHelpOpen(true)}>Help</Button>
    </aside>}
    <Dialog open={helpOpen} initialFocus="heading" title="Surface controls" onDismiss={() => setHelpOpen(false)}>
      {resourceSession?.error && <StatusMessage priority="error">{resourceSession.error}</StatusMessage>}
      <p>Right-click ground: move around obstacles. Right-click an object: always open its context menu; no action runs until you left-click an enabled option.</p>
      <p>W A S D: walk and override queued movement or approach. E: nearby action keyboard fallback. Use arrows, Home / End and Enter in object menus.</p>
      <p>Escape: dismiss the topmost menu or dialog first, then inventory; otherwise cancel the route, approach or interruptible work. In the cockpit, its observation controls take priority.</p>
      <p>Wheel over world: zoom · Middle-mouse drag: pan · Space with world focus: recenter / follow player. The next movement or approach resumes follow. Panels and text fields keep their own input.</p>
      <p>I: inventory · P: pause active time. Menus do not pause time. Movement, Escape or stowing the tool interrupts gathering; partial work is saved.</p>
      <p>M: inspect / mark a field location · G: toggle local Scout guidance. Field notes save with your expedition.</p>
      <button onClick={() => {setHelpOpen(false);session.current?.cancel();setNotebookOpen(true)}}>Inspect / mark</button>
      <button onClick={() => {setHelpOpen(false);setScoutGuiding(value => !value)}}>Return to Scout</button>
      <p>In the cockpit use the observation controls and Exit cockpit to stand up.</p>
      {gameplay && <ExpeditionAudioSettings audio={audio} />}
      {gameplay && <WorldGraphicsSettings graphics={graphics} />}
    </Dialog>
    {state.cockpit.view === 'radar' && <div inert={paused.current || !!resourceSession?.error}>
      {!NavigationPresentation && <div className="world-art-error">
        <p role={panoramic.error ? 'alert' : 'status'}>{panoramic.error ? 'Navigation artwork could not load.' : 'Loading navigation…'}</p>
        {panoramic.error && <button onClick={() => { setPanoramic({}); setNavigationRetry(value => value + 1) }}>Retry navigation</button>}
        <button onClick={() => update(leaveCockpit(current.current))}>Exit cockpit</button>
      </div>}
      {NavigationPresentation && <NavigationPresentation graphics={graphics} conditionWarning={warning} navigation={navigation} view={observation} advanceSimulation={false}
        isPaused={() => paused.current || !session.current || !!session.current.error || session.current.survival.health === 0 || !controlsCockpit(current.current.cockpit, LOCAL_PLAYER)}
        canPilot={() => !paused.current && !!session.current && !session.current.error && survivalPolicy(session.current.survival).canWork && controlsCockpit(current.current.cockpit, LOCAL_PLAYER)}
        environment={resourceSession ?? undefined}
        landing={{ preview: bodyId => session.current?.landingPreview(bodyId) ?? { denial: 'Expedition unavailable.' }, activate: bodyId => {
          if (!session.current || paused.current) return
          session.current.world = current.current
          session.current.transitionScout({ kind: 'begin-landing', bodyId }, session.current.snapshot.revision, current.current.cockpit)
          update(session.current.world)
        } }}
        takeoff={{ denial: () => paused.current ? 'Resume the expedition before takeoff.' : session.current?.scoutCommandDenial() ?? (session.current ? null : 'Expedition unavailable.'), activate: () => { if (!session.current || paused.current) return; session.current.world = current.current; session.current.transitionScout({ kind: 'begin-takeoff' }, session.current.snapshot.revision, current.current.cockpit); update(session.current.world) } }} onExit={() => update(leaveCockpit(current.current))} />}
    </div>}
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
      {gameplay && <aside className="resource-hud" aria-label="Resource inventory" data-gathering={resourceSession?.activeSourceId ?? ''}>
        <button ref={inventoryTrigger} aria-expanded={inventoryOpen} aria-controls="field-inventory" onClick={() => toggleInventory(!inventoryOpen)}>Field inventory <KeyHint>I</KeyHint></button>
        {resourceSession && usage && <>
          <p data-inventory-weight><strong>{usage.grams / 1000}</strong> / {resourceSession.snapshot.inventory.capacityGrams / 1000} kg</p>
          <p data-inventory-slots><strong>{usage.slots}</strong> / {resourceSession.snapshot.inventory.capacitySlots} slots</p>
          <div id="field-inventory" ref={inventoryPanel} className="hud-utility" data-ui-utility hidden={!inventoryOpen} tabIndex={-1} onKeyDown={event => { if (event.code !== 'Escape') event.stopPropagation() }}>
          <UtilityPanel title="Carried resources and items">
          <ul aria-label="Raw resources">{Object.entries(RESOURCE_CATALOG).map(([id, entry]) => <li key={id}>{entry.name} <span data-resource={id}>{(resourceSession.snapshot.inventory.quantities[id] ?? 0) / 1000} kg</span></li>)}</ul>
          <p className="resource-note">Each carried raw type uses 1 slot.</p>
          <ul aria-label="Count items">{Object.entries(resourceSession.snapshot.inventory.items).map(([id, count]) => {
            const meta = resourceSession.itemCatalog[id]!
            return <li className="count-item" key={id} data-item={id}><span>{meta.name ?? id} × {count}</span><span>{count * meta.gramsPerItem / 1000} kg · {Math.ceil(count / meta.stackSize)} slots</span></li>
          })}</ul>
          <Button onClick={()=>{resourceSession.cancel();container.current?.close();setGearOpen(true)}}>Equipment and storage</Button>
          <MarkerDeployment session={resourceSession} blocked={paused.current || resourceSession.survival.health === 0 || !!resourceSession.error} onChange={() => setState({...current.current})} />
          {!Object.keys(resourceSession.snapshot.inventory.items).length && <p data-empty-items>No items carried.</p>}
          <p className="resource-note">{resourceSession.error || saveStatus}</p>
          <Button onClick={() => toggleInventory(false, true)}>Close inventory</Button>
          </UtilityPanel></div>
        </>}
        <button disabled={!resourceSession || !!resourceSession.error || resourceSession.survival.health === 0 || paused.current} aria-pressed={resourceSession?.equipped ?? false} onClick={event => { resourceSession?.toggleTool(); event.currentTarget.blur(); setState({ ...current.current }) }}><EquipmentIcon />{resourceSession?.equipped ? 'Stow gathering tool' : 'Equip gathering tool'}</button>
        {(!resourceSession || resourceSession.error) && <StatusMessage announce={!resourceSession?.error}>{resourceSession?.error || saveStatus}</StatusMessage>}
      </aside>}
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
