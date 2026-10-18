import type { Attachment } from 'svelte/attachments'

/** How far a finger moves sideways before it's dragging the slider. */
const SLOP = 8

/**
 * Lets a finger scroll the page over a slider without moving it. A press on
 * the track moves the thumb at once, before anyone can tell a drag from the
 * start of a scroll, so a swipe up the page changed the years. Attached to
 * the slider's box, which sees its events first (capture): while a touch is
 * undecided, the slider's changes are kept from the page; a sideways move or
 * a tap hands them over, and the browser's taking the touch for a scroll
 * (`pointercancel`, which `touch-action: pan-y` lets it do) puts the slider
 * back where it was. A mouse and the keyboard are left alone.
 */
export const scrollsPast: Attachment<HTMLElement> = (box) => {
  const slider = box.querySelector<HTMLInputElement>('input[type="range"]')!
  /** Where an undecided touch began, and the slider's value before it. */
  let held: { value: string; x: number; y: number } | null = null

  const handOver = () => {
    held = null
    slider.dispatchEvent(new Event('input', { bubbles: true }))
  }
  const listeners: Record<string, (event: Event) => void> = {
    pointerdown: (event) => {
      const { pointerType, clientX, clientY } = event as PointerEvent
      if (pointerType === 'touch') held = { value: slider.value, x: clientX, y: clientY }
    },
    pointermove: (event) => {
      if (!held) return
      const { clientX, clientY } = event as PointerEvent
      const [dx, dy] = [Math.abs(clientX - held.x), Math.abs(clientY - held.y)]
      if (dx > SLOP && dx > dy) handOver()
    },
    pointerup: () => {
      if (held) handOver()
    },
    pointercancel: () => {
      if (!held) return
      slider.value = held.value
      held = null
    },
    input: (event) => {
      if (held) event.stopImmediatePropagation()
    },
    change: (event) => {
      if (held) event.stopImmediatePropagation()
    },
  }
  for (const [type, listener] of Object.entries(listeners)) box.addEventListener(type, listener, true)
  return () => {
    for (const [type, listener] of Object.entries(listeners)) box.removeEventListener(type, listener, true)
  }
}
