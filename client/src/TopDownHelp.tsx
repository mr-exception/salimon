import { ExpeditionAudioSettings, useExpeditionAudio } from './audio/ExpeditionAudio.tsx'
import { useWorldGraphics, WorldGraphicsSettings } from './rendering/WorldGraphicsSettings.tsx'
import { ResourceSession } from './resourceSession.ts'
import { Dialog } from './ui/primitives.tsx'
import { StatusMessage } from './ui/Feedback.tsx'

interface TopDownHelpProps {
  open: boolean
  gameplay: boolean
  resourceSession: ResourceSession | null
  audio: ReturnType<typeof useExpeditionAudio>
  graphics: ReturnType<typeof useWorldGraphics>
  onDismiss(): void
  onNotebook(): void
  onGuidance(): void
}

export function TopDownHelp({ open, gameplay, resourceSession, audio, graphics, onDismiss, onNotebook, onGuidance }: TopDownHelpProps) {
  return <Dialog open={open} initialFocus="heading" title="Surface controls" onDismiss={onDismiss}>
      {resourceSession?.error && <StatusMessage priority="error">{resourceSession.error}</StatusMessage>}
      <p>Right-click ground: move around obstacles. Right-click an object: always open its context menu; no action runs until you left-click an enabled option.</p>
      <p>W A S D: walk and override queued movement or approach. E: nearby action keyboard fallback. Use arrows, Home / End and Enter in object menus.</p>
      <p>Escape: dismiss the topmost menu or dialog first, then inventory; otherwise cancel the route, approach or interruptible work. In the cockpit, its observation controls take priority.</p>
      <p>Wheel over world: zoom · Middle-mouse drag: pan · Space with world focus: recenter / follow player. The next movement or approach resumes follow. Panels and text fields keep their own input.</p>
      <p>I: inventory · P: pause active time. Menus do not pause time. Movement, Escape or stowing the tool interrupts gathering; partial work is saved.</p>
      <p>M: inspect / mark a field location · G: toggle local Scout guidance. Field notes save with your expedition.</p>
      <button onClick={onNotebook}>Inspect / mark</button>
      <button onClick={onGuidance}>Return to Scout</button>
      <p>In the cockpit use the observation controls and Exit cockpit to stand up.</p>
      {gameplay && <ExpeditionAudioSettings audio={audio} />}
      {gameplay && <WorldGraphicsSettings graphics={graphics} />}
    </Dialog>
}
