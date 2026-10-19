import { expect, test } from 'vitest'
import type { LineSeriesOption } from 'echarts/charts'
import { BAND_HIGH, BAND_IDS, BAND_LOW, HEADROOM, alternativesOption, stockLine } from './alternatives-chart'
import { kindColor, KIND_COLORS } from './colors'
import type { AlternativesData, Kind, KindSeriesData } from './core/core'
import { kindKey } from './kinds'
import { t } from './text'

const index: Kind = { Way: { product: 'IsraeliIndexFund', vehicle: 'Brokerage', exchange: 'Tlv' } as never }
const gemel: Kind = { Way: { product: 'InvestmentGemelTrack', vehicle: 'InvestmentGemel' } as never }

const line = (kind: Kind, end: number, recommended = false): KindSeriesData => ({
  kind,
  name: typeof kind === 'string' ? kind : kindKey(kind),
  recommended,
  safety: 'SumKnown',
  safetyTag: '',
  safetyNote: '',
  card: undefined,
  arena: undefined,
  valueByMonth: [100, (100 + end) / 2, end],
  left: { today: end, nominal: end },
})

const data = (over: Partial<AlternativesData> = {}): AlternativesData => ({
  months: 24,
  series: [
    line('FixedDeposit', 110, true),
    line('MoneyMarketFund', 108),
    line(index, 130),
    line('CurrentAccount', 95),
  ],
  band: {
    band: {} as never,
    worst: [100, 80, 60],
    best: [100, 150, 400],
    sentence: 'sentence',
    caveat: 'caveat',
  },
  ...over,
})

const view = (d: AlternativesData, highlighted: string[] = []) => ({
  data: d,
  highlighted: new Set(highlighted),
  colors: new Map(d.series.map((each) => [kindKey(each.kind), kindColor(each.kind)])),
  touch: false,
  measure: (text: string) => text.length * 7,
})

const seriesOf = (d: AlternativesData, highlighted: string[] = []) =>
  alternativesOption(view(d, highlighted)).series as LineSeriesOption[]

test('every series has an id: the kind’s key, and the band’s own two', () => {
  const series = seriesOf(data())
  const ids = series.map(({ id }) => id)
  expect(ids.every((id) => typeof id === 'string' && id.length > 0)).toBe(true)
  expect(new Set(ids).size).toBe(ids.length)
  expect(ids).toEqual(
    expect.arrayContaining(['FixedDeposit', 'MoneyMarketFund', kindKey(index), 'CurrentAccount']),
  )
  expect(ids).toEqual(expect.arrayContaining([...BAND_IDS]))
})

test('one series per kind, and two more for the band when there is one', () => {
  expect(seriesOf(data())).toHaveLength(4 + 2)
  expect(seriesOf(data({ band: undefined }))).toHaveLength(4)
})

test('the band is two stacked series: the lower line transparent, the upper one with the area', () => {
  const series = seriesOf(data())
  const low = series.find(({ id }) => id === BAND_LOW)!
  const high = series.find(({ id }) => id === BAND_HIGH)!
  expect(low.stack).toBe(high.stack)
  expect(low.stack).toBeDefined()
  expect(low.lineStyle?.opacity).toBe(0)
  expect(low.areaStyle).toBeUndefined()
  expect(high.areaStyle).toBeDefined()
  // The upper series is what the best adds above the worst, so they stack to the best.
  expect((high.data as number[][]).map(([, value]) => value)).toEqual([0, 70, 340])
  expect((low.data as number[][]).map(([, value]) => value)).toEqual([100, 80, 60])
  // Not rows: nothing to hover or click.
  expect(low.silent && high.silent).toBe(true)
})

test('the band takes the color of the stock fund it is drawn behind', () => {
  const high = seriesOf(data()).find(({ id }) => id === BAND_HIGH)!
  expect((high.areaStyle as { color: string }).color).toBe(kindColor(index))
})

test('only the recommended kind, the stock fund and the current account have end labels', () => {
  const labelled = seriesOf(data()).filter(({ endLabel }) => endLabel?.show)
  expect(labelled.map(({ id }) => id).sort()).toEqual(
    ['CurrentAccount', 'FixedDeposit', kindKey(index)].sort(),
  )
  const word = (id: string) =>
    labelled.find((each) => each.id === id)!.endLabel!.formatter as unknown as string
  expect(word('FixedDeposit')).toBe(t.endRecommended)
  expect(word(kindKey(index))).toBe(t.endStock)
  expect(word('CurrentAccount')).toBe(t.endCash)
})

test('when the recommended kind is the stock fund, it has one label, not two', () => {
  const d = data({
    series: [line(index, 130, true), line('FixedDeposit', 110), line('CurrentAccount', 95)],
  })
  expect(seriesOf(d).filter(({ endLabel }) => endLabel?.show)).toHaveLength(2)
})

test('the band goes behind the first way of holding the index when none is recommended', () => {
  const lines = [line('FixedDeposit', 110, true), line(gemel, 120), line(index, 130)]
  expect(stockLine(lines)).toBe(lines[1])
  expect(stockLine([lines[0]])).toBeUndefined()
  const recommended = [line(gemel, 120), line(index, 130, true)]
  expect(stockLine(recommended)).toBe(recommended[1])
})

