import type { BarSeriesOption } from 'echarts/charts'
import { expect, test } from 'vitest'
import type { FeeAmounts } from './core/core'
import {
  barsOption,
  byFee,
  colorOf,
  FADED,
  FEE_TYPES,
  fitName,
  overTimeOption,
  stacking,
  type Bar,
} from './fee-breakdown'

const fees = (purchases: number, account: number): FeeAmounts => ({
  purchases,
  conversions: 0,
  account,
  management: 0,
  selling: 0,
  total: purchases + account,
})

const bars: Bar[] = [
  { id: 'a', label: 'A', color: '#111111', hollow: false, fees: fees(500, 100) },
  { id: 'b', label: 'B', color: '#222222', hollow: true, fees: fees(100, 300) },
]

/** Six pixels a character, standing in for the canvas. */
const measure = (text: string) => [...text].length * 6

test('bars are sorted by the total, or by the focused fee', () => {
  expect(byFee(bars, 'all').map(({ id }) => id)).toEqual(['b', 'a'])
  expect(byFee(bars, 'account').map(({ id }) => id)).toEqual(['a', 'b'])
  expect(byFee(bars, 'purchases').map(({ id }) => id)).toEqual(['b', 'a'])
})

test('a focused fee is stacked first and keeps its color; the rest fade', () => {
  expect(stacking('all')).toBe(FEE_TYPES)
  expect(stacking('account').map(({ key }) => key)).toEqual([
    'account',
    'purchases',
    'conversions',
    'management',
    'selling',
  ])
  const account = FEE_TYPES.find(({ key }) => key === 'account')!
  const purchases = FEE_TYPES.find(({ key }) => key === 'purchases')!
  expect(colorOf(account, 'account')).toBe(account.color)
  expect(colorOf(purchases, 'account')).toBe(FADED)
  expect(colorOf(purchases, 'all')).toBe(purchases.color)
})

test('the bars name each plan, marking pinned ones, with the totals at the end', () => {
  const option = barsOption({ bars, focus: 'all', pinned: new Set(['b']), width: 800, measure })
  const [names, totals] = option.yAxis as {
    data: string[]
    axisLabel: { formatter: (id: string, index: number) => string }
  }[]
  expect(names.data).toEqual(['a', 'b'])
  expect(names.axisLabel.formatter('a', 0)).toBe('{dot0|●} {name|A}')
  expect(names.axisLabel.formatter('b', 1)).toBe('{dot1|◯} {pinned1|B}')
  expect(totals.data).toEqual(['₪600', '₪400'])
  // The outline, the slots above the bars where the names go, then a
  // stacked series per fee, each faded for unpinned plans.
  const series = option.series as (BarSeriesOption & { type: string })[]
  expect(series.map(({ type }) => type)).toEqual(['custom', 'bar', 'bar', 'bar', 'bar', 'bar', 'bar'])
  expect(series[2].data).toEqual([
    { value: 500, itemStyle: { opacity: 0.6 } },
    { value: 100, itemStyle: { opacity: 1 } },
  ])
  expect(series.slice(2).map(({ name }) => name)).toEqual(FEE_TYPES.map(({ name }) => name))
})

test("the tooltip is headed by the plan's name, not its id", () => {
  const option = barsOption({ bars, focus: 'all', pinned: new Set(), width: 800, measure })
  const { axisPointer } = option.tooltip as {
    axisPointer: { label: { formatter: (params: { value: string }) => string } }
  }
  expect(axisPointer.label.formatter({ value: 'b' })).toBe('B')
})

test('a name too long for the box is cut, by whole characters', () => {
  expect(fitName('Own plan', 48, measure)).toBe('Own plan')
  expect(fitName('Own plan with a long name', 60, measure)).toBe('Own plan…')
  expect(fitName('🙂🙂🙂🙂', 18, measure)).toBe('🙂🙂…')
  // The box's width, less its margins, the dot and a pinned name's padding:
  // 150 - 8 - 12 - 10 leaves 120 px, twenty characters with the "…".
  const long = [{ ...bars[0], label: 'A plan of your own with a long name' }]
  const name = (width: number) => {
    const option = barsOption({ bars: long, focus: 'all', pinned: new Set(), width, measure })
    const [names] = option.yAxis as { axisLabel: { formatter: (id: string, index: number) => string } }[]
    return names.axisLabel.formatter('a', 0)
  }
  expect(name(800)).toBe('{dot0|●} {name|A plan of your own with a long name}')
  expect(name(150)).toBe('{dot0|●} {name|A plan of your own…}')
})

test('amounts show inside a part only where they fit', () => {
  const formatter = (width: number, focus: 'all' | 'account') => {
    const option = barsOption({ bars, focus, pinned: new Set(), width, measure })
    const purchases = (option.series as BarSeriesOption[]).find(({ name }) => name === 'Purchases')!
    return purchases.label!.formatter as (params: { value: number }) => string
  }
  // 800 px wide, 740 for ₪600 of bars: ₪500 is about 617 px, ₪20 about 25.
  expect(formatter(800, 'all')({ value: 500 })).toBe('₪500')
  expect(formatter(800, 'all')({ value: 20 })).toBe('')
  // Faded parts show nothing.
  expect(formatter(800, 'account')({ value: 500 })).toBe('')
})

test('over time, each fee piles up from nothing to the total by the end of each year', () => {
  const byYear = [fees(10, 5), fees(20, 15), fees(30, 30)]
  const option = overTimeOption(byYear, 'all')
  const series = option.series as { name: string; data: number[][] }[]
  expect(series.map(({ name }) => name)).toEqual(FEE_TYPES.map(({ name }) => name))
  expect(series[0].data).toEqual([
    [0, 0],
    [1, 10],
    [2, 20],
    [3, 30],
  ])
  expect(option.xAxis).toMatchObject({ max: 3 })
})
