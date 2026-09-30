import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState, PLAN_COLORS, planId, type Details } from './app.svelte'
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

/** Answers the app's sweep request, as the worker does (see sweeper.ts). */
function answerSweep(app: AppState) {
  const request = app.sweepRequest
  let sweep: core.SweepData | undefined
  try {
    sweep = core.sweep(request.inputs, request.swept)
  } catch {
    sweep = undefined
  }
  app.sweepAnswer = { request, sweep }
}

const draftOf = (details: Details | null) => {
  if (details?.kind !== 'draft') throw new Error('no draft open')
  return details.draft
}

test("each broker's usual plan and the provident fund's average are ticked at first, ranked by what is left after tax", () => {
  const app = start()
  const usual = app.brokers.flatMap((broker, index) =>
    broker.comparedAtFirst ? [planId({ kind: 'listed', broker: index, plan: broker.newCustomerPlan })] : [],
  )
  expect([...app.selected]).toEqual(usual)
  // Every broker, and of the funds only the provident fund for investment.
  const funds = app.brokers.filter(({ kind }) => kind === 'Funds')
  expect(funds.map(({ englishName, comparedAtFirst }) => [englishName, comparedAtFirst])).toEqual([
    ['Provident fund for investment', true],
    ['Study fund', false],
    ['Savings policy', false],
  ])
  expect(usual).toHaveLength(app.brokers.length - 2)
  const ranked = results(app)
  expect(ranked.map(({ plan }) => plan.id).toSorted()).toEqual(usual.toSorted())
  const left = ranked.map(({ outcome }) => outcome?.afterTax ?? -Infinity)
  expect(left).toEqual(left.toSorted((a, b) => b - a))
  expect(app.purchase).toBe('קרן סל שנקנית בארה״ב')
})

test('how the money is taken out is asked only while a ticked plan pays a pension, and everything is sold', () => {
  const app = start()
  const fund = app.listedPlans.find((plan) => plan.englishLabel === 'Provident fund · Average fee')!
  const outcomeOf = () => results(app).find(({ plan }) => plan.id === fund.id)!.outcome!
  expect(app.pensionFromAge).toBe(60)
  expect(app.inputs).toMatchObject({ asPension: false })
  const atOnce = outcomeOf()
  expect(atOnce.tax).toBeGreaterThan(0)

  // 45 today, 65 after the 20 years: old enough, and the fund comes first.
  app.wayOut = 'pension'
  expect(app.inputs).toMatchObject({ asPension: true, age: 45 })
  expect(outcomeOf().tax).toBe(0)
  expect(outcomeOf().afterTax).toBe(atOnce.afterSelling)
  expect(results(app)[0].plan.id).toBe(fund.id)
  app.age = 39
  expect(outcomeOf().tax).toBe(atOnce.tax)
  app.age = null
  expect(app.comparison).toEqual({ error: 'מלאו את הגיל שלכם' })
  app.age = 45

  // Without the fund there's no pension to ask about, and none is sent.
  app.setSelected([fund.id], false)
  expect(app.pensionFromAge).toBeUndefined()
  expect(app.inputs).toMatchObject({ asPension: false })
  app.setSelected([fund.id], true)
  expect(app.inputs).toMatchObject({ asPension: true })
  // Nor when the holdings are kept: nothing is taken out.
  app.moreOptions = true
  app.atEnd = 'hold'
  expect(app.pensionFromAge).toBeUndefined()
  expect(outcomeOf().tax).toBe(0)
})

test("deposits over a fund's yearly ceiling leave it without numbers, saying why", () => {
  const app = start()
  const fund = app.listedPlans.find((plan) => plan.englishLabel === 'Provident fund · Average fee')!
  app.monthlyDeposit = 10_000
  const row = results(app).at(-1)!
  expect(row.plan.id).toBe(fund.id)
  expect(row).toMatchObject({ outcome: undefined, rank: undefined, whyNot: 'OverTheCeiling' })
  expect(row.notOffered).toBe(
    'אי אפשר להפקיד יותר מ-₪83,641 בשנה, וההפקדות שלכם בשנה הראשונה מגיעות ל-₪130,000',
  )
})

