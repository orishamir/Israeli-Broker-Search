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
  MARGIN,
  overTimeOption,
  stacking,
  TOTAL_GAP,
  type Bar,
  type Focus,
} from './fee-breakdown'

const fees = (purchases: number, holding: number): FeeAmounts => ({
  purchases,
  conversions: 0,
  holding,
  product: 0,
  selling: 0,
  total: purchases + holding,
})

const bars: Bar[] = [
  { id: 'a', label: 'A', color: '#111111', hollow: false, fees: fees(500, 100) },
  { id: 'b', label: 'B', color: '#222222', hollow: true, fees: fees(100, 300) },
]

/** Six pixels a character, standing in for the canvas. */
const measure = (text: string) => [...text].length * 6

type Labelled = BarSeriesOption & {
  type: string
  label: { formatter: (params: { dataIndex: number }) => string }
}
const seriesOf = (option: ReturnType<typeof barsOption>) => option.series as Labelled[]
/** The name the chart writes over each bar, top to bottom. */
const names = (option: ReturnType<typeof barsOption>) => {
  const series = seriesOf(option).find(({ id }) => id === 'names')!
  return (series.data as unknown[]).map((_, dataIndex) => series.label.formatter({ dataIndex }))
}

test('bars are sorted by the total, or by the focused fee', () => {
  expect(byFee(bars, 'all').map(({ id }) => id)).toEqual(['b', 'a'])
  expect(byFee(bars, 'holding').map(({ id }) => id)).toEqual(['a', 'b'])
  expect(byFee(bars, 'purchases').map(({ id }) => id)).toEqual(['b', 'a'])
})

test('a focused fee is stacked first and keeps its color; the rest fade', () => {
  expect(stacking('all')).toBe(FEE_TYPES)
  expect(stacking('holding').map(({ key }) => key)).toEqual([
    'holding',
    'purchases',
    'conversions',
    'product',
    'selling',
  ])
  const holding = FEE_TYPES.find(({ key }) => key === 'holding')!
  const purchases = FEE_TYPES.find(({ key }) => key === 'purchases')!
  expect(colorOf(holding, 'holding')).toBe(holding.color)
  expect(colorOf(purchases, 'holding')).toBe(FADED)
  expect(colorOf(purchases, 'all')).toBe(purchases.color)
})

test('the bars name each plan, marking pinned ones, with the totals at the end', () => {
  const option = barsOption({ bars, focus: 'all', pinned: new Set(['b']), width: 800, measure })
  expect(option.yAxis).toMatchObject({ data: ['a', 'b'], axisLabel: { show: false } })
  // The names and totals ride on the bars, so they move with them when the
  // order changes: the names on the slots above the bars, the totals past
  // the rest of each row, which is clear.
  expect(names(option)).toEqual(['{dot0|●} {name|A}', '{dot1|◯} {pinned1|B}'])
  const series = seriesOf(option)
  const totals = series.find(({ id }) => id === 'totals')!
  expect(totals.data).toEqual([0, 200])
  expect([0, 1].map((dataIndex) => totals.label.formatter({ dataIndex }))).toEqual([
    '{total|₪600}',
    '{total|₪400}',
  ])
  expect(option.grid).toMatchObject({ right: MARGIN.right + measure('₪600') + TOTAL_GAP })
  // The outline, the slots for the names, a stacked series per fee, each
  // faded for unpinned plans, then the rest of the row.
  expect(series.map(({ type }) => type)).toEqual(['custom', 'bar', ...FEE_TYPES.map(() => 'bar'), 'bar'])
  expect(series[2].data).toEqual([
    { value: 500, itemStyle: { opacity: 0.6 } },
    { value: 100, itemStyle: { opacity: 1 } },
  ])
  expect(series.slice(2, -1).map(({ name }) => name)).toEqual(FEE_TYPES.map(({ name }) => name))
})

test('every series keeps its id, so a change glides instead of drawing the bars again', () => {
  const ids = (focus: Focus) =>
    seriesOf(barsOption({ bars, focus, pinned: new Set(), width: 800, measure })).map(({ id }) => id)
  const places = FEE_TYPES.map((_, place) => `fee ${place}`)
  // By place in the stack, whichever fee is first.
  expect(ids('all')).toEqual(['outline', 'names', ...places, 'totals'])
  expect(ids('holding')).toEqual(ids('all'))
  const overTime = overTimeOption([fees(1, 1)], 'holding').series as { id: string }[]
  expect(overTime.map(({ id }) => id)).toEqual(places)
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
  const name = (width: number) =>
    names(barsOption({ bars: long, focus: 'all', pinned: new Set(), width, measure }))[0]
  expect(name(800)).toBe('{dot0|●} {name|A plan of your own with a long name}')
  expect(name(150)).toBe('{dot0|●} {name|A plan of your own…}')
})

test('amounts show inside a part only where they fit', () => {
  const formatter = (width: number, focus: 'all' | 'holding') => {
    const option = barsOption({ bars, focus, pinned: new Set(), width, measure })
    const purchases = (option.series as BarSeriesOption[]).find(({ name }) => name === 'קניות')!
    return purchases.label!.formatter as (params: { value: number }) => string
  }
  // 800 px wide, 760 for ₪600 of bars beside the totals: ₪500 is about 633
  // px, ₪20 about 25.
  expect(formatter(800, 'all')({ value: 500 })).toBe('₪500')
  expect(formatter(800, 'all')({ value: 20 })).toBe('')
  // Faded parts show nothing.
  expect(formatter(800, 'holding')({ value: 500 })).toBe('')
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
