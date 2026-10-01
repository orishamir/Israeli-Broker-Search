import { flushSync } from 'svelte'
import { beforeEach, expect, test } from 'vitest'
import * as core from './core/core'
import { percent } from './format'
import { placeId, ShortTermState } from './short-term.svelte'
import { t } from './text'

beforeEach(() => localStorage.clear())

/** The short-term calculator, as AppState makes it: its effects need a root. */
function start(expert = false): ShortTermState {
  let short!: ShortTermState
  $effect.root(() => {
    short = new ShortTermState(() => expert)
  })
  return short
}

const results = (short: ShortTermState) => {
  const { comparison } = short
  if ('error' in comparison) throw new Error(comparison.error)
  return comparison.results
}

test('the places the core ticks are ticked at first, each in its own color', () => {
  const short = start()
  const ticked = short.listedPlaces.filter(({ info }) => info.comparedAtFirst)
  expect(short.compared.map(({ id }) => id)).toEqual(ticked.map(({ id }) => id))
  expect(new Set(short.compared.map(({ color }) => color)).size).toBe(ticked.length)
})

test("the table is the core's comparison, best first, and the best is its first place", () => {
  const short = start()
  const data = core.compareShortTerm(short.inputs)
  expect(results(short).map(({ place }) => place.key)).toEqual(data.places.map(({ key }) => key))
  expect(short.best?.place.key).toEqual(data.places[0].key)
  expect(results(short)[0].rank).toBe(1)
})

test('with money every month, the deposits come last, not offered, and unranked', () => {
  const short = start()
  short.monthlyDeposit = 1000
  const deposits = results(short).filter(({ place }) => place.info.rates.length > 0)
  expect(deposits.length).toBeGreaterThan(0)
  for (const { outcome, rank, notOffered } of deposits) {
    expect(outcome).toBeUndefined()
    expect(rank).toBeUndefined()
    expect(notOffered).toBe('מקבל סכום אחד')
  }
  expect(results(short).at(-1)!.outcome).toBeUndefined()
})

test('More options off, prices rise as usual whatever the inflation field says', () => {
  const short = start()
  short.inflationPercent = 9
  expect(short.inputs.inflationPercent).toBe(core.usualInflationPercent())
  expect(start(true).inputs.inflationPercent).toBe(core.usualInflationPercent())
  const expert = start(true)
  expert.inflationPercent = 9
  expect(expert.inputs.inflationPercent).toBe(9)
})

test('what a place pays for these months: a deposit its rate for the term, a fund its fee', () => {
  const short = start()
  const [fund] = short.listedPlaces
  expect(short.paysFor(fund)).toBe(t.feeOf(percent(fund.info.feePercent!)))
  const leumi = short.listedPlaces.find(
    ({ englishLabel }) => englishLabel === 'Fixed-rate deposit · Bank Leumi',
  )!
  expect(short.paysFor(leumi)).toBe(t.rateOf(percent(leumi.info.rates[core.termFor(12)!].rate!)))
  // No rate published for the term.
  const massad = short.listedPlaces.find(
    ({ englishLabel }) => englishLabel === 'Fixed-rate deposit · Bank Massad',
  )!
  short.months = 48
  expect(short.paysFor(massad)).toBe(t.noRate)
})

test('a deposit of your own is added ticked at today’s rate, numbered, saved, and deleted unticked', () => {
  const short = start()
  const first = short.addYourDeposit()
  const second = short.addYourDeposit()
  expect([first.name, second.name]).toEqual([t.yourDepositName, t.yourDepositNumbered(2)])
  expect(first.ratePercent).toBe(core.todaysRatePercent())
  expect(short.selected.has(placeId({ kind: 'yours', id: first.id }))).toBe(true)
  short.updateYourDeposit({ ...first, ratePercent: 4.5 })
  const yours = results(short).find(({ place }) => place.yours?.id === first.id)!
  // 4.5% for a year on ₪100,000, less 15%: ₪3,825 left of the interest.
  expect(yours.outcome!.afterTax).toBeCloseTo(103_825, 6)
  flushSync()
  expect(JSON.parse(localStorage.getItem('your-deposits-v1')!)).toHaveLength(2)
  expect(start().yourDeposits.map(({ id }) => id)).toEqual([first.id, second.id])
  short.deleteYourDeposit(first.id)
  expect(short.selected.has(placeId({ kind: 'yours', id: first.id }))).toBe(false)
  expect(short.yourDeposits.map(({ id }) => id)).toEqual([second.id])
})

test('an empty rate of your own says which field to fill in', () => {
  const short = start()
  const yours = short.addYourDeposit()
  short.updateYourDeposit({ ...yours, ratePercent: null })
  expect(short.inputError).toBe('מלאו את הריבית של הפיקדון שלכם')
})
