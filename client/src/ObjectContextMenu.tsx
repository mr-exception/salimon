import { useLayoutEffect, useRef, useState } from 'react'
import { useDismissLayer } from './ui/interaction.ts'
import type { ObjectAction, ObjectActionId } from './topDownActions.ts'

export interface ObjectMenuSelection { targetId: string; x: number; y: number; sequence: number }
export function ObjectContextMenu({ selection, label, actions, onClose, onChoose }: {
  selection: ObjectMenuSelection; label: string; actions: ObjectAction[]
  onClose: () => void; onChoose: (id: ObjectActionId) => void
}) {
  const ref = useRef<HTMLDivElement>(null)
  const [point, setPoint] = useState({ x: selection.x, y: selection.y })
  const restore = () => document.querySelector<SVGSVGElement>('.top-down-scene')?.focus({ preventScroll: true })
  useDismissLayer(true, () => { onClose(); restore() })
  useLayoutEffect(() => {
    const menu = ref.current!
    const clamp = () => {
      const box = menu.getBoundingClientRect(), margin = 8
      setPoint({ x: Math.max(margin, Math.min(selection.x, innerWidth - box.width - margin)), y: Math.max(margin, Math.min(selection.y, innerHeight - box.height - margin)) })
    }
    clamp()
    const observer = new ResizeObserver(clamp)
    observer.observe(menu)
    menu.querySelector<HTMLElement>('[role=menuitem]')?.focus({ preventScroll: true })
    const outside = (event: PointerEvent) => { if (!menu.contains(event.target as Node)) onClose() }
    const focus = (event: FocusEvent) => { if (!menu.contains(event.target as Node)) onClose() }
    const blur = () => onClose()
    document.addEventListener('pointerdown', outside, true)
    document.addEventListener('focusin', focus)
    window.addEventListener('blur', blur)
    window.addEventListener('resize', clamp)
    document.addEventListener('visibilitychange', blur)
    return () => {
      observer.disconnect()
      if (menu.contains(document.activeElement) && document.hasFocus()) restore()
      document.removeEventListener('pointerdown', outside, true)
      document.removeEventListener('focusin', focus)
      window.removeEventListener('blur', blur)
      window.removeEventListener('resize', clamp)
      document.removeEventListener('visibilitychange', blur)
    }
  }, [selection, onClose])
  return <div ref={ref} className="object-context-menu" role="menu" aria-label={`Actions for ${label}`} style={{ left: point.x, top: point.y }}
    onContextMenu={event => event.preventDefault()} onPointerDown={event => event.stopPropagation()}
    onKeyDown={event => {
      event.stopPropagation()
      const rows = [...ref.current!.querySelectorAll<HTMLElement>('[role=menuitem]')]
      const index = rows.indexOf(document.activeElement as HTMLElement)
      if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
        event.preventDefault()
        rows[event.key === 'Home' ? 0 : event.key === 'End' ? rows.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + rows.length) % rows.length]?.focus()
      } else if (event.key === 'Tab') { event.preventDefault(); onClose(); restore() }
    }}>
    <p className="object-context-title">{label}</p>
    {actions.map(action => <button key={action.id} role="menuitem" tabIndex={-1} aria-disabled={!action.available} aria-describedby={action.reason ? `object-action-${action.id}-reason` : undefined}
      onClick={() => { if (action.available) { onClose(); restore(); onChoose(action.id) } }}>
      <span>{action.label}</span>
      {action.available && action.requiresApproach && <small>Approach, then {action.label.toLowerCase()}</small>}
      {action.reason && <small id={`object-action-${action.id}-reason`}>{action.reason}</small>}
    </button>)}
  </div>
}
