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
function start(): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState()
  })
  return app
}

const results = (app: AppState) => {
  const { comparison } = app
  if ('error' in comparison) throw new Error(comparison.error)
  return comparison.results
}

const draftOf = (details: Details | null) => {
  if (details?.kind !== 'draft') throw new Error('no draft open')
  return details.draft
}

test("each broker's usual plan is ticked at first, and the plans are ranked best first", () => {
  const app = start()
  const usual = app.brokers.map((broker, index) =>
    planId({ kind: 'listed', broker: index, plan: broker.newCustomerPlan }),
  )
  expect([...app.selected]).toEqual(usual)
  const ranked = results(app)
  expect(ranked.map(({ plan }) => plan.id).toSorted()).toEqual(usual.toSorted())
  const left = ranked.map(({ outcome }) => outcome?.afterSelling ?? -Infinity)
  expect(left).toEqual(left.toSorted((a, b) => b - a))
  expect(app.purchase).toBe('an ETF bought in the USA')
})

test('bad inputs are reported in words, not thrown', () => {
  const app = start()
  app.monthlyDeposit = -5
  expect(app.comparison).toEqual({ error: expect.stringMatching(/negative/) })
  expect(app.buying.largestTrade).toBeNull()
  app.monthlyDeposit = 2000
  app.yearlyReturnPercent = null
  expect(app.comparison).toEqual({ error: expect.stringMatching(/yearly return/) })
  app.yearlyReturnPercent = 10
  expect(results(app).length).toBeGreaterThan(0)
  expect(app.buying.largestTrade).toBeGreaterThan(0)
})

test('the share price is asked for only where it matters', () => {
  const app = start()
  expect(app.sharePriceSymbol).toBe('$')
  app.exchange = 'Europe'
  expect(app.sharePriceSymbol).toBe('€')
  app.exchange = 'Tlv'
  expect(app.sharePriceSymbol).toBeFalsy()
})

test('ticked plans all differ in color, and an unticked one frees its color', () => {
  const app = start()
  const tickedColors = () => app.plans.filter(({ id }) => app.selected.has(id)).map(({ color }) => color)
  expect(new Set(tickedColors()).size).toBe(app.selected.size)

  const [first] = app.selected
  const freed = app.plansById.get(first)!.color
  app.setSelected([first], false)
  const other = app.plans.find(({ id }) => !app.selected.has(id))!
  app.setSelected([other.id], true)
  expect(other.color).toBe(freed)

  app.setSelected(
    app.plans.map(({ id }) => id),
    true,
  )
  expect(new Set(tickedColors()).size).toBe(app.plans.length)
  app.setSelected([other.id], false)
  expect(other.color).toBe('#8e95a5')
})

test('unticking a plan unpins it; showing its fees pins it last and opens the breakdown', () => {
  const app = start()
  const [first, second] = app.selected
  app.togglePin(first)
  app.togglePin(second)
  app.showFees(first)
  expect([...app.pinned]).toEqual([second, first])
  expect(app.chartView).toBe('breakdown')
  app.setSelected([first], false)
  expect(app.pinned.has(first)).toBe(false)
  app.togglePin(second)
  expect(app.pinned.size).toBe(0)
})

test('a new plan of your own takes the next free name, and is ticked when added', () => {
  const app = start()
  app.draftNewPlan()
  const draft = draftOf(app.details)
  expect(core.planInfo(draft.plan).name).toBe('Your plan')
  app.addYourPlan(draft)
  expect(app.selected.has(planId({ kind: 'yours', id: draft.id }))).toBe(true)
  app.draftNewPlan()
  expect(core.planInfo(draftOf(app.details).plan).name).toBe('Your plan 2')
})

test('deleting a plan of your own unticks it, unpins it and stops hovering it', () => {
  const app = start()
  app.draftNewPlan()
  const draft = draftOf(app.details)
  app.addYourPlan(draft)
  const id = planId({ kind: 'yours', id: draft.id })
  app.togglePin(id)
  app.hovered = id
  app.deleteYourPlan(draft.id)
  expect(app.selected.has(id)).toBe(false)
  expect(app.pinned.has(id)).toBe(false)
  expect(app.hovered).toBeNull()
  expect(app.yourPlans).toEqual([])
})