test('a study fund is locked for six years, and taxes only the gains on what is over its yearly amount', () => {
  const app = start()
  const study = app.listedPlans.find((plan) => plan.englishLabel === 'Study fund · Average fee')!
  app.setSelected([study.id], true)
  const rowOf = () => results(app).find(({ plan }) => plan.id === study.id)!
  // ₪10,000 and ₪2,000 a month is more than the ₪20,566 a year whose gains
  // are tax-free: some tax, less than the same money pays elsewhere.
  const fund = app.listedPlans.find((plan) => plan.englishLabel === 'Provident fund · Average fee')!
  const taxOf = (id: string) => results(app).find(({ plan }) => plan.id === id)!.outcome!.tax
  expect(taxOf(study.id)).toBeGreaterThan(0)
  expect(taxOf(study.id)).toBeLessThan(taxOf(fund.id))
  app.firstDeposit = 0
  app.monthlyDeposit = 1_500
  expect(taxOf(study.id)).toBe(0)
  // The fund says its rule under its name, and its row what became of it.
  expect(study.broker!.taxRule).toBe('אין מס על הרווחים אחרי 6 שנים, על עד ₪20,566 שהופקדו בשנה')
  expect(rowOf().taxNote).toBe('אין מס: ההפקדות שלכם בתוך ₪20,566 בשנה')
  app.monthlyDeposit = 2_000
  expect(rowOf().taxNote).toBe('המס הוא רק על מה שמופקד מעל ₪20,566 בשנה')
  expect(results(app).find(({ plan }) => plan.broker?.kind !== 'Funds')!.taxNote).toBeUndefined()
  // No pension from a study fund: it asks nothing about the way out.
  expect(study.info.pensionFromAge).toBeUndefined()

  app.years = 5
  expect(rowOf()).toMatchObject({ outcome: undefined, whyNot: 'Locked' })
  expect(rowOf().notOffered).toContain('רק 6 שנים אחרי ההפקדה הראשונה')
})

test('bad inputs are reported in words, not thrown', () => {
  const app = start()
  app.monthlyDeposit = -5
  expect(app.comparison).toEqual({ error: expect.stringMatching(/ערך שלילי/) })
  expect(app.buying.largestTrade).toBeNull()
  app.monthlyDeposit = 2000
  app.yearlyReturnPercent = null
  expect(app.comparison).toEqual({ error: expect.stringMatching(/התשואה השנתית/) })
  app.yearlyReturnPercent = 10
  expect(results(app).length).toBeGreaterThan(0)
  expect(app.buying.largestTrade).toBeGreaterThan(0)
})

