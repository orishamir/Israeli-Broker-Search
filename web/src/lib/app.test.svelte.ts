import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState, planId, type Details } from './app.svelte'
import * as core from './core/core'
import { decode, encode } from './link'

// Fixed rates instead of today's, so nothing depends on the network or the day.
beforeEach(() => {
  localStorage.clear()
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})

/** An app, as App.svelte makes one: its effects need a root to live in. */
function start(shared = {}): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState(shared)
  })
  flushSync()
  return app
}

/** What the core answers for the answers, choices and settings as they are now. */
const coreSays = (app: AppState) => core.answer(app.answerRequest)

const draftOf = (details: Details | null) => {
  if (details?.kind !== 'draft') throw new Error('no draft open')
  return details.draft
}

test('the guide opens on its first situation, and its answer is the core’s, without the horizons', () => {
  const app = start()
  expect(app.situationId).toBe('goal')
  expect(app.answer).toEqual(coreSays(app))
  expect(app.answer?.horizons).toEqual([])
  expect(app.answerFresh).toBe(true)
  expect(app.inputError).toBeUndefined()
  // The workbench's guide is worked out only with the switch on.
  expect(app.recommendation).toBeUndefined()
})

test.each(['goal', 'cushion', 'salary', 'retirement'])(
  'the situation %s: the answer is the core’s, short horizons on the page and long ones from the worker',
  async (id) => {
    const app = start()
    app.setSituation(id)
    flushSync()
    // A long horizon is worked out off the page's thread: the last answer
    // stays until the new one arrives.
    await vi.waitFor(() => expect(app.answerFresh).toBe(true))
    expect(app.answer).toEqual(coreSays(app))
  },
)

test('the horizons are a job of their own, after the answer, and keep the last rows until the next', async () => {
  const app = start()
  await vi.waitFor(() => expect(app.horizonsFresh).toBe(true))
  expect(app.horizons).toEqual(core.answer(app.horizonsRequest).horizons)
  expect(app.horizons?.length).toBeGreaterThan(0)
  const before = app.horizons
  app.answers.now = (app.answers.now ?? 0) + 1000
  flushSync()
  // The answer is there at once (a short horizon); the rows are the last ones, not fresh.
  expect(app.answerFresh).toBe(true)
  expect(app.horizonsFresh).toBe(false)
  expect(app.horizons).toBe(before)
  await vi.waitFor(() => expect(app.horizonsFresh).toBe(true))
  expect(app.horizons).not.toBe(before)
})

test('the switch changes what is shown and nothing the answer is asked', () => {
  const app = start()
  const request = $state.snapshot(app.answerRequest)
  const answer = app.answer
  app.expert = true
  flushSync()
  expect($state.snapshot(app.answerRequest)).toEqual(request)
  expect(app.answer).toEqual(answer)
  expect(app.recommendation).toBeDefined()
  app.expert = false
  flushSync()
  expect(app.answer).toEqual(answer)
})

test('the switch is remembered, and a chip for experts turns it on', () => {
  const app = start()
  app.expert = true
  flushSync()
  expect(localStorage.getItem('expert-v2')).toBe('true')
  expect(start().expert).toBe(true)
  localStorage.clear()
  const other = start()
  expect(other.expert).toBe(false)
  const workbench = other.situations.find(({ workbench }) => workbench)
  if (!workbench) throw new Error('no situation opens the workbench')
  other.setSituation(workbench.id)
  expect(other.expert).toBe(true)
})

test('answers that can’t be compared keep the last answer, with the core’s reason', () => {
  const app = start()
  const last = app.answer
  app.answers.horizon = { Months: 0 }
  flushSync()
  // The core says why, or (if zero months is an answer) there's no error and
  // the answer is simply the core's.
  if (app.inputError !== undefined) {
    expect(app.stale).toBe(true)
    expect(app.answer).toEqual(last)
  } else {
    expect(app.answer).toEqual(coreSays(app))
  }
  app.answers.horizon = { Months: 60 }
  app.answers.now = null
  app.answers.monthly = null
  flushSync()
  expect(app.answer).toBeDefined()
})

test('the workbench follows the answer until the user chooses another thing, and a chip starts over', () => {
  const app = start()
  app.expert = true
  flushSync()
  // Held to the kind the answer is about, if it is about one.
  const focus = app.answer?.focus
  if (focus) expect(app.overrides.kind).toEqual(focus)
  else expect(app.overrides).toEqual({ arena: undefined, kind: undefined })
  const kind = app.overrides.kind ?? app.recommendation!.kind.kind
  const followed =
    typeof kind === 'object' && 'Way' in kind ? 'sp500' : typeof kind === 'object' ? 'bonds' : 'short'
  expect(app.thing).toBe(followed)
  app.chooseThing('other')
  expect(app.thing).toBe('other')
  flushSync()
  expect(app.comparison && 'results' in app.comparison).toBe(true)
  app.setSituation('goal')
  expect(app.thing).toBe(followed)
})

test('the ways: all but the ones asked for are ticked at first, and the last one can’t be unticked', () => {
  const app = start()
  const first = app.selectedWays
  expect(first.length).toBeGreaterThan(0)
  for (const way of first.slice(1)) app.toggleWay(way)
  expect(app.selectedWays).toHaveLength(1)
  app.toggleWay(app.selectedWays[0])
  expect(app.selectedWays).toHaveLength(1)
})

