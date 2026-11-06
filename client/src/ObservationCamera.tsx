import { useEffect, useMemo } from 'react'
import { useFrame, useThree } from '@react-three/fiber'
import { PerspectiveCamera } from 'three'
import { acceptsFlightShortcut, NavigationInput, panAxisForCode } from './input.ts'
import { ObservationCameraState } from './observationCamera.ts'

/** Reusable perspective adapter. The parent owns view lifetime and flight input.
 * Ship-relative scene geometry makes the render camera's zero translation the
 * exact physical ship origin, even beyond Number.MAX_SAFE_INTEGER cells.
 */
export function ObservationCamera({ view }: { view: ObservationCameraState }) {
  const { set, get, gl, size } = useThree()
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
    const previousTouchAction = canvas.style.touchAction
    const previousTabIndex = canvas.getAttribute('tabindex')
    canvas.style.touchAction = 'none'
    canvas.tabIndex = 0
    let drag: { id: number; x: number; y: number } | null = null
    const permitted = () => !document.hidden && acceptsFlightShortcut(document.activeElement)
    const cancel = () => {
      held.clear()
      if (drag && canvas.hasPointerCapture(drag.id)) canvas.releasePointerCapture(drag.id)
      drag = null
    }
    const keydown = (event: KeyboardEvent) => {
      const axis = panAxisForCode(event.code)
      if (!permitted() || !acceptsFlightShortcut(event.target)) { cancel(); return }
      if (event.altKey || event.ctrlKey || event.metaKey) { cancel(); return }
      if (axis) { event.preventDefault(); held.press(event.code, axis) }
    }
    const keyup = (event: KeyboardEvent) => { held.release(event.code) }
    const down = (event: PointerEvent) => {
      if (event.button !== 0 || !event.isPrimary || document.hidden || drag) return
      // Clicking the playfield explicitly leaves the focused control, including
      // Reset View. Wheel/keyboard alone never steal focus from forms.
      canvas.focus({ preventScroll: true })
      if (!permitted()) return
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
    const wheel = (event: WheelEvent) => {
      if (!permitted() || event.ctrlKey || event.metaKey) return
      event.preventDefault()
      view.command({ type: 'zoom', pixels: event.deltaY *
        (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.clientHeight : 1) })
    }
    window.addEventListener('keydown', keydown)
    window.addEventListener('keyup', keyup)
    window.addEventListener('blur', cancel)
    window.addEventListener('resize', cancel)
    document.addEventListener('visibilitychange', cancel)
    document.addEventListener('focusin', cancel)
    canvas.addEventListener('pointerdown', down)
    canvas.addEventListener('pointermove', move)
    canvas.addEventListener('pointerup', cancel)
    canvas.addEventListener('pointercancel', cancel)
    canvas.addEventListener('lostpointercapture', cancel)
    canvas.addEventListener('wheel', wheel, { passive: false })
    return () => {
      cancel(); canvas.style.touchAction = previousTouchAction
      if (previousTabIndex === null) canvas.removeAttribute('tabindex')
      else canvas.setAttribute('tabindex', previousTabIndex)
      window.removeEventListener('keydown', keydown)
      window.removeEventListener('keyup', keyup)
      window.removeEventListener('blur', cancel)
      window.removeEventListener('resize', cancel)
      document.removeEventListener('visibilitychange', cancel)
      document.removeEventListener('focusin', cancel)
      canvas.removeEventListener('pointerdown', down)
      canvas.removeEventListener('pointermove', move)
      canvas.removeEventListener('pointerup', cancel)
      canvas.removeEventListener('pointercancel', cancel)
      canvas.removeEventListener('lostpointercapture', cancel)
      canvas.removeEventListener('wheel', wheel)
    }
  }, [gl, held, view])
  useFrame((_, dt) => {
    const keys = held.controls
    if (acceptsFlightShortcut(document.activeElement) && !document.hidden) {
      const right = Number(keys.right) - Number(keys.left), up = Number(keys.up) - Number(keys.down)
      const step = Math.min(dt, 0.05) * 0.65 / Math.max(1, Math.hypot(right, up))
      view.command({ type: 'look', right: right * step, up: up * step })
    } else held.clear()
    const snapshot = view.snapshot()
    camera.position.set(0, 0, 0)
    camera.quaternion.fromArray(snapshot.quaternion)
    if (camera.fov !== snapshot.fov) { camera.fov = snapshot.fov; camera.updateProjectionMatrix() }
    camera.updateMatrixWorld()
  }, -2)
  return null
}
