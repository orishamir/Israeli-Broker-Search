import { expect, test } from 'vitest'
import type { PieSeriesOption } from 'echarts/charts'
import type { BreakdownData } from './core/core'
import { DONUT_COLORS, donutOption, donutParts, sharePercent, tinyList } from './donut'
import { PART_COLORS } from './colors'
import { t } from './text'

const breakdown = (over: Partial<BreakdownData> = {}): BreakdownData => ({
  putIn: 50000,
  fees: 60,
  keptByThePlace: 0,
  tax: 1625,
  left: 59208,
  ...over,
})

const pie = (b: BreakdownData) => donutOption({ breakdown: b, touch: false }).series as PieSeriesOption[]

test('the parts are the deposits, the gain kept, the fees and the tax, in the short words', () => {
  const parts = donutParts(breakdown())
  expect(parts.map(({ id }) => id)).toEqual(['putIn', 'gain', 'fees', 'tax'])
  expect(parts.map(({ name }) => name)).toEqual([t.donutDeposit, t.donutGain, t.donutFees, t.donutTax])
  expect(parts.find(({ id }) => id === 'gain')!.value).toBe(9208)
  expect(parts.reduce((sum, { share }) => sum + share, 0)).toBeCloseTo(1)
})

test('what the place keeps is a part only when it keeps something', () => {
  expect(donutParts(breakdown()).some(({ id }) => id === 'kept')).toBe(false)
  // A bank paying above the rate keeps nothing: the core says it as a minus.
  expect(donutParts(breakdown({ keptByThePlace: -1456 })).some(({ id }) => id === 'kept')).toBe(false)
  const kept = donutParts(breakdown({ keptByThePlace: 800 })).find(({ id }) => id === 'kept')!
  expect(kept.value).toBe(800)
})

test('a loss after fees and tax leaves no gain slice, not a negative one', () => {
  const parts = donutParts(breakdown({ left: 45000 }))
  expect(parts.some(({ id }) => id === 'gain')).toBe(false)
  expect(parts.every(({ value }) => value > 0)).toBe(true)
})

test('every data item has an id, and the series has one', () => {
  const [series] = pie(breakdown())
  expect(series.id).toBeDefined()
  for (const item of series.data as { id: string }[]) expect(item.id).toBeTruthy()
})

test('labels are never wrapped or cut by ECharts', () => {
  const [series] = pie(breakdown())
  expect(series.label?.overflow).toBe('none')
})

test('a slice under 0.5% is not drawn, and is listed under the donut instead', () => {
  const b = breakdown({ fees: 100, left: 59208 })
  const [series] = pie(b)
  const names = (series.data as { id: string }[]).map(({ id }) => id)
  expect(names).not.toContain('fees')
  expect(tinyList(donutParts(b))).toBe(`${t.donutFees} ${sharePercent(100 / 60933)}`)
  // Nothing tiny: nothing listed.
  expect(tinyList(donutParts(breakdown({ fees: 3000 })))).toBe('')
})

test('percents: a whole number, one decimal under 1%', () => {
  expect(sharePercent(0.1234)).toBe('12%')
  expect(sharePercent(0.0034)).toBe('0.3%')
})

test('each part has one color, from the interest split’s where they are the same parts', () => {
  expect(DONUT_COLORS.gain).toBe(PART_COLORS.yours)
  expect(DONUT_COLORS.tax).toBe(PART_COLORS.tax)
  expect(DONUT_COLORS.kept).toBe(PART_COLORS.kept)
  expect(new Set(Object.values(DONUT_COLORS)).size).toBe(5)
  const [series] = pie(breakdown())
  const colors = (series.data as { id: keyof typeof DONUT_COLORS; itemStyle: { color: string } }[]).map(
    ({ id, itemStyle }) => [id, itemStyle.color],
  )
  for (const [id, color] of colors) expect(color).toBe(DONUT_COLORS[id as keyof typeof DONUT_COLORS])
})