test('your plans are kept for the next visit, and ones the core cannot read are dropped', () => {
  const app = start()
  app.draftNewPlan()
  const draft = draftOf(app.details)
  app.addYourPlan(draft)
  app.editorView = 'full'
  flushSync()
  const saved = JSON.parse(localStorage.getItem('your-plans-v1')!) as unknown[]
  expect(saved).toHaveLength(1)

  const junk = { id: 'junk', plan: { nonsense: true }, brokerName: null, basedOn: null }
  localStorage.setItem('your-plans-v1', JSON.stringify([...saved, junk]))
  const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
  const again = start()
  expect(again.yourPlans.map(({ id }) => id)).toEqual([draft.id])
  expect(again.editorView).toBe('full')
  expect(warn).toHaveBeenCalledOnce()
})

test('a copy knows its original by name, shares its color, and charges the same until changed', () => {
  const app = start()
  const original = app.listedPlans.find(({ id }) => app.selected.has(id))!
  app.draftCopyOf(original)
  const draft = draftOf(app.details)
  expect(draft.basedOn).toEqual({
    broker: original.subtitle,
    plan: original.info.name,
    track: original.info.tariff.tracks[app.trackOf(original) ?? 0]?.name,
  })
  app.addYourPlan(draft)
  const copy = app.plans.find(({ yours }) => yours?.id === draft.id)!
  expect(copy.original).toBe(original)
  expect(copy.color).toBe(original.color)
  expect(copy.subtitle).toBe(`Your deal · ${original.subtitle}`)
  expect(copy.label).toBe(original.label.replace(original.info.name, `${original.info.name}, your deal`))
  const leftAfterSelling = (id: string) =>
    results(app).find(({ plan }) => plan.id === id)!.outcome!.afterSelling
  expect(leftAfterSelling(copy.id)).toBe(leftAfterSelling(original.id))
  // The fees shown for it are its own claim: no caveats.
  expect(app.feesFor(copy).caveats).toEqual([])
})

test('off, the expert inputs are not sent; on, they are, and keeping the holdings drops the sale', () => {
  const app = start()
  expect(app.moreOptions).toBe(false)
  app.depositGrowthPercent = 5
  app.inflationPercent = 2
  app.atEnd = 'hold'
  expect(app.inputs).toMatchObject({ depositGrowthPercent: 0, inflationPercent: 0, sellAtEnd: true })
  expect(app.inTodaysMoney).toBe(false)
  const plain = results(app)[0].outcome!

  app.moreOptions = true
  expect(app.inputs).toMatchObject({ depositGrowthPercent: 5, inflationPercent: 2, sellAtEnd: false })
  expect(app.sellAtEnd).toBe(false)
  expect(app.inTodaysMoney).toBe(true)
  const kept = results(app).find(({ plan }) => plan.id === results(app)[0].plan.id)!.outcome!
  expect(kept.fees.selling).toBe(0)
  expect(kept.afterSelling).toBe(kept.held)
  expect(kept.yearlyCostPercent).not.toBe(plain.yearlyCostPercent)
  flushSync()
  expect(localStorage.getItem('more-options')).toBe('true')
})

test('an example fills every basic input, and leaves the expert ones', () => {
  const app = start()
  app.moreOptions = true
  app.inflationPercent = 3
  const example = app.examples.find(({ name }) => name === 'Bonds, 5 years')!
  app.applyExample(example)
  expect(app.inputs).toMatchObject({
    security: example.security,
    exchange: example.exchange,
    firstDeposit: example.firstDeposit,
    monthlyDeposit: example.monthlyDeposit,
    yearlyReturnPercent: example.yearlyReturnPercent,
    years: example.years,
    buyEveryMonths: example.buyEveryMonths,
    inflationPercent: 3,
  })
  expect(app.examples.map(({ name }) => name)).toEqual(core.examples().map(({ name }) => name))
})

