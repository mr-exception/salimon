import { useEffect, useMemo, useRef } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import { PerspectiveCamera } from 'three'
import { acceptsFlightShortcut, NavigationInput, panAxisForCode } from './input.ts'
import { ObservationCameraState } from './observationCamera.ts'

/** Reusable perspective adapter. The parent owns view lifetime and flight input.
 * Ship-relative scene geometry makes the render camera's zero translation the
 * exact physical ship origin, even beyond Number.MAX_SAFE_INTEGER cells.
 */
export function ObservationCamera({ view, isPaused = () => false }: { view: ObservationCameraState; isPaused?: () => boolean }) {
  const { set, get, gl, size } = useThree()
  const paused = useRef(isPaused)
  paused.current = isPaused
  const camera = useMemo(() => new PerspectiveCamera(75, 1, 0.01, 1e9), [])
  const held = useMemo(() => new NavigationInput(), [])
  useEffect(() => {
    const previous = get().camera
    set({ camera })
    return () => { set({ camera: previous }) }
  }, [camera, get, set])
  useEffect(() => {
    camera.aspect = size.width / Math.max(1, size.height)
    camera.updateProjectionMatrix()
  }, [camera, size.width, size.height])
  useEffect(() => {
    const canvas = gl.domElement
    // Scene markers are DOM siblings of the canvas. Their hovered wheel must
    // reach the same optical input, while panels retain native scrolling.
    const wheelSurface = canvas.closest('[data-observation-surface]') ?? canvas
    const previousTouchAction = canvas.style.touchAction
    const previousTabIndex = canvas.getAttribute('tabindex')
    canvas.style.touchAction = 'none'
    canvas.tabIndex = 0
    let drag: { id: number; x: number; y: number } | null = null
    const permitted = () => !document.hidden && !paused.current() && acceptsFlightShortcut(document.activeElement)
    const cancel = () => {
      held.clear()
      if (drag && canvas.hasPointerCapture(drag.id)) canvas.releasePointerCapture(drag.id)
      drag = null
    }
    const interrupt = () => { cancel(); view.cancelLookAt() }
    const keydown = (event: KeyboardEvent) => {
      const axis = panAxisForCode(event.code)
      if (!permitted() || !acceptsFlightShortcut(event.target)) { cancel(); return }
      if (event.altKey || event.ctrlKey || event.metaKey) { cancel(); return }
      if (axis) { view.cancelLookAt(); event.preventDefault(); held.press(event.code, axis) }
    }
    const keyup = (event: KeyboardEvent) => { held.release(event.code) }
    const down = (event: PointerEvent) => {
      if (event.button !== 0 || !event.isPrimary || document.hidden || drag) return
      // Clicking the playfield explicitly leaves the focused control, including
      // Reset View. Wheel/keyboard alone never steal focus from forms.
      canvas.focus({ preventScroll: true })
      if (!permitted()) return
      view.cancelLookAt()
      drag = { id: event.pointerId, x: event.clientX, y: event.clientY }
      canvas.setPointerCapture(event.pointerId)
      event.preventDefault()
    }
    const move = (event: PointerEvent) => {
      if (!drag || event.pointerId !== drag.id) return
      if (!permitted() || !(event.buttons & 1)) { cancel(); return }
      const height = Math.max(1, canvas.clientHeight)
      view.command({ type: 'look', right: -(event.clientX - drag.x) / height,
        up: (event.clientY - drag.y) / height })
      drag.x = event.clientX; drag.y = event.clientY
    }
    const wheel = (event: Event) => {
      if (!(event instanceof WheelEvent)) return
      if (event.target !== canvas && !(event.target instanceof Element && event.target.closest('[data-sensor-marker]'))) return
      if (event.ctrlKey || event.metaKey) return
      // The full-viewport playfield has no scroll action. Consume its wheel
      // even while a form owns focus, while keeping observation unchanged.
      event.preventDefault()
      if (!permitted()) return
      view.command({ type: 'zoom', pixels: event.deltaY *
        (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.clientHeight : 1) })
    }
    window.addEventListener('keydown', keydown)
    window.addEventListener('keyup', keyup)
    window.addEventListener('blur', interrupt)
    window.addEventListener('resize', interrupt)
    document.addEventListener('visibilitychange', interrupt)
    document.addEventListener('focusin', cancel)
    canvas.addEventListener('pointerdown', down)
    canvas.addEventListener('pointermove', move)
    canvas.addEventListener('pointerup', cancel)
    canvas.addEventListener('pointercancel', cancel)
    canvas.addEventListener('lostpointercapture', cancel)
    wheelSurface.addEventListener('wheel', wheel, { passive: false })
    return () => {
      interrupt(); canvas.style.touchAction = previousTouchAction
      if (previousTabIndex === null) canvas.removeAttribute('tabindex')
      else canvas.setAttribute('tabindex', previousTabIndex)
      window.removeEventListener('keydown', keydown)
      window.removeEventListener('keyup', keyup)
      window.removeEventListener('blur', interrupt)
      window.removeEventListener('resize', interrupt)
      document.removeEventListener('visibilitychange', interrupt)
      document.removeEventListener('focusin', cancel)
      canvas.removeEventListener('pointerdown', down)
      canvas.removeEventListener('pointermove', move)
      canvas.removeEventListener('pointerup', cancel)
      canvas.removeEventListener('pointercancel', cancel)
      canvas.removeEventListener('lostpointercapture', cancel)
      wheelSurface.removeEventListener('wheel', wheel)
    }
  }, [gl, held, view])
  useFrame((_, dt) => {
    const keys = held.controls
    if (acceptsFlightShortcut(document.activeElement) && !document.hidden && !paused.current()) {
      const right = Number(keys.right) - Number(keys.left), up = Number(keys.up) - Number(keys.down)
      const step = Math.min(dt, 0.05) * 0.65 / Math.max(1, Math.hypot(right, up))
      view.command({ type: 'look', right: right * step, up: up * step })
    } else held.clear()
    if (document.hidden || paused.current()) view.cancelLookAt()
    else view.advanceLookAt(Math.min(dt, 0.05))
    const snapshot = view.snapshot()
    camera.position.set(0, 0, 0)
    camera.quaternion.fromArray(snapshot.quaternion)
    if (camera.fov !== snapshot.fov) { camera.fov = snapshot.fov; camera.updateProjectionMatrix() }
    camera.updateMatrixWorld()
  }, -2)
  return null
}
