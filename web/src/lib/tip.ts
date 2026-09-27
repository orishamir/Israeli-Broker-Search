import { autoUpdate, computePosition, flip, offset, shift } from '@floating-ui/dom'
import type { Attachment } from 'svelte/attachments'

/** Closes the open tip, so only one is open at a time. */
let closeOpen: (() => void) | null = null

/** Shows `popover` (a `popover="manual"` element) next to the element this is
 * attached to: while the mouse is over it or it has keyboard focus, and, if
 * `onClick`, from a click or tap until the next tap elsewhere or Esc. Manual
 * rather than auto popovers, because an auto one closes when its own trigger
 * is pressed, and the click then reopens it. */
export const tip =
  (popover: HTMLElement, { onClick = true } = {}): Attachment<HTMLElement> =>
  (trigger) => {
    let stopPositioning: (() => void) | null = null
    /** Opened by a click, so leaving with the mouse doesn't close it. */
    let clicked = false
    let hoverTimer: ReturnType<typeof setTimeout> | undefined

    const place = () =>
      computePosition(trigger, popover, {
        strategy: 'fixed',
        placement: 'top',
        middleware: [offset(8), flip(), shift({ padding: 8 })],
      }).then(({ x, y }) => {
        popover.style.left = `${x}px`
        popover.style.top = `${y}px`
      })

    function close() {
      clearTimeout(hoverTimer)
      if (!stopPositioning) return
      stopPositioning()
      stopPositioning = null
      clicked = false
      popover.hidePopover()
      if (onClick) trigger.setAttribute('aria-expanded', 'false')
      document.removeEventListener('pointerdown', onPointerDownElsewhere, true)
      document.removeEventListener('keydown', onKeyDown)
      if (closeOpen === close) closeOpen = null
    }

    function open() {
      if (stopPositioning) return
      if (closeOpen) closeOpen()
      closeOpen = close
      popover.showPopover()
      if (onClick) trigger.setAttribute('aria-expanded', 'true')
      stopPositioning = autoUpdate(trigger, popover, place)
      document.addEventListener('pointerdown', onPointerDownElsewhere, true)
      document.addEventListener('keydown', onKeyDown)
    }

    function onPointerDownElsewhere(event: PointerEvent) {
      const target = event.target as Node
      if (!trigger.contains(target) && !popover.contains(target)) close()
    }
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') close()
    }

    function onPointerEnter(event: PointerEvent) {
      if (event.pointerType !== 'mouse') return
      hoverTimer = setTimeout(open, 150)
    }
    function onPointerLeave(event: PointerEvent) {
      if (event.pointerType !== 'mouse') return
      clearTimeout(hoverTimer)
      if (!clicked) close()
    }
    function onClickTrigger() {
      if (clicked) return close()
      open()
      clicked = true
    }
    function onFocus() {
      if (trigger.matches(':focus-visible, :has(:focus-visible)')) open()
    }
    function onBlur() {
      if (!clicked) close()
    }

    if (onClick) trigger.setAttribute('aria-expanded', 'false')
    trigger.addEventListener('pointerenter', onPointerEnter)
    trigger.addEventListener('pointerleave', onPointerLeave)
    trigger.addEventListener('focusin', onFocus)
    trigger.addEventListener('focusout', onBlur)
    if (onClick) trigger.addEventListener('click', onClickTrigger)
    return () => {
      close()
      trigger.removeEventListener('pointerenter', onPointerEnter)
      trigger.removeEventListener('pointerleave', onPointerLeave)
      trigger.removeEventListener('focusin', onFocus)
      trigger.removeEventListener('focusout', onBlur)
      trigger.removeEventListener('click', onClickTrigger)
    }
  }
