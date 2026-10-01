import type { Attachment } from 'svelte/attachments'
import { slide, type TransitionConfig } from 'svelte/transition'
import { createCubicEasingFunc } from 'zrender/lib/animation/cubicEasing.js'

/** Whether the user asked their system for less motion. Animations are then
 * skipped: CSS ones by the rule in app.css, the rest by checking this. */
export const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches

/** An animation's length: `ms`, or none when the user asked for less motion. */
export const duration = (ms: number): number => (reducedMotion ? 0 : ms)

/** How long the results take to move to new numbers: the charts glide, the
 * summary's numbers roll and the table's rows slide for this long, together.
 * `--settle` in app.css. */
export const SETTLE = 400

/** The page's curves, as app.css has them (`--ease-out`, `--ease-in`): what
 * arrives starts fast and settles; what leaves gets out of the way. ECharts
 * reads them as they are. */
export const EASE_OUT = 'cubic-bezier(0.2, 0, 0, 1)'
export const EASE_IN = 'cubic-bezier(0.4, 0, 1, 1)'

/** The same curves as functions, for Svelte's tweens and transitions. They
 * are ECharts' own, so the numbers and the charts move alike. */
export const easeOut = createCubicEasingFunc(EASE_OUT)!
export const easeIn = createCubicEasingFunc(EASE_IN)!

/** For a block that comes and goes (More options' fields, a card): it opens
 * downwards as it fades in, and closes faster. `transition:reveal`. */
export function reveal(
  node: Element,
  _params: undefined,
  { direction }: { direction: 'in' | 'out' | 'both' },
): TransitionConfig {
  const leaving = direction === 'out'
  const opening = slide(node, { duration: duration(leaving ? 160 : 240), easing: leaving ? easeIn : easeOut })
  // Slide's own fade is over in the first twentieth of the way. Its CSS ends
  // without a semicolon.
  return { ...opening, css: (t, u) => `${opening.css!(t, u)}; opacity: ${t}` }
}

/** For a box whose content changes size at once (one description swapped
 * for another, a line coming or going with it): the box glides from its old
 * height to the new one, in one motion, so what's below moves smoothly and
 * one way, for `ms` (as long as what it opens with: 240 for a card's
 * fields, `SETTLE` for the results). Its first child is the content, clipped while the box glides
 * (and only then: a field's focus ring reaches past it). A part that came
 * and went with its own transition inside it would pull the other way, so
 * nothing in it has one. */
export const glideHeight =
  (ms = 240): Attachment<HTMLElement> =>
  (box) => {
    const content = box.firstElementChild as HTMLElement
    const measure = () => content.getBoundingClientRect().height
    let height = measure()
    // The content changed: told before the page is painted again, so the
    // glide starts on the next frame shown, from the height the box had (or
    // where a glide under way has got to). A ResizeObserver is told after,
    // which showed the new height for a frame first.
    const changing = new MutationObserver(() => {
      const next = measure()
      if (Math.abs(next - height) < 0.5) return
      const gliding = box.getAnimations()
      const from = gliding.length > 0 ? box.getBoundingClientRect().height : height
      height = next
      for (const glide of gliding) glide.cancel()
      if (reducedMotion || box.offsetParent === null) return
      box.style.overflow = 'clip'
      const glide = box.animate([{ height: `${from}px` }, { height: `${next}px` }], {
        duration: ms,
        easing: EASE_OUT,
      })
      glide.onfinish = () => box.style.removeProperty('overflow')
    })
    changing.observe(content, { subtree: true, childList: true, characterData: true, attributes: true })
    // Other changes of size (a narrower window, fonts arriving) are only kept
    // track of: nothing in the box changed.
    const resizing = new ResizeObserver(() => (height = measure()))
    resizing.observe(content)
    return () => {
      changing.disconnect()
      resizing.disconnect()
    }
  }
