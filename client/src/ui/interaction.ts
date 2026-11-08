import { useEffect, useRef } from 'react'

type Layer = { dismiss: () => void; modal: boolean }
const layers: Layer[] = []
function escape(event: KeyboardEvent) {
  if (event.key !== 'Escape' || event.defaultPrevented) return
  // Native top-layer dialogs outrank ordinary utilities regardless of opening order.
  const layer = [...layers].reverse().find(layer => layer.modal) ?? layers.at(-1)
  if (!layer) return
  event.preventDefault(); event.stopImmediatePropagation()
  if (!event.repeat) layer.dismiss()
}
/** Register on opening, not each render. Owners retain cancellation authority. */
export function useDismissLayer(open: boolean, onDismiss: () => void, modal = false) {
  const callback = useRef(onDismiss)
  callback.current = onDismiss
  useEffect(() => {
    if (!open) return
    const layer = { dismiss: () => callback.current(), modal }
    layers.push(layer)
    if (layers.length === 1) window.addEventListener('keydown', escape, true)
    return () => {
      layers.splice(layers.indexOf(layer), 1)
      if (!layers.length) window.removeEventListener('keydown', escape, true)
    }
  }, [open, modal])
}
export function restoreFocus(element: Element | null) {
  if (element instanceof HTMLElement && element.isConnected && element.getClientRects().length && !element.closest('[inert], [hidden]') && !element.matches(':disabled')) element.focus({ preventScroll: true })
}