test('the sweep varies the monthly deposit while there is one, else the one-time deposit, and follows the table', () => {
  const app = start()
  expect(app.swept).toBe('Monthly')
  const sweep = app.sweep!
  expect(sweep.swept).toBe('Monthly')
  expect(sweep.plans.map(({ key }) => planId(key))).toEqual([...app.selected])
  // At each amount, the cost is the table's for that deposit.
  const [first] = sweep.plans
  const amount = sweep.amounts[3]
  app.monthlyDeposit = amount
  const row = results(app).find(({ plan }) => plan.id === planId(first.key))!
  expect(first.costs![3]).toBe(row.outcome!.yearlyCostPercent)
  // Typing the swept deposit doesn't change the sweep.
  expect(app.sweep).toBe(sweep)

  app.monthlyDeposit = 0
  expect(app.swept).toBe('OneTime')
  expect(app.sweep!.swept).toBe('OneTime')
  // The swept deposit's own field may be empty: it isn't used. Another empty field stops the sweep.
  app.firstDeposit = null
  expect(app.sweep).toBeDefined()
  app.yearlyReturnPercent = null
  expect(app.sweep).toBeUndefined()
})

test('a link carries the comparison: opened from one, the page shows the same, your plans included', () => {
  const app = start()
  app.draftNewPlan()
  const draft = draftOf(app.details)
  app.addYourPlan(draft)
  app.security = 'Bond'
  app.exchange = 'Tlv'
  app.monthlyDeposit = 3_000
  app.years = 7
  app.moreOptions = true
  app.inflationPercent = 2
  const [firstListed] = app.listedPlans
  app.setSelected([firstListed.id], true)
  const link = app.shareLink()
  expect(link).toMatch(/^http:\/\/localhost:3000\/#s=Bond&x=Tlv/)

  localStorage.clear()
  const opened = start_from(link)
  expect(opened.security).toBe('Bond')
  expect(opened.exchange).toBe('Tlv')
  expect(opened.monthlyDeposit).toBe(3_000)
  expect(opened.years).toBe(7)
  expect(opened.moreOptions).toBe(true)
  expect(opened.inflationPercent).toBe(2)
  expect(opened.yourPlans.map(({ id }) => id)).toEqual([draft.id])
  expect([...opened.selected].toSorted()).toEqual([...app.selected].toSorted())
  expect(results(opened).map(({ plan, outcome }) => [plan.id, outcome?.afterSelling])).toEqual(
    results(app).map(({ plan, outcome }) => [plan.id, outcome?.afterSelling]),
  )
})

test('a link without plans keeps what is ticked; one with plans replaces it, and yours joins by id', () => {
  const first = start()
  first.draftNewPlan()
  const mine = draftOf(first.details)
  first.addYourPlan(mine)
  flushSync()
  const ticked = [...first.selected]

  // Only inputs: the ticks stay, your plan included.
  const inputsOnly = start_from(`http://localhost:3000/#${encode({ years: 3 })}`)
  expect(inputsOnly.years).toBe(3)
  expect([...inputsOnly.selected]).toEqual(ticked)

  // A newer version of your own plan, and one listed plan: exactly those are ticked.
  const newer = { ...mine, plan: core.rename(mine.plan, 'Renamed') }
  const withPlans = start_from(
    `http://localhost:3000/#${encode({ plans: [first.listedPlans[0].label], yours: [newer] })}`,
  )
  expect(withPlans.yourPlans).toHaveLength(1)
  expect(core.planInfo(withPlans.yourPlans[0].plan).name).toBe('Renamed')
  expect([...withPlans.selected].toSorted()).toEqual(
    [first.listedPlans[0].id, planId({ kind: 'yours', id: mine.id })].toSorted(),
  )
})

/** An app opened from `link`, as App.svelte opens one from the address. */
function start_from(link: string): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState(decode(new URL(link).hash))
  })
  return app
}
