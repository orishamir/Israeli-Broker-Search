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
  const [, totals] = option.yAxis as { data: string[] }[]
  expect(totals.data).toEqual([shekels(3290), shekels(2807)])
})

test('once a place is pinned, the others fade a little', () => {
  const [yours] = bars(splitOption(view({ pinned: new Set(['fund']) })))
  expect(
    (yours.data as { itemStyle: { opacity: number } }[]).map(({ itemStyle }) => itemStyle.opacity),
  ).toEqual([0.6, 1])
})
