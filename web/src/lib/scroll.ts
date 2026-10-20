// Where the page scrolls to when what the visitor did changed something they
// can't see: a situation chip on a phone changes the answer under the
// inputs; a decision can open another one (REDESIGN.md principle 8).
import { SETTLE } from './motion'

/** The page is one column: the answer is under the inputs (App.svelte). */
export const oneColumn = (): boolean => matchMedia('(width < 800px)').matches

const behavior = (): ScrollBehavior =>
  matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'

/** The answer card's top, at the top of the screen. */
export function showAnswer() {
  document.querySelector('section.answer')?.scrollIntoView({ block: 'start', behavior: behavior() })
}

/** The decisions in the answer card that wait for the visitor, by line. */
const openDecisions = (): Set<string> =>
  new Set(
    [...document.querySelectorAll<HTMLElement>('section.answer .line.asks')]
      .filter((line) => !line.querySelector('.choose[aria-pressed="true"]'))
      .map((line) => line.dataset.line ?? ''),
  )

/** Room kept above what is scrolled to, under the screen's top edge. */
const MARGIN = 12

/**
 * Keeps the option just chosen where it was on the screen while the answer
 * changes around it (lines above it may come or go, a phone has no scroll
 * anchoring in Safari), then, if the choice opened a decision the visitor
 * can't see whole, brings it gently into view: its question and its
 * options, or as much of them as the screen holds. Stops as soon as the
 * visitor scrolls.
 *
 * `find` finds the option again (the answer may draw it anew); `fresh`
 * says whether the answer is worked out for the latest inputs (a worker
 * may still be at it): drawn, with the option pressed, the new one is in.
 */
export function holdChosen(find: () => Element | null, fresh: () => boolean) {
  const first = find()
  if (!first) return
  const top = first.getBoundingClientRect().top
  const before = openDecisions()
  const began = performance.now()
  let changedAt: number | undefined
  let stopped = false

  const stop = () => {
    stopped = true
    removeEventListener('wheel', stop)
    removeEventListener('touchstart', stop)
    removeEventListener('keydown', stop)
  }
  addEventListener('wheel', stop, { passive: true })
  addEventListener('touchstart', stop, { passive: true })
  addEventListener('keydown', stop)

  const revealNew = () => {
    const opened = [...openDecisions()].find((line) => !before.has(line))
    const group = opened && document.querySelector(`section.answer [data-line="${opened}"]`)
    const question = group && group.querySelector('.question')
    if (!group || !question) return
    const asked = question.getBoundingClientRect().top
    const bottom = group.getBoundingClientRect().bottom
    if (asked < MARGIN) {
      // Opened above (it depends on nothing below it): go up to it.
      scrollBy({ top: asked - MARGIN * 4, behavior: behavior() })
    } else if (bottom > innerHeight - MARGIN) {
      // Below the screen's end: the question alone isn't the decision, its
      // options are (the review's N8). Up until the whole decision is on
      // the screen, or, where it's taller than the screen (stacked options
      // on a phone), until its question is near the top. What was chosen is
      // above it, and may go off the top: the new question wins.
      const by = Math.min(bottom - (innerHeight - MARGIN), asked - MARGIN * 4)
      if (by > 0) scrollBy({ top: by, behavior: behavior() })
    }
  }

  const frame = () => {
    if (stopped) return
    const now = performance.now()
    const shift = (find()?.getBoundingClientRect().top ?? top) - top
    if (Math.abs(shift) >= 1) scrollBy({ top: shift, behavior: 'instant' })
    const pressed = find()?.querySelector('.choose[aria-pressed="true"]')
    if (changedAt === undefined && pressed && fresh()) changedAt = now
    // The card glides to its new height for SETTLE: hold until it's there.
    if (changedAt !== undefined && now - changedAt > SETTLE + 50) {
      stop()
      revealNew()
      return
    }
    if (now - began > 3000) return stop()
    requestAnimationFrame(frame)
  }
  requestAnimationFrame(frame)
}
