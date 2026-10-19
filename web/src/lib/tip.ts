import { autoUpdate, computePosition, flip, offset, shift, size } from '@floating-ui/dom'
import type { Attachment } from 'svelte/attachments'

/** Closes the open tip, so only one is open at a time. */
let closeOpen: (() => void) | null = null
/** When the last tip closed: straight after one, the next opens at once and
 * without its rise, as the system's own tooltips do. */
let lastClosed = -Infinity
/** How long a tip with a link waits for the pointer to cross to it. */
const LINK_GRACE = 250
const warm = () => performance.now() - lastClosed < 300

/** Shows `popover` (a `popover="manual"` element) next to the element this is
 * attached to: if `hover`, while the mouse is over it or it has keyboard
 * focus, and, if `onClick`, from a click or tap until the next tap elsewhere
 * or Esc. Manual rather than auto popovers, because an auto one closes when
 * its own trigger is pressed, and the click then reopens it.
 *
 * `below` places it under the element rather than above, as menus are.
 * `closeOnClick` closes a hover tip when its button is pressed: a preset's
 * description would otherwise cover the results the press just changed.
 * `keepClear` is the group the trigger belongs to, which the tip must not
 * cover: it is placed against the group's top or bottom edge, at the
 * trigger's own place along it.
 * `onClose` is called when the user closes it (not when it's removed). */
export const tip =
  (
    popover: HTMLElement,
    {
      onClick = true,
      hover = true,
      below = false,
      closeOnClick = false,
      keepClear,
      onClose,
    }: {
      onClick?: boolean
      hover?: boolean
      below?: boolean
      closeOnClick?: boolean
      onClose?: () => void
      /** The trigger's neighbours (the group of buttons it is one of): the
       * tip stands above or below the whole group, never over a sibling. */
      keepClear?: HTMLElement
    } = {},
  ): Attachment<HTMLElement> =>
  (trigger) => {
    let stopPositioning: (() => void) | null = null
    /** Opened by a click, so leaving with the mouse doesn't close it. */
    let clicked = false
    let hoverTimer: ReturnType<typeof setTimeout> | undefined

    // The trigger's width, but the group's height: placing the tip "above"
    // this stands it above every button of the group, not just this one.
    const reference = {
      getBoundingClientRect: () => {
        const own = trigger.getBoundingClientRect()
        if (!keepClear) return own
        const group = keepClear.getBoundingClientRect()
        return new DOMRect(own.x, group.y, own.width, group.height)
      },
      contextElement: trigger,
    }

    const place = () =>
      computePosition(reference, popover, {
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
      }).then(({ x, y, placement }) => {
        popover.style.left = `${x}px`
        popover.style.top = `${y}px`
        // It rises towards the trigger (see `.popover` in app.css).
        popover.style.setProperty('--rise', placement.startsWith('top') ? '4px' : '-4px')
      })

    function close({ byUser = true } = {}) {
      clearTimeout(hoverTimer)
      if (!stopPositioning) return
      stopPositioning()
      stopPositioning = null
      clicked = false
      popover.classList.remove('open')
      popover.hidePopover()
      lastClosed = performance.now()
      if (onClick) trigger.setAttribute('aria-expanded', 'false')
      document.removeEventListener('pointerdown', onPointerDownElsewhere, true)
      document.removeEventListener('keydown', onKeyDown)
      popover.removeEventListener('pointerenter', onPopoverEnter)
      popover.removeEventListener('pointerleave', onPopoverLeave)
      popover.removeEventListener('click', onPopoverClick)
      popover.removeEventListener('focusout', onPopoverFocusOut)
      if (closeOpen === close) closeOpen = null
      if (byUser) onClose?.()
    }

    function open() {
      if (stopPositioning) return
      if (closeOpen) closeOpen()
      closeOpen = close
      // `warm` and `open` drive the fade in app.css.
      popover.classList.toggle('warm', warm())
      popover.showPopover()
      popover.classList.add('open')
      if (onClick) trigger.setAttribute('aria-expanded', 'true')
      stopPositioning = autoUpdate(reference, popover, place)
      document.addEventListener('pointerdown', onPointerDownElsewhere, true)
      document.addEventListener('keydown', onKeyDown)
      popover.addEventListener('pointerenter', onPopoverEnter)
      popover.addEventListener('pointerleave', onPopoverLeave)
      popover.addEventListener('click', onPopoverClick)
      popover.addEventListener('focusout', onPopoverFocusOut)
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
      hoverTimer = setTimeout(open, warm() ? 0 : 150)
    }
    // A tip with a link ("עוד על זה ›", `.more`) must be reachable: leaving
    // the trigger for it mustn't close it, so it waits a moment for the
    // pointer to arrive, and closes when the pointer leaves it.
    const hasLink = () => popover.querySelector('.more') !== null
    function onPointerLeave(event: PointerEvent) {
      if (event.pointerType !== 'mouse') return
      clearTimeout(hoverTimer)
      if (clicked) return
      if (hasLink()) hoverTimer = setTimeout(() => close(), LINK_GRACE)
      else close()
    }
    function onPopoverEnter(event: PointerEvent) {
      if (event.pointerType === 'mouse') clearTimeout(hoverTimer)
    }
    function onPopoverLeave(event: PointerEvent) {
      if (event.pointerType === 'mouse' && !clicked && hasLink())
        hoverTimer = setTimeout(() => close(), LINK_GRACE)
    }
    function onPopoverClick(event: Event) {
      if (event.target instanceof Element && event.target.closest('.more')) close()
    }
    function onClickTrigger() {
      if (clicked) return close()
      open()
      clicked = true
    }
    function onClickToClose() {
      close()
    }
    function onFocus() {
      if (hover && trigger.matches(':focus-visible, :has(:focus-visible)')) open()
    }
    function onBlur(event: FocusEvent) {
      // Tabbing on to the tip's own link keeps it open.
      if (event.relatedTarget instanceof Node && popover.contains(event.relatedTarget)) return
      if (!clicked) close()
    }
    function onPopoverFocusOut(event: FocusEvent) {
      if (
        event.relatedTarget instanceof Node &&
        (popover.contains(event.relatedTarget) || trigger.contains(event.relatedTarget))
      )
        return
      if (!clicked) close()
    }

    if (onClick) trigger.setAttribute('aria-expanded', 'false')
    trigger.addEventListener('pointerenter', onPointerEnter)
    trigger.addEventListener('pointerleave', onPointerLeave)
    trigger.addEventListener('focusin', onFocus)
    trigger.addEventListener('focusout', onBlur)
    if (onClick) trigger.addEventListener('click', onClickTrigger)
    else if (closeOnClick) trigger.addEventListener('click', onClickToClose)
    return () => {
      close({ byUser: false })
      trigger.removeEventListener('pointerenter', onPointerEnter)
      trigger.removeEventListener('pointerleave', onPointerLeave)
      trigger.removeEventListener('focusin', onFocus)
      trigger.removeEventListener('focusout', onBlur)
      trigger.removeEventListener('click', onClickTrigger)
      trigger.removeEventListener('click', onClickToClose)
    }
  }
