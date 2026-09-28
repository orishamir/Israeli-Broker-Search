import { autoUpdate, computePosition, flip, offset, shift, size } from '@floating-ui/dom'
import type { Attachment } from 'svelte/attachments'

/** Closes the open tip, so only one is open at a time. */
let closeOpen: (() => void) | null = null

/** Shows `popover` (a `popover="manual"` element) next to the element this is
 * attached to: if `hover`, while the mouse is over it or it has keyboard
 * focus, and, if `onClick`, from a click or tap until the next tap elsewhere
 * or Esc. Manual rather than auto popovers, because an auto one closes when
 * its own trigger is pressed, and the click then reopens it.
 *
 * `below` places it under the element rather than above, as menus are.
 * `onClose` is called when the user closes it (not when it's removed). */
export const tip =
  (
    popover: HTMLElement,
    {
      onClick = true,
      hover = true,
      below = false,
      onClose,
    }: { onClick?: boolean; hover?: boolean; below?: boolean; onClose?: () => void } = {},
  ): Attachment<HTMLElement> =>
  (trigger) => {
    let stopPositioning: (() => void) | null = null
    /** Opened by a click, so leaving with the mouse doesn't close it. */
    let clicked = false
    let hoverTimer: ReturnType<typeof setTimeout> | undefined

    const place = () =>
      computePosition(trigger, popover, {
        strategy: 'fixed',
        placement: below ? 'bottom-start' : 'top',
        middleware: [
          offset(8),
          flip(),
          shift({ padding: 8 }),
          // No taller than the room left, so a long tip scrolls rather than
          // running off the screen.
          size({
            padding: 8,
            apply: ({ availableHeight }) => {
              popover.style.maxHeight = `${Math.max(availableHeight, 120)}px`
            },
          }),
        ],
      }).then(({ x, y }) => {
        popover.style.left = `${x}px`
        popover.style.top = `${y}px`
      })

    function close({ byUser = true } = {}) {
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
      if (byUser) onClose?.()
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
      if (event.key !== 'Escape') return
      // Only the popover: in a dialog, Esc would close the dialog too.
      event.preventDefault()
      close()
    }

    function onPointerEnter(event: PointerEvent) {
      if (!hover || event.pointerType !== 'mouse') return
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
      if (hover && trigger.matches(':focus-visible, :has(:focus-visible)')) open()
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
      close({ byUser: false })
      trigger.removeEventListener('pointerenter', onPointerEnter)
      trigger.removeEventListener('pointerleave', onPointerLeave)
      trigger.removeEventListener('focusin', onFocus)
      trigger.removeEventListener('focusout', onBlur)
      trigger.removeEventListener('click', onClickTrigger)
    }
  }
