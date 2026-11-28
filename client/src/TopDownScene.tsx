import { CameraDrag, panWorldCamera, zoomWorldCamera } from './topDownCamera.ts'
import { FieldMarkCues } from './ExplorationHud.tsx'
import { GeneratedGround } from './GeneratedGround.tsx'
import { BOARDING_DOOR_ID } from './scoutDoor.ts'
import { groundFromView, interactiveAt } from './topDownNavigation.ts'
import { relativeAnchor, rotate2, worldToView, type Anchor } from './topDownFrames.ts'
import { useEffect, useMemo, useRef, useState } from 'react'
import { drawList, PLAYER_ID, type DrawItem, type TopDownState } from './topDownRuntime.ts'
import { scoutWallRequests, spriteBounds, spriteRequest, usePresentationVisibility, useWorldSprites, WorldSprite } from './topDownSprites.tsx'
import { QUALITY, renderDpr, type WorldQuality } from './rendering/graphicsPreferences.ts'
import { diagnosticsEnabled, FrameDiagnostics, RenderDiagnostics } from './rendering/RenderDiagnostics.tsx'
import { isScoutModule, scoutArtObstructs, scoutWallJoins } from './rendering/scoutArt.ts'
import { characterArt, ScoutCharacter } from './rendering/ScoutCharacter.tsx'
import { scoutFixture, ScoutFixtureSignal } from './rendering/ScoutFixtures.tsx'
import { machineState, scoutMachine, ScoutMachineSignal } from './rendering/ScoutMachine.tsx'
import { ScoutModule, ScoutWall } from './rendering/ScoutModules.tsx'
import { svgArtIntersects, svgLayers, svgSpriteTransform } from './rendering/svgWorld.ts'
import { ResourceSession } from './resourceSession.ts'
import { SCOUT_RESOURCE_CONTAINER_ID } from './scoutResourceContainer.ts'


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