test('a link carries the answers, the switch and what the workbench looks at back into a new page', () => {
  const app = start()
  app.setSituation('retirement')
  app.expert = true
  app.chooseThing('bonds')
  app.answers.now = 12_345
  flushSync()
  const link = app.shareLink()
  const hash = link.slice(link.indexOf('#') + 1)
  const again = start(decode(hash))
  expect(again.answers).toEqual($state.snapshot(app.answers))
  expect(again.situationId).toBe('retirement')
  expect(again.expert).toBe(true)
  expect(again.thing).toBe('bonds')
  expect(again.overrides).toEqual(app.overrides)
  expect([...again.selected].sort()).toEqual([...app.selected].sort())
})

test('the guide carries assumptions that are not the usual ones in its address, and only those', () => {
  const app = start()
  expect(decode(app.address())).not.toHaveProperty('yearlyReturnPercent')
  app.yearlyReturnPercent = 7
  app.boiRatePercent = 2.5
  flushSync()
  expect(app.expert).toBe(false)
  expect(decode(app.address())).toMatchObject({ yearlyReturnPercent: 7, boiRatePercent: 2.5 })
  expect(decode(app.address())).not.toHaveProperty('inflationPercent')
  expect(start(decode(app.address())).yearlyReturnPercent).toBe(7)
})

test('the first visit is an example until anything changes; a link is not one', () => {
  const app = start()
  expect(app.exampleShown).toBe(true)
  app.answers.now = 1234
  flushSync()
  expect(app.exampleShown).toBe(false)
  expect(start(decode(app.address())).exampleShown).toBe(false)
})

test('restoring an address puts back the goal, the route and the assumptions, and an empty one the example', () => {
  const app = start()
  const first = $state.snapshot(app.answers)
  app.setSituation('retirement')
  app.expert = true
  app.yearlyReturnPercent = 6
  flushSync()
  const there = decode(app.address())
  app.restore({})
  flushSync()
  expect(app.situationId).toBe('goal')
  expect(app.expert).toBe(false)
  expect(app.yearlyReturnPercent).toBe(10)
  expect(app.answers).toEqual(first)
  expect(app.exampleShown).toBe(true)
  app.restore(there)
  flushSync()
  expect(app.situationId).toBe('retirement')
  expect(app.expert).toBe(true)
  expect(app.yearlyReturnPercent).toBe(6)
  expect(app.exampleShown).toBe(false)
})

test('a link without v=2 opens the page as usual', () => {
  const app = start(decode('s=Etf&d=99999&m=1&y=7'))
  expect(app.answers.now).not.toBe(99_999)
  expect(encode({})).toBe('v=2')
})

test('your plans are kept in this browser, ticked when added, and leave with their tick', () => {
  const app = start()
  app.draftNewPlan()
  const draft = draftOf(app.details)
  app.addYourPlan(draft)
  flushSync()
  const id = planId({ kind: 'yours', id: draft.id })
  expect(app.selected.has(id)).toBe(true)
  expect(JSON.parse(localStorage.getItem('your-plans-v1')!)).toHaveLength(1)
  expect(app.settings.yourPlans).toHaveLength(1)
  app.deleteYourPlan(draft.id)
  flushSync()
  expect(app.selected.has(id)).toBe(false)
  expect(JSON.parse(localStorage.getItem('your-plans-v1')!)).toHaveLength(0)
})

test('learn cards stack, go back, and close', () => {
  const app = start()
  expect(app.cardTerm).toBeUndefined()
  app.openCard('etf')
  app.openCard('indexFund')
  expect(app.cardTerm).toBe('indexFund')
  expect(app.cardFor('etf').term).toBe('etf')
  app.cardBack()
  expect(app.cardTerm).toBe('etf')
  app.closeCard()
  expect(app.cardTerm).toBeUndefined()
  expect(app.glossary.length).toBeGreaterThan(0)
})

// A retirement is said by the two ages alone (REDESIGN.md §4.2): the date's
// field isn't on screen, and the years are the gap between them.
test('a retirement is the two ages, and the years are the gap between them', async () => {
  const app = start()
  app.setSituation('retirement')
  app.setHorizonRetirement(45, 63)
  flushSync()
  await vi.waitFor(() => expect(app.answerFresh).toBe(true))
  expect(app.answers.horizon).toEqual({ Retirement: { age: 45, retire_at: 63 } })
  expect(app.horizonMonths).toBe(18 * 12)
  expect(app.answer).toEqual(coreSays(app))
})

test('the answer is the core’s, and a choice re-evaluates it', async () => {
  const app = start()
  await vi.waitFor(() => expect(app.answerFresh).toBe(true))
  expect(app.answer).toEqual(coreSays(app))
  const way = app.answer?.lines.find((line) => line.state.type === 'decision')
  if (way?.state.type !== 'decision') throw new Error('the goal has a decision')
  const option = way.state.decision.options[0]
  app.choose(option.pick)
  flushSync()
  await vi.waitFor(() => expect(app.answerFresh).toBe(true))
  const after = app.answer?.lines.find((line) => line.kind === way.kind)
  expect(after?.state.type === 'decision' && after.state.decision.chosen).toEqual(option.pick)
  // The choice travels in the link, and a chip starts the choices over.
  expect(decode(encode(decode(app.address()))).choices).toEqual(app.choices)
  app.setSituation('salary')
  expect(app.choices).toEqual({})
})

test('the visitor’s bank stays across the chips, and a row tapped is focused', () => {
  const app = start()
  app.setBank('Leumi')
  app.setSituation('bonus')
  flushSync()
  expect(app.answers.bank).toBe('Leumi')
  app.focusRow('MoneyMarketFund')
  expect(app.focusedRow).toBe('MoneyMarketFund')
  app.focusRow(undefined)
  expect(app.focusedRow).toEqual(app.answer?.focus ?? undefined)
})
