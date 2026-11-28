import { advanceObjectCommand } from './topDownCommands.ts'
import { followWorldCamera } from './topDownCamera.ts'
import { BOARDING_DOOR_ID } from './scoutDoor.ts'
import { cancelGroundMovement } from './topDownNavigation.ts'
import { viewToWorld } from './topDownFrames.ts'
import { survivalPolicy } from './survival.ts'
import { DRY_MASS_KG, setMass } from './simulation.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { controlsCockpit, LOCAL_PLAYER, isCockpitTarget, leaveCockpit } from './cockpit.ts'
import { useEffect } from 'react'
import { acceptsFlightShortcut, FrameTimer, NavigationInput, panAxisForCode } from './input.ts'
import { cancelInteraction, requestInteraction, stepTopDown, TopDownClock, type TopDownState } from './topDownRuntime.ts'
import { ResourceSession } from './resourceSession.ts'
import { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import type { MutableRefObject, Dispatch, SetStateAction } from 'react'
interface TopDownLoopOptions {
  gameplay: boolean
  current: MutableRefObject<TopDownState>
  session: MutableRefObject<ResourceSession | null>
  container: MutableRefObject<ResourceContainerInteraction | null>
  navigation: NavigationRuntime
  paused: MutableRefObject<boolean>
  inventoryExpanded: MutableRefObject<boolean>
  closeObjectMenu(): void
  setGearOpen: Dispatch<SetStateAction<boolean>>
  toggleInventory(open: boolean, restoreFocus?: boolean): void
  setNotebookOpen: Dispatch<SetStateAction<boolean>>
  setScoutGuiding: Dispatch<SetStateAction<boolean>>
  setState: Dispatch<SetStateAction<TopDownState>>
  openEquipment(world: TopDownState): boolean
}

/** Retain the initial controllers for the sole world clock and keyboard lifecycle. */
export function useTopDownLoop({ gameplay, current, session, container, navigation, paused, inventoryExpanded, closeObjectMenu, setGearOpen, toggleInventory, setNotebookOpen, setScoutGuiding, setState, openEquipment }: TopDownLoopOptions) {
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
}