test("a plan's rank is its row's number, counting only the plans with an outcome", () => {
  const app = start()
  app.setSelected(
    app.plans.map(({ id }) => id),
    true,
  )
  app.exchange = 'Europe'
  const all = results(app)
  const offered = all.filter(({ outcome }) => outcome)
  expect(offered.length).toBeLessThan(all.length)
  expect(offered.map(({ rank }) => rank)).toEqual(offered.map((_, index) => index + 1))
  expect(all.filter(({ outcome }) => !outcome).every(({ rank }) => rank === undefined)).toBe(true)
  for (const { plan, rank } of all) expect(app.rankOf(plan.id)).toBe(rank)
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

  // Every plan ticked: more plans than colors, so every color is used, and
  // no color much more than another.
  app.setSelected(
    app.plans.map(({ id }) => id),
    true,
  )
  expect(app.plans.length).toBeGreaterThan(PLAN_COLORS.length)
  const uses: Record<string, number> = {}
  for (const color of tickedColors()) uses[color] = (uses[color] ?? 0) + 1
  const counts = Object.values(uses)
  expect(counts.length).toBe(PLAN_COLORS.length)
  expect(Math.max(...counts) - Math.min(...counts)).toBeLessThanOrEqual(1)
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
  expect(core.planInfo(draft.plan).name).toBe('המסלול שלכם')
  app.addYourPlan(draft)
  expect(app.selected.has(planId({ kind: 'yours', id: draft.id }))).toBe(true)
  app.draftNewPlan()
  expect(core.planInfo(draftOf(app.details).plan).name).toBe('המסלול שלכם 2')
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
  // By English names, the same in every language.
  expect(draft.basedOn).toEqual({
    broker: original.broker!.englishName,
    plan: original.info.englishName,
    track: original.info.tariff.tracks[app.trackOf(original) ?? 0]?.englishName,
  })
  app.addYourPlan(draft)
  const copy = app.plans.find(({ yours }) => yours?.id === draft.id)!
  expect(copy.original).toBe(original)
  expect(copy.color).toBe(original.color)
  expect(copy.subtitle).toBe(`העסקה שלכם · ${original.subtitle}`)
  expect(copy.label).toBe(original.label.replace(original.info.name, `${original.info.name}, העסקה שלכם`))
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
  app.atEnd = 'hold'
  app.inflationPercent = 3
  app.todaysMoney = true
  expect(app.usualInflation).toBe(2)
  expect(app.inputs).toMatchObject({
    depositGrowthPercent: 0,
    inflationPercent: 2,
    inTodaysMoney: false,
    sellAtEnd: true,
  })
  expect(app.inTodaysMoney).toBe(false)
  const plain = results(app)[0].outcome!

  app.moreOptions = true
  expect(app.inputs).toMatchObject({
    depositGrowthPercent: 5,
    inflationPercent: 3,
    inTodaysMoney: true,
    sellAtEnd: false,
  })
  expect(app.sellAtEnd).toBe(false)
  expect(app.inTodaysMoney).toBe(true)
  // The inflation alone lowers the tax; the amounts are as they will be.
  app.todaysMoney = false
  expect(app.inputs).toMatchObject({ inflationPercent: 3, inTodaysMoney: false })
  expect(app.inTodaysMoney).toBe(false)
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
  const example = app.examples.find(({ name }) => name === 'אג״ח, 5 שנים')!
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
  answerSweep(app)
  const sweep = app.sweep!
  expect(sweep.swept).toBe('Monthly')
  expect(sweep.plans.map(({ key }) => planId(key))).toEqual([...app.selected])
  // At each amount, the cost is the table's for that deposit.
  const [first] = sweep.plans
  const amount = sweep.amounts[3]
  app.monthlyDeposit = amount
  const row = results(app).find(({ plan }) => plan.id === planId(first.key))!
  expect(first.costs![3]).toBe(row.outcome!.yearlyCostPercent)
  // Typing the swept deposit asks for no new sweep.
  expect(app.sweepAnswer!.request).toBe(app.sweepRequest)

  app.monthlyDeposit = 0
  expect(app.swept).toBe('OneTime')
  answerSweep(app)
  expect(app.sweep!.swept).toBe('OneTime')
  // The swept deposit's own field may be empty: it isn't used. Another empty field stops the sweep.
  app.firstDeposit = null
  answerSweep(app)
  expect(app.sweep).toBeDefined()
  app.yearlyReturnPercent = null
  answerSweep(app)
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
  app.inflationPercent = 3
  app.todaysMoney = true
  app.wayOut = 'pension'
  app.age = 52
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
  expect(opened.inflationPercent).toBe(3)
  expect(opened.inTodaysMoney).toBe(true)
  expect(opened.inputs).toMatchObject({ asPension: true, age: 52 })
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
    `http://localhost:3000/#${encode({ plans: [first.listedPlans[0].englishLabel], yours: [newer] })}`,
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

test('the chart starts on what was lost to fees; the one by deposit is shown only with More options', () => {
  const app = start()
  expect(app.chartView).toBe('lost')
  app.moreOptions = true
  app.chartView = 'crossover'
  expect(app.chartView).toBe('crossover')
  app.moreOptions = false
  expect(app.chartView).toBe('lost')
  app.moreOptions = true
  expect(app.chartView).toBe('crossover')
})

test("the best plan's line keeps its words until the sweep answers, shown while they're about the best", () => {
  const app = start()
  const answer = () => answerSweep(app)
  // Before the first answer: words that hold the line's place, hidden.
  expect(app.aroundLine.shown).toBe(false)
  answer()
  const first = app.aroundLine
  expect(first).toEqual({ text: expect.stringMatching(/^הזול ביותר בעמלות בכל הפקדה חודשית/), shown: true })
  // Typing the deposit the sweep varies asks nothing new.
  const asked = app.sweepRequest
  app.monthlyDeposit = 2500
  expect(app.sweepRequest).toBe(asked)
  expect(app.aroundLine.shown).toBe(true)
  // Other inputs do. Until the answer, the words stay while the best plan
  // is the same, and hide when another plan is best.
  const best = app.best!.plan.id
  app.years = 25
  expect(app.sweepRequest).not.toBe(asked)
  expect(app.best!.plan.id).toBe(best)
  expect(app.aroundLine).toEqual({ ...first, shown: true })
  app.exchange = 'Tlv'
  expect(app.best!.plan.id).not.toBe(best)
  expect(app.aroundLine).toEqual({ text: first.text, shown: false })
  answer()
  expect(app.aroundLine.shown).toBe(true)
  // An answer to an older request isn't used, and one the core can't use hides the line.
  app.sweepAnswer = { request: asked, sweep: app.sweep }
  expect(app.around).toBeUndefined()
  app.sweepAnswer = { request: app.sweepRequest, sweep: undefined }
  expect(app.aroundLine.shown).toBe(false)
})