test('a label is never left to ECharts to wrap or cut: it is measured and fitted here', () => {
  const narrow = alternativesOption({ ...view(data()), measure: (text) => text.length * 40 })
  const labels = (narrow.series as LineSeriesOption[]).filter(({ endLabel }) => endLabel?.show)
  for (const each of labels) {
    const label = each.endLabel as { formatter: string; overflow?: string; width?: number }
    expect(label.overflow).toBeUndefined()
    expect(label.width).toBeUndefined()
    // Cut by us, with an ellipsis, to what the room holds.
    expect(typeof label.formatter).toBe('string')
    expect(label.formatter.length * 40).toBeLessThanOrEqual(76)
  }
})

test('every line is drawn in its kind’s color, the current account dashed', () => {
  const series = seriesOf(data())
  expect(series.find(({ id }) => id === 'FixedDeposit')!.color).toBe(KIND_COLORS.FixedDeposit)
  expect(series.find(({ id }) => id === kindKey(index))!.color).toBe(KIND_COLORS.IsraeliIndexFund)
  expect(series.find(({ id }) => id === 'CurrentAccount')!.lineStyle?.type).toBe('dashed')
  expect(series.find(({ id }) => id === 'FixedDeposit')!.lineStyle?.type).toBe('solid')
})

test('pinned kinds are drawn thicker and the others fade only a little', () => {
  const series = seriesOf(data(), ['MoneyMarketFund'])
  const pinned = series.find(({ id }) => id === 'MoneyMarketFund')!
  const other = series.find(({ id }) => id === kindKey(index))!
  expect(pinned.lineStyle?.width).toBeGreaterThan(other.lineStyle!.width as number)
  expect(pinned.lineStyle?.opacity).toBe(1)
  expect(other.lineStyle?.opacity).toBeGreaterThanOrEqual(0.4)
  expect(other.lineStyle?.opacity).toBeLessThan(1)
  // Nothing pinned: nothing fades.
  const lines = seriesOf(data()).filter(({ id }) => !BAND_IDS.includes(id as string))
  expect(lines.every(({ lineStyle }) => lineStyle?.opacity === 1)).toBe(true)
})

test('hovering is not in the options: a hovered row is the highlight action’s', () => {
  const a = JSON.stringify(alternativesOption(view(data())).series)
  expect(a).toBe(JSON.stringify(alternativesOption(view(data())).series))
  expect(Object.keys(view(data()))).not.toContain('hovered')
})

test('the chart reaches only a little above the highest line, and cuts the band off there', () => {
  const option = alternativesOption(view(data()))
  expect((option.yAxis as { max: number }).max).toBe(130 * HEADROOM)
  // The band's best case (400) is above it.
  expect((option.yAxis as { max: number }).max).toBeLessThan(400)
  expect((option.xAxis as { max: number }).max).toBe(2)
})

test('time is in years on a value axis, from the months', () => {
  const first = seriesOf(data())[2].data as number[][]
  expect(first.map(([time]) => time)).toEqual([0, 1 / 12, 2 / 12])
})

// A four-month cushion on a years axis had one tick (0), and its tooltip
// read "אחרי 0 שנים ו‑4 חודשים".
test('a horizon under two years is drawn in months, with ticks to read them by', () => {
  const option = alternativesOption(view({ ...data(), months: 4 }))
  const x = option.xAxis as { name: string; max: number; axisLabel: { formatter: (n: number) => string } }
  expect(x.name).toBe(t.chartMonths)
  expect(x.max).toBe(4)
  expect([0, 1, 2, 3, 4].map(x.axisLabel.formatter)).toEqual(['0', '1', '2', '3', '4'])
  const deposit = (option.series as LineSeriesOption[]).find(({ id }) => id === 'FixedDeposit')!
  expect((deposit.data as number[][]).map(([time]) => time)).toEqual([0, 1, 2])
  // Two years and over stay in years.
  expect((alternativesOption(view(data())).xAxis as { name: string }).name).toBe(t.chartYears)
})

type Formatter = (rows: unknown) => string
const tooltipAt = (d: AlternativesData, time: number) => {
  const option = alternativesOption(view(d))
  const row = { seriesId: 'FixedDeposit', seriesName: '', color: '#fff', value: [time, 100] }
  return (option.tooltip as { formatter: Formatter }).formatter([row])
}

test('the tooltip says the time in months under a year, never "0 years and 4 months"', () => {
  const text = tooltipAt({ ...data(), months: 4 }, 1)
  expect(text).toContain(t.after(0, 1))
  expect(text).not.toContain('0 שנים')
})

test('the last month’s tooltip says it is after selling and tax; the others do not', () => {
  const d = { ...data(), months: 4 }
  expect(tooltipAt(d, 4)).toContain(t.lastPointAfterTax)
  expect(tooltipAt(d, 3)).not.toContain(t.lastPointAfterTax)
  // In years: the last month of two.
  expect(tooltipAt(data(), 2)).toContain(t.lastPointAfterTax)
  expect(tooltipAt(data(), 1)).not.toContain(t.lastPointAfterTax)
})
