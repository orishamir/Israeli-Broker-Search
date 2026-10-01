import type { BarSeriesOption } from 'echarts/charts'
import { expect, test } from 'vitest'
import { shekels } from './format'
import { lengthOf, splitOption, type Split, type SplitView } from './interest-split'

const fund: Split = {
  id: 'fund',
  label: 'Fund',
  color: '#56b4e9',
  hollow: false,
  yours: 2807,
  kept: 174,
  tax: 269,
}
// A bank that pays more than the rate: it keeps less than nothing.
const bank: Split = {
  id: 'bank',
  label: 'Bank',
  color: '#e69f00',
  hollow: false,
  yours: 3290,
  kept: -620,
  tax: 580,
}

const view = (changes: Partial<SplitView> = {}): SplitView => ({
  splits: [bank, fund],
  atTheRate: 3250,
  pinned: new Set(),
  width: 600,
  measure: (text) => text.length * 6,
  ...changes,
})

const bars = (option: ReturnType<typeof splitOption>) =>
  (option.series as BarSeriesOption[]).filter(({ stack }) => stack === 'interest')

type Labelled = BarSeriesOption & { label: { formatter: (params: { dataIndex: number }) => string } }
/** What a series writes on each row, top to bottom. */
const written = (option: ReturnType<typeof splitOption>, id: string) => {
  const series = (option.series as Labelled[]).find((each) => each.id === id)!
  return (series.data as unknown[]).map((_, dataIndex) => series.label.formatter({ dataIndex }))
}

test("a place that keeps something fills the rate's interest; one that pays more passes it", () => {
  expect(lengthOf(fund)).toBe(3250)
  expect(lengthOf(bank)).toBe(3870)
  const option = splitOption(view())
  expect(option.xAxis).toMatchObject({ max: 3870 })
  const [yours, kept, tax] = bars(option)
  const values = (series: BarSeriesOption) => (series.data as { value: number }[]).map(({ value }) => value)
  expect(values(yours)).toEqual([3290, 2807])
  // Less than nothing isn't drawn: the bar passes the line instead.
  expect(values(kept)).toEqual([0, 174])
  expect(values(tax)).toEqual([580, 269])
  // The line is the rate's interest.
  expect(yours.markLine).toMatchObject({ data: [{ xAxis: 3250 }] })
})

test("the saver's part is in each place's color, and its amount is the total beside the bar", () => {
  const option = splitOption(view())
  const [yours] = bars(option)
  expect((yours.data as { itemStyle: { color: string } }[]).map(({ itemStyle }) => itemStyle.color)).toEqual([
    '#e69f00',
    '#56b4e9',
  ])
  expect(yours.label).toMatchObject({ show: false })
  expect(written(option, 'totals')).toEqual([`{total|${shekels(3290)}}`, `{total|${shekels(2807)}}`])
})

test('names and totals ride on the bars, and every series keeps its id', () => {
  const option = splitOption(view({ pinned: new Set(['fund']) }))
  // So that a change glides rather than drawing the bars again from nothing.
  expect((option.series as BarSeriesOption[]).map(({ id }) => id)).toEqual([
    'outline',
    'names',
    'yours',
    'kept',
    'tax',
    'totals',
  ])
  expect(written(option, 'names')).toEqual(['{dot0|●} {name|Bank}', '{dot1|●} {pinned1|Fund}'])
  // The totals go past the rest of each row, which is clear: the bank's bar
  // is the longest.
  const totals = (option.series as BarSeriesOption[]).find(({ id }) => id === 'totals')!
  expect(totals.data).toEqual([0, 3870 - 3250])
  expect(option.yAxis).toMatchObject({ data: ['bank', 'fund'], axisLabel: { show: false } })
})

test('once a place is pinned, the others fade a little', () => {
  const [yours] = bars(splitOption(view({ pinned: new Set(['fund']) })))
  expect(
    (yours.data as { itemStyle: { opacity: number } }[]).map(({ itemStyle }) => itemStyle.opacity),
  ).toEqual([0.6, 1])
})
