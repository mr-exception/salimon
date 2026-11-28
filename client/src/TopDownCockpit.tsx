import { survivalPolicy } from './survival.ts'
import type { PanoramicNavigation } from './PanoramicNavigation.tsx'
import { ObservationCameraState } from './observationCamera.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { controlsCockpit, LOCAL_PLAYER, leaveCockpit } from './cockpit.ts'
import { useEffect, useState } from 'react'
import { type TopDownState } from './topDownRuntime.ts'
import { useWorldGraphics } from './rendering/WorldGraphicsSettings.tsx'
import { ResourceSession } from './resourceSession.ts'
import type { MutableRefObject } from 'react'
interface TopDownCockpitProps {
  state: TopDownState
  preload: boolean
  graphics: ReturnType<typeof useWorldGraphics>
  warning: string
  navigation: NavigationRuntime
  observation: ObservationCameraState
  current: MutableRefObject<TopDownState>
  session: MutableRefObject<ResourceSession | null>
  paused: MutableRefObject<boolean>
  update(next: TopDownState): void
}

let navigationPresentation: Promise<typeof PanoramicNavigation> | undefined
function loadNavigationPresentation() {
  navigationPresentation ??= import('./PanoramicNavigation.tsx').then(module => module.PanoramicNavigation)
    .catch(error => { navigationPresentation = undefined; throw error })
  return navigationPresentation
}

/** Lazy observation presentation; simulation and command ownership stay in the world. */
export function TopDownCockpit({ state, preload, graphics, warning, navigation, observation, current, session, paused, update }: TopDownCockpitProps) {
  const [panoramic, setPanoramic] = useState<{ component?: typeof PanoramicNavigation; error?: boolean }>({})
  const [navigationRetry, setNavigationRetry] = useState(0)
  const needsPanoramic = preload || state.cockpit.view === 'radar'
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
  return <>{state.cockpit.view === 'radar' && <div inert={paused.current || !!resourceSession?.error}>
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
    </div>}</>
}
