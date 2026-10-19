import { expect, test } from 'vitest'
import type { GainRowData } from './core/core'
import { gainBars, GAIN_PARTS } from './gain'

const amount = (today: number) => ({ today, nominal: today * 1.8 })
const row = (name: string, kept: number, tax: number, fees: number, product: number): GainRowData => ({
  name,
  kept: amount(kept),
  tax: amount(tax),
  fees: amount(fees),
  product: amount(product),
})

test('every bar is on the scale of the biggest gain, in today’s money', () => {
  const [wide, narrow] = gainBars([
    row('a', 290_000, 92_000, 2_500, 21_000),
    row('b', 200_000, 74_000, 45_000, 17_000),
  ])
  const width = (bar: typeof wide) => bar.segments.reduce((sum, { width }) => sum + width, 0)
  expect(width(wide)).toBeCloseTo(1)
  expect(width(narrow)).toBeCloseTo(
    (200_000 + 74_000 + 45_000 + 17_000) / (290_000 + 92_000 + 2_500 + 21_000),
  )
  expect(wide.segments[0].value).toBe(290_000)
})

test('an empty part is named under its bar but not drawn', () => {
  const [bar] = gainBars([row('study fund', 333_000, 0, 27_000, 21_000)])
  expect(bar.parts.map(({ id }) => id)).toEqual(GAIN_PARTS)
  expect(bar.segments.map(({ id }) => id)).toEqual(['kept', 'fees', 'product'])
})

test('a gain below the deposits draws no negative part', () => {
  const [bar] = gainBars([row('a', -500, 0, 300, 100)])
  expect(bar.segments.every(({ width }) => width >= 0 && width <= 1)).toBe(true)
  expect(bar.segments.map(({ id }) => id)).toEqual(['fees', 'product'])
})
