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

/** What the core says for the answers and settings as they are now. */
const coreSays = (app: AppState) => core.recommend(app.guideRequest.answers, app.settings)

const draftOf = (details: Details | null) => {
  if (details?.kind !== 'draft') throw new Error('no draft open')
  return details.draft
}

test('the guide opens on its first situation, and what it recommends is the core’s answer', () => {
  const app = start()
  expect(app.situationId).toBe('goal')
  expect(app.recommendation).toEqual(coreSays(app))
  expect(app.recommendationFresh).toBe(true)
  expect(app.inputError).toBeUndefined()
})

test.each(['goal', 'cushion', 'salary', 'retirement'])(
  'the situation %s: the arena and the kind are the core’s, short horizons on the page and long ones from the worker',
  async (id) => {
    const app = start()
    app.setSituation(id)
    flushSync()
    // A long horizon is worked out off the page's thread: the last answer
    // stays until the new one arrives.
    await vi.waitFor(() => expect(app.recommendationFresh).toBe(true))
    const expected = coreSays(app)
    expect(app.recommendation?.arena).toBe(expected.arena)
    expect(app.recommendation?.kind).toEqual(expected.kind)
    expect(app.recommendation).toEqual(expected)
  },
)

test('the lines of the alternatives are plain objects with their kind and name, not maps', () => {
  const app = start()
  const { alternatives } = app.recommendation!
  expect(alternatives.length).toBeGreaterThan(0)
  for (const alternative of alternatives) {
    expect(alternative).not.toBeInstanceOf(Map)
    expect(alternative.kind).toBeDefined()
    expect(alternative.name).toBeTruthy()
  }
  const lines = core.alternatives(app.guideRequest.answers, app.settings).series
  for (const line of lines) expect(line.kind).toBeDefined()
})

test('the ladder and the alternatives come from the worker, and keep the last answer until the next', async () => {
  const app = start()
  await vi.waitFor(() => expect(app.ladderFresh && app.alternativesFresh).toBe(true))
  expect(app.alternatives).toEqual(core.alternatives(app.guideRequest.answers, app.settings))
  expect(app.ladder).toEqual(core.ladder(app.guideRequest.answers, app.settings))
  const before = app.alternatives
  app.answers.now = (app.answers.now ?? 0) + 1000
  flushSync()
  // Between the change and the answer, the last one is still there, not fresh.
  expect(app.alternativesFresh).toBe(false)
  expect(app.alternatives).toBe(before)
  await vi.waitFor(() => expect(app.alternativesFresh).toBe(true))
  expect(app.alternatives).not.toBe(before)
})

test('the switch changes what is shown and nothing the core is asked', () => {
  const app = start()
  const request = $state.snapshot(app.guideRequest)
  const recommendation = app.recommendation
  app.expert = true
  flushSync()
  expect($state.snapshot(app.guideRequest)).toEqual(request)
  expect(app.recommendation).toEqual(recommendation)
  app.expert = false
  flushSync()
  expect(app.recommendation).toEqual(recommendation)
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

test('a kind chosen over the guide is what the recommendation then is, until cleared', () => {
  const app = start()
  const first = app.recommendation!
  const other = first.alternatives.find(({ kind }) => kind !== first.kind.kind && kind !== 'CurrentAccount')
  if (!other) throw new Error('no other kind to choose')
  app.chooseKind(other.kind)
  flushSync()
  expect(app.overridden).toBe(true)
  expect(app.recommendation?.kind.kind).toEqual(other.kind)
  expect(app.recommendation).toEqual(coreSays(app))
  app.clearOverrides()
  flushSync()
  expect(app.overridden).toBe(false)
  expect(app.recommendation?.kind.kind).toEqual(first.kind.kind)
  // A new situation starts over, too.
  app.chooseKind(other.kind)
  app.setSituation('goal')
  expect(app.overridden).toBe(false)
})

test('answers that can’t be compared keep the last recommendation, with the core’s reason', () => {
  const app = start()
  const last = app.recommendation
  app.answers.horizon = { Months: 0 }
  flushSync()
  // The core says why, or (if zero months is an answer) there's no error and
  // the recommendation is simply the core's.
  if (app.inputError !== undefined) {
    expect(app.stale).toBe(true)
    expect(app.recommendation).toEqual(last)
  } else {
    expect(app.recommendation).toEqual(coreSays(app))
  }
  app.answers.horizon = { Months: 60 }
  app.answers.now = null
  app.answers.monthly = null
  flushSync()
  expect(app.recommendation).toBeDefined()
})

test('the workbench follows the recommendation until the user chooses another thing', () => {
  const app = start()
  const kind = app.recommendation!.kind.kind
  const followed =
    typeof kind === 'object' && 'Way' in kind ? 'sp500' : typeof kind === 'object' ? 'bonds' : 'short'
  expect(app.thing).toBe(followed)
  app.chooseThing('other')
  expect(app.thing).toBe('other')
  flushSync()
  expect(app.comparison && 'results' in app.comparison).toBe(true)
  app.clearOverrides()
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

test('a link carries the answers, the switch and the overrides back into a new page', () => {
  const app = start()
  app.setSituation('retirement')
  app.expert = true
  app.chooseKind({ GovernmentBond: { linked: false } })
  app.answers.now = 12_345
  flushSync()
  const link = app.shareLink()
  const hash = link.slice(link.indexOf('#') + 1)
  const again = start(decode(hash))
  expect(again.answers).toEqual($state.snapshot(app.answers))
  expect(again.situationId).toBe('retirement')
  expect(again.expert).toBe(true)
  expect(again.overrides).toEqual($state.snapshot(app.overrides))
  expect([...again.selected].sort()).toEqual([...app.selected].sort())
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

test('typing years for a retirement keeps the ages: the years are the gap between them', async () => {
  const app = start()
  app.setSituation('retirement')
  await vi.waitFor(() => expect(app.followUps).toContain('AgeAndRetirement'))
  app.setHorizonMonths(18 * 12)
  flushSync()
  await vi.waitFor(() => expect(app.recommendationFresh).toBe(true))
  expect(app.answers.horizon).toEqual({ Retirement: { age: 45, retire_at: 63 } })
  expect(app.horizonMonths).toBe(18 * 12)
  expect(app.followUps).toContain('AgeAndRetirement')
  expect(JSON.stringify(app.recommendation?.kind)).toContain('InvestmentGemel')
})

test('a pick is dropped when the goal changes, and the ladder shows what fits each horizon', async () => {
  const app = start()
  app.setSituation('bonus')
  flushSync()
  const stock = app.recommendation!.alternatives.find(({ kind }) => typeof kind === 'object' && 'Way' in kind)
  if (!stock) throw new Error('no way to pick')
  app.chooseKind(stock.kind)
  await vi.waitFor(() => expect(app.ladderFresh).toBe(true))
  // The year's rung is a safe kind's, whatever was picked in the index.
  expect(app.ladder![0].arena).toBe('Safe')
  expect(app.ladder!.at(-1)!.kind.kind).toEqual(stock.kind)
  app.setSituation('child')
  flushSync()
  expect(app.overridden).toBe(false)
  await vi.waitFor(() => expect(app.recommendationFresh).toBe(true))
  expect(app.recommendation).toEqual(coreSays(app))
})
