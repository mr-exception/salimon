import { useEffect, useRef } from 'react'

export function HelpDialog({onClose,panoramic=false}:{onClose:()=>void;panoramic?:boolean}) {
  const dialog=useRef<HTMLDialogElement>(null)
  useEffect(()=>{dialog.current?.showModal()},[])
  return <dialog ref={dialog} aria-labelledby="help-title" onClose={onClose}>
    <h2 id="help-title">Navigation help</h2>
    <dl>
      <dt>W / A / S / D</dt><dd>{panoramic?'Look up / left / down / right without turning the ship.':'Pan the camera up / left / down / right in the navigation view.'}</dd>
      <dt>Mouse</dt><dd>{panoramic?'Left-drag to observe in any direction. Wheel changes optical FOV from 75° to 0.1°.':'Left-drag empty space to orbit; right-drag to pan. Release to end the drag. Wheel zooms smoothly within the view limits.'}</dd>
      <dt>Arrow keys</dt><dd>Turn ship pitch (up/down) and yaw (left/right). Manual turning cancels Set direction; camera movement does not.</dd>
      {panoramic && <><dt>C / Contacts</dt><dd>Open the searchable scanned-contact browser. Look at rotates the camera once; Set direction rotates only the ship. Solid compass glyphs face forward; hollow glyphs face rear. Yellow heading and green velocity remain distinct when coincident. Cyan is the view axis. Stationary hides velocity.</dd></>}
      <dt>Selection</dt><dd>Click a contact or choose a scanned contact in the list, then use Set direction for one-time alignment. It applies no thrust.</dd>
      <dt>Throttle / Space</dt><dd>Use the throttle slider for signed thrust; Space sets zero throttle while navigating. Existing velocity keeps coasting.</dd>
      <dt>Reset camera</dt><dd>{panoramic?'Reset observation to current ship heading and 75° FOV.':'Restore the default camera view without changing the ship.'}</dd>
      <dt>Keyboard focus</dt><dd>Tab moves between controls; Enter activates buttons. Form fields keep their typing and slider keys. Escape or Close help dismisses this dialog.</dd>
    </dl>
    <form method="dialog"><button autoFocus>Close help</button></form>
  </dialog>
}
