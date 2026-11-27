import type { ReactNode } from 'react'
import { Dialog } from './ui/primitives.tsx'

export function HelpDialog({onClose,panoramic=false, warnings, children}:{onClose:()=>void;panoramic?:boolean;warnings?:ReactNode; children?:ReactNode}) {
  return <div className="expedition-ui"><Dialog open initialFocus={panoramic ? 'heading' : 'dismiss'} title="Navigation help" onDismiss={onClose} dismissLabel="Close help">
    {warnings}
    <dl>
      <dt>W / A / S / D</dt><dd>{panoramic?'Look up / left / down / right without turning the ship.':'Pan the camera up / left / down / right in the navigation view.'}</dd>
      <dt>Mouse</dt><dd>{panoramic?'Left-drag to observe in any direction. Wheel changes optical FOV from 75° to 0.1°.':'Left-drag empty space to orbit; right-drag to pan. Release to end the drag. Wheel zooms smoothly within the view limits.'}</dd>
      <dt>Arrow keys</dt><dd>Turn ship pitch (up/down) and yaw (left/right). Manual turning cancels Set direction; camera movement does not.</dd>
      {panoramic && <><dt>C / Contacts</dt><dd>Open the searchable scanned-contact browser. Look at rotates the camera once; Set direction rotates only the ship. Solid compass glyphs face forward; hollow glyphs face rear. Amber ship heading and light-green velocity lines stay inside the compass; their triangle and ring remain distinct when coincident. Cyan marks the camera axis and open reticle. Muted contacts brighten on hover or focus; cyan brackets mark selection, amber diamonds mark an active landing destination, and red ! diamonds signal dangerous approach. Stationary hides velocity.</dd></>}
      {panoramic && <><dt>Grounded Scout</dt><dd>Close the aft boarding door, then use Take off from the authorized powered seat. Automatic launch cannot be cancelled; throttle, steering and Set direction become available when flying. Scanner and observation remain active; Look at can observe other scanned bodies. Exit cockpit returns you standing; power loss keeps you seated until you stand or explicitly reopen navigation. Your observation direction and zoom last for this session.</dd></>}
      <dt>Selection</dt><dd>Click a contact or choose a scanned contact in the list, then use Set direction for one-time alignment. It applies no thrust.</dd>
      <dt>Throttle / Space</dt><dd>Use the throttle slider for signed thrust; Space sets zero throttle while navigating. Existing velocity keeps coasting.</dd>
      <dt>Reset camera</dt><dd>{panoramic?'Reset observation to current ship heading and 75° FOV.':'Restore the default camera view without changing the ship.'}</dd>
      <dt>Keyboard focus</dt><dd>Tab moves between controls; Enter activates buttons. Form fields keep their typing and slider keys. Escape or Close help dismisses this dialog.</dd>
    </dl>
    {children}
  </Dialog></div>
}
