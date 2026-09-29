import type { LineSeriesOption } from 'echarts/charts'
import { expect, test } from 'vitest'
import { growthOption, NO_FEES, type GrowthView, type Line } from './growth-chart'

const line = (id: string, dotted = false): Line => ({
  id,
  label: `Plan ${id}`,
  color: '#56b4e9',
  dotted,
  values: [100, 110, 120, 130],
})

const view = (changes: Partial<GrowthView> = {}): GrowthView => ({
  lines: [line('a'), line('b', true)],
  noFees: [100, 120, 140, 160],
  lost: false,
  pinned: new Set(),
  touch: false,
  ...changes,
})

const series = (option: ReturnType<typeof growthOption>) => option.series as LineSeriesOption[]

test('every line is a series by its id, a month apart in years, with the no-fee line first', () => {
  const [noFees, a, b] = series(growthOption(view()))
  expect(noFees.id).toBe(NO_FEES)
  expect(noFees.silent).toBe(true)
  expect(noFees.lineStyle?.type).toBe('dashed')
  expect([a.id, b.id]).toEqual(['a', 'b'])
  expect(a.data).toEqual([
    [0, 100],
    [1 / 12, 110],
    [2 / 12, 120],
    [3 / 12, 130],
  ])
  expect(a.lineStyle?.type).toBe('solid')
  expect(b.lineStyle?.type).toBe('dotted')
})

test('the lost view has no no-fee line and ranks least first', () => {
  const option = growthOption(view({ lost: true, noFees: null }))
  expect(series(option).map(({ id }) => id)).toEqual(['a', 'b'])
  expect(option.tooltip).toMatchObject({ order: 'valueAsc' })
  expect(growthOption(view()).tooltip).toMatchObject({ order: 'valueDesc' })
})

test('pinned lines are wider, labelled at whole years, and the rest fade', () => {
  const [, a, b] = series(growthOption(view({ pinned: new Set(['a']) })))
  expect(a.lineStyle).toMatchObject({ width: 3.5, opacity: 1 })
  expect(b.lineStyle).toMatchObject({ width: 2, opacity: 0.4 })
  expect(b.endLabel).toMatchObject({ opacity: 0.6 })
  const points = a.data as { symbol: string; label: { show: boolean } }[]
  expect(points.map(({ symbol }) => symbol)).toEqual(['emptyCircle', 'none', 'none', 'none'])
  expect(points[0].label.show).toBe(true)
  expect(a.showSymbol).toBe(true)
  expect(b.showSymbol).toBe(false)
})

test('pinned labels alternate above and below, in the order of the lines', () => {
  const [, a, b] = series(growthOption(view({ pinned: new Set(['b', 'a']) })))
  expect(a.label?.position).toBe('top')
  expect(b.label?.position).toBe('bottom')
})

test('touch screens zoom with a slider, the rest with the wheel', () => {
  const zoom = (touch: boolean) => (growthOption(view({ touch })).dataZoom as { type: string }[])[0].type
  expect(zoom(false)).toBe('inside')
  expect(zoom(true)).toBe('slider')
  expect(growthOption(view({ touch: true })).grid).toMatchObject({ bottom: 88 })
})

test('on a phone, labels that would overlap are hidden rather than moved apart', () => {
  const layout = (touch: boolean) =>
    series(growthOption(view({ touch, pinned: new Set(['a']) })))[1].labelLayout
  expect(layout(false)).toEqual({ moveOverlap: 'shiftY', hideOverlap: true })
  expect(layout(true)).toEqual({ hideOverlap: true })
})

test('the value axis fits the lines with a little room above', () => {
  const yAxis = growthOption(view()).yAxis as { max: (range: { min: number; max: number }) => number }
  expect(yAxis.max({ min: 100, max: 200 })).toBe(204)
})
