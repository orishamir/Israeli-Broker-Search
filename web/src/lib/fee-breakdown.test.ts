import type { BarSeriesOption } from 'echarts/charts'
import { expect, test } from 'vitest'
import type { FeeAmounts } from './core/core'
import {
  barsOption,
  byFee,
  colorOf,
  FADED,
  FEE_TYPES,
  nameWidth,
  overTimeOption,
  rowHeight,
  stacking,
  type Bar,
} from './fee-breakdown'

const fees = (purchases: number, custody: number): FeeAmounts => ({
  purchases,
  conversions: 0,
  custody,
  handling: 0,
  selling: 0,
  total: purchases + custody,
})

const bars: Bar[] = [
  { id: 'a', label: 'A', color: '#111111', hollow: false, fees: fees(500, 100) },
  { id: 'b', label: 'B', color: '#222222', hollow: true, fees: fees(100, 300) },
]

test('bars are sorted by the total, or by the focused fee', () => {
  expect(byFee(bars, 'all').map(({ id }) => id)).toEqual(['b', 'a'])
  expect(byFee(bars, 'custody').map(({ id }) => id)).toEqual(['a', 'b'])
  expect(byFee(bars, 'purchases').map(({ id }) => id)).toEqual(['b', 'a'])
})

test('a focused fee is stacked first and keeps its color; the rest fade', () => {
  expect(stacking('all')).toBe(FEE_TYPES)
  expect(stacking('custody').map(({ key }) => key)).toEqual([
    'custody',
    'purchases',
    'conversions',
    'handling',
    'selling',
  ])
  const custody = FEE_TYPES.find(({ key }) => key === 'custody')!
  const purchases = FEE_TYPES.find(({ key }) => key === 'purchases')!
  expect(colorOf(custody, 'custody')).toBe(custody.color)
  expect(colorOf(purchases, 'custody')).toBe(FADED)
  expect(colorOf(purchases, 'all')).toBe(purchases.color)
})

test('the bars name each plan, marking pinned ones, with the totals at the end', () => {
  const option = barsOption({ bars, focus: 'all', pinned: new Set(['b']), width: 800, narrow: false })
  const [names, totals] = option.yAxis as {
    data: string[]
    axisLabel: { formatter: (id: string, index: number) => string }
  }[]
  expect(names.data).toEqual(['a', 'b'])
  expect(names.axisLabel.formatter('a', 0)).toBe('{dot0|●} {name|A}')
  expect(names.axisLabel.formatter('b', 1)).toBe('{dot1|◯} {pinned1|B}')
  expect(totals.data).toEqual(['₪600', '₪400'])
  // The outline, then a stacked series per fee, each faded for unpinned plans.
  const series = option.series as (BarSeriesOption & { type: string })[]
  expect(series.map(({ type }) => type)).toEqual(['custom', 'bar', 'bar', 'bar', 'bar', 'bar'])
  expect(series[1].data).toEqual([
    { value: 500, itemStyle: { opacity: 0.6 } },
    { value: 100, itemStyle: { opacity: 1 } },
  ])
  expect(series.slice(1).map(({ name }) => name)).toEqual(FEE_TYPES.map(({ name }) => name))
})

test('amounts show inside a part only where they fit', () => {
  const formatter = (width: number, focus: 'all' | 'custody') => {
    const option = barsOption({ bars, focus, pinned: new Set(), width, narrow: false })
    const purchases = (option.series as BarSeriesOption[]).find(({ name }) => name === 'Purchases')!
    return purchases.label!.formatter as (params: { value: number }) => string
  }
  // 800 px wide, 620 for ₪600 of bars: ₪500 is about 517 px, ₪20 about 21.
  expect(formatter(800, 'all')({ value: 500 })).toBe('₪500')
  expect(formatter(800, 'all')({ value: 20 })).toBe('')
  // Faded parts show nothing.
  expect(formatter(800, 'custody')({ value: 500 })).toBe('')
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

test('phones give the names less room and the rows more height', () => {
  expect([nameWidth(true), rowHeight(true)]).toEqual([104, 40])
  expect([nameWidth(false), rowHeight(false)]).toEqual([120, 34])
})
