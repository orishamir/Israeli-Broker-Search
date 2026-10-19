// The workbench's rows and chart entries, on the real core: the S&P 500 in
// every way, with ₪10,000 and ₪2,000 a month for 20 years (the redesign's
// scenario b), and another index's plans on one exchange.
import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState, planId } from './app.svelte'
import { missingPlans, planEntries, wayColorOf, wayEntries } from './chart-entries'
import * as core from './core/core'
import { wayKey } from './kinds'
import { FEE_TYPES } from './fee-breakdown'
import { planRows, rankedRows, usedColumns, wayRows } from './within'

beforeEach(() => {
  localStorage.clear()
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})

function scenarioB(thing: 'sp500' | 'other' = 'sp500'): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState({})
  })
  app.answers.now = 10_000
  app.answers.monthly = 2_000
  app.answers.horizon = { Months: 240 }
  app.expert = true
  app.chooseThing(thing)
  flushSync()
  return app
}

const whyNot = (app: AppState) => (reason: Parameters<AppState['whyNotWords']>[0]) => app.whyNotWords(reason)

test('the S&P 500 in every way: a row for each way the core lists, best first, numbered from 1', () => {
  const app = scenarioB()
  const view = app.waysComparison
  if (!view || !('ways' in view)) throw new Error('no ways compared')
  const rows = wayRows(
    view.ways,
    wayColorOf(app.wayChoices),
    (key) => app.plansById.get(planId(key))?.label ?? '',
    (way) => app.resultsOf(way.plans),
    whyNot(app),
  )
  expect(rows.map(({ id }) => id)).toEqual(view.ways.map(({ way }) => wayKey(way)))
  const numbered = rows.filter(({ rank }) => rank !== undefined).map(({ rank }) => rank)
  expect(numbered).toEqual(numbered.map((_, index) => index + 1))
  // What the table says is what the core worked out.
  const first = view.ways.find(({ afterTax }) => afterTax !== undefined)!
  const row = rows.find(({ id }) => id === wayKey(first.way))!
  expect(row.cells?.LeftAfterTax).toContain('₪')
  expect(row.cells?.CostAYear).toMatch(/%$|^[^\d]/)
  // A way opens onto the plans that can hold it, under keys of their own.
  const opened = rows.find(({ more }) => (more?.length ?? 0) > 1)!
  expect(opened.more!.every(({ key }) => key.startsWith(`${opened.id}>`))).toBe(true)
  expect(new Set(rows.flatMap(({ more }) => more ?? []).map(({ key }) => key)).size).toBe(
    rows.flatMap(({ more }) => more ?? []).length,
  )
})

test('a way whose fund keeps something says so under its name', () => {
  const app = scenarioB()
  const view = app.waysComparison
  if (!view || !('ways' in view)) throw new Error('no ways compared')
  const rows = wayRows(
    view.ways,
    wayColorOf(app.wayChoices),
    () => '',
    () => [],
    whyNot(app),
  )
  const withFund = view.ways.filter(({ productCost }) => (productCost ?? 0) > 0)
  expect(withFund.length).toBeGreaterThan(0)
  for (const way of withFund) {
    const row = rows.find(({ id }) => id === wayKey(way.way))!
    expect(row.notes.some(({ kind }) => kind === 'note')).toBe(true)
  }
})

test('the ways are drawn as their best plans, each way in its own color, ranked as the table is', () => {
  const app = scenarioB()
  const view = app.waysComparison
  if (!view || !('ways' in view)) throw new Error('no ways compared')
  const entries = wayEntries(view.ways, wayColorOf(app.wayChoices))
  expect(entries.length).toBeGreaterThan(1)
  expect(new Set(entries.map(({ id }) => id)).size).toBe(entries.length)
  expect(new Set(entries.map(({ color }) => color)).size).toBe(entries.length)
  expect(entries.map(({ rank }) => rank)).toEqual(entries.map((_, index) => index + 1))
  // The fund's own cost is one of the parts of the fee bars, and a bar's
  // parts add up to its total.
  for (const { outcome } of entries) {
    const parts = FEE_TYPES.reduce((sum, { key }) => sum + outcome.fees[key], 0)
    expect(parts).toBeCloseTo(outcome.fees.total, 6)
  }
  expect(entries.some(({ outcome }) => outcome.fees.product > 0)).toBe(true)
})

test('another index on one exchange: rows and chart entries for the plans compared, with the ones left out grouped by why', () => {
  const app = scenarioB('other')
  const view = app.comparison
  if (!view || !('results' in view)) throw new Error('no plans compared')
  const rows = planRows(view.results, whyNot(app))
  expect(rows).toHaveLength(view.results.length)
  expect(rows.filter(({ linked }) => linked)).toHaveLength(
    view.results.filter(({ outcome }) => outcome).length,
  )
  expect(planEntries(view.results).map(({ id }) => id)).toEqual(
    view.results.filter(({ outcome }) => outcome).map(({ plan }) => plan.id),
  )
  const left = missingPlans(view.results).flatMap(({ plans }) => plans)
  expect(left).toHaveLength(view.results.filter(({ outcome }) => !outcome).length)
  // The columns the core names for the index arena are all there.
  expect(usedColumns(core.columnsFor('Index'), rows).map(({ id }) => id)).toEqual([
    'LeftAfterTax',
    'CostAYear',
    'Fees',
    'Tax',
  ])
})

test('the guide’s list has only the columns it has amounts for, and the top rows come first', () => {
  const app = scenarioB()
  app.expert = false
  app.chooseThing('short')
  flushSync()
  const recommendation = app.recommendation!
  const rows = rankedRows(recommendation.plans, recommendation.arena, true)
  expect(rows.length).toBe(recommendation.plans.length)
  const columns = usedColumns(recommendation.columns, rows).map(({ id }) => id)
  expect(columns).toContain('LeftAfterTax')
  expect(columns).not.toContain('Fees')
  expect(columns).toContain('Tax')
})

test('an ETF in Tel Aviv can be held by two funds, and a picked fund that stops fitting is dropped', () => {
  const app = scenarioB('other')
  app.security = 'Etf'
  app.exchange = 'Tlv'
  expect(app.productChoices.map(({ value }) => value)).toEqual(['IsraeliEtf', 'ForeignEtfInTelAviv'])
  app.product = 'ForeignEtfInTelAviv'
  expect(app.chosenProduct).toBe('ForeignEtfInTelAviv')
  app.exchange = 'Usa'
  expect(app.productChoices).toHaveLength(1)
  expect(app.chosenProduct).toBeUndefined()
  app.security = 'Stock'
  expect(app.productChoices).toHaveLength(0)
})
