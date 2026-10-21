// What the page does with the scroll while the answer changes around what
// the visitor is using: it keeps the pressed control where it was
// (principle 21). The page never scrolls by itself (REDESIGN.md §4.3, and
// principles 21 and 22 at every width): a decision a choice opens out of
// sight is announced by a pill instead (`DecisionPill`), and only what the
// visitor asks for ("לתשובה", the pill) scrolls.
import { SETTLE } from './motion'

/** The site's phone: one column, the answer under the inputs (App.svelte). */
export const PHONE = '(width < 800px)'

/** The page is one column: the answer is under the inputs (App.svelte). */
export const oneColumn = (): boolean => matchMedia(PHONE).matches

const behavior = (): ScrollBehavior =>
  matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'

/** The answer card's top, at the top of the screen. */
export function showAnswer() {
  document.querySelector('section.answer')?.scrollIntoView({ block: 'start', behavior: behavior() })
}

/** The decisions anywhere in the answer card (its lines, or a step of the
 * path) that wait for the visitor, by id (`Decision`'s `data-decision`). */
const waiting = (): Set<string> =>
  new Set(
    [...document.querySelectorAll<HTMLElement>('section.answer [data-decision]')]
      .filter((group) => !group.querySelector('.choose[aria-pressed="true"]'))
      .map((group) => group.dataset.decision ?? ''),
  )

/** Room the phone's bottom dock (the pill, the answer's bar) may cover. */
const DOCK = 96

/** Whether a decision can be seen: some of it is on the screen, above
 * whatever floats at its bottom. */
export function inSight(element: Element): boolean {
  const box = element.getBoundingClientRect()
  return box.top < innerHeight - DOCK && box.bottom > 0
}

/** The decision of `id` in the answer card, if it's there. */
export const decisionElement = (id: string): HTMLElement | null =>
  document.querySelector<HTMLElement>(`section.answer [data-decision="${id}"]`)

/** On a phone: the first decision that waits now and didn't before, if
 * it's out of sight, for the pill. */
function announceNew(before: Set<string>, opened: ((id: string) => void) | undefined) {
  if (!opened) return
  const id = [...waiting()].find((each) => !before.has(each))
  const group = id && decisionElement(id)
  if (id && group && !inSight(group)) opened(id)
}

/** Takes the visitor to a decision, when they ask (the pill): its question
 * at the top, under a little room. */
export function showDecision(id: string) {
  const group = decisionElement(id)
  // Its question is the line's heading, or the step's, just above it.
  const shown = group?.closest('.line, .step') ?? group
  shown?.scrollIntoView({ block: 'start', behavior: behavior() })
}

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
export function holdChosen(find: () => Element | null, fresh: () => boolean, opened?: (id: string) => void) {
  const first = find()
  if (!first) return
  const top = first.getBoundingClientRect().top
  const waitingBefore = waiting()
  const began = performance.now()
  let changedAt: number | undefined
  let stopped = false

  const stop = () => {
    stopped = true
    removeEventListener('wheel', stop)
    removeEventListener('touchstart', stop)
    removeEventListener('keydown', stop)
    removeEventListener('pointerdown', stop)
  }
  addEventListener('wheel', stop, { passive: true })
  addEventListener('touchstart', stop, { passive: true })
  addEventListener('keydown', stop)
  // The next thing the visitor presses is theirs to keep still.
  addEventListener('pointerdown', stop)

  const frame = () => {
    if (stopped) return
    const now = performance.now()
    const shift = (find()?.getBoundingClientRect().top ?? top) - top
    if (Math.abs(shift) >= 1) scrollBy({ top: shift, behavior: 'instant' })
    // In: the option drawn pressed, or, where the choice made the plan, the
    // path's step that stands for it now (`find` falls back to it).
    const found = find()
    const pressed = found?.matches('[data-step]') || found?.querySelector('.choose[aria-pressed="true"]')
    if (changedAt === undefined && pressed && fresh()) changedAt = now
    // The card glides to its new height for SETTLE: hold until it's there.
    if (changedAt !== undefined && now - changedAt > SETTLE + 50) {
      stop()
      // The page never scrolls by itself: a sign in sight instead.
      announceNew(waitingBefore, opened)
      return
    }
    if (now - began > 3000) return stop()
    requestAnimationFrame(frame)
  }
  requestAnimationFrame(frame)
}

/**
 * Keeps what the visitor is using (a step of the path) where it is on the
 * screen while what's above it changes (the plan's sentence and amount),
 * until the answer is in and the card has glided to its height (principle
 * 21). Nothing is brought into view. Stops as soon as the visitor scrolls.
 */
export function holdStill(find: () => Element | null, fresh: () => boolean, opened?: (id: string) => void) {
  const first = find()
  if (!first) return
  const top = first.getBoundingClientRect().top
  const waitingBefore = waiting()
  const began = performance.now()
  let freshAt: number | undefined
  let stopped = false
  const stop = () => {
    stopped = true
    removeEventListener('wheel', stop)
    removeEventListener('touchstart', stop)
    removeEventListener('keydown', stop)
    removeEventListener('pointerdown', stop)
  }
  addEventListener('wheel', stop, { passive: true })
  addEventListener('touchstart', stop, { passive: true })
  addEventListener('keydown', stop)
  // The next thing the visitor presses is theirs to keep still.
  addEventListener('pointerdown', stop)
  const frame = () => {
    if (stopped) return
    const now = performance.now()
    const shift = (find()?.getBoundingClientRect().top ?? top) - top
    if (Math.abs(shift) >= 1) scrollBy({ top: shift, behavior: 'instant' })
    if (freshAt === undefined && fresh()) freshAt = now
    if (freshAt !== undefined && now - freshAt > SETTLE + 50) {
      stop()
      announceNew(waitingBefore, opened)
      return
    }
    if (now - began > 3000) return stop()
    requestAnimationFrame(frame)
  }
  requestAnimationFrame(frame)
}
