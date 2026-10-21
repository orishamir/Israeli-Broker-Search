import { expect, test } from 'vitest'
import type { GainPartKind, GainRowData } from './core/core'
import { GAIN_COLORS, gainBars, partText } from './gain'

const amount = (today: number) => ({ today, nominal: today * 1.8 })
const names: Record<GainPartKind, string> = { Kept: 'נשאר לכם', Tax: 'מס', Fees: 'עמלות', Fund: 'הקרן עצמה' }
const row = (name: string, kept: number, tax: number, fees: number, fund: number): GainRowData => {
  const values: [GainPartKind, number][] = [
    ['Kept', kept],
    ['Tax', tax],
    ['Fees', fees],
    ['Fund', fund],
  ]
  return {
    name,
    kept: amount(kept),
    tax: amount(tax),
    fees: amount(fees),
    product: amount(fund),
    parts: values
      .filter(([, value]) => value !== 0)
      .map(([which, value]) => ({
        which,
        label: names[which],
        value: amount(value),
        amount: `₪${Math.round(value).toLocaleString('en-US')}`,
      })),
    note: undefined,
  }
}
/** About as wide as the page's 12 px words. */
const measure = (text: string) => text.length * 6.5 + 6

const salary = [
  row('קרן סל על מדד מניות רחב', 292_000, 92_600, 2_090, 21_000),
  row('קרן השתלמות, במסלול מניות', 333_000, 0, 27_100, 21_100),
  row('קרן מחקה, הזול מבין הבנקים', 203_000, 74_900, 43_300, 17_500),
]

test('every bar is on the scale of the biggest gain, in today’s money', () => {
  const [wide, narrow] = gainBars(
    [row('a', 290_000, 92_000, 2_500, 21_000), row('b', 200_000, 74_000, 45_000, 17_000)],
    500,
    measure,
  )
  const width = (bar: typeof wide) => bar.segments.reduce((sum, { width }) => sum + width, 0)
  expect(width(wide)).toBeCloseTo(1)
  expect(width(narrow)).toBeCloseTo(
    (200_000 + 74_000 + 45_000 + 17_000) / (290_000 + 92_000 + 2_500 + 21_000),
  )
  expect(wide.segments[0].value).toBe(290_000)
  // Parts follow each other.
  for (const [i, segment] of wide.segments.slice(1).entries())
    expect(segment.from).toBeCloseTo(wide.segments[i].from + wide.segments[i].width)
})

test('every part is named with its amount, on it or beside it, never covering another', () => {
  for (const width of [288, 320, 343, 500, 700]) {
    for (const bar of gainBars(salary, width, measure)) {
      const where = `${bar.name} at ${width}px`
      for (const segment of bar.segments) {
        const named = segment.inside || bar.beside.some(({ which }) => which === segment.which)
        expect(named, `${where}: ${segment.which}`).toBe(true)
        if (segment.inside) expect(measure(segment.text)).toBeLessThanOrEqual(segment.width * width)
        expect(segment.text).toMatch(/₪/)
      }
      for (const [i, a] of bar.beside.entries()) {
        expect(a.x, where).toBeGreaterThanOrEqual(0)
        expect(a.x + a.width, where).toBeLessThanOrEqual(width + 1e-9)
        for (const b of bar.beside.slice(i + 1))
          if (a.lane === b.lane) expect(a.x + a.width <= b.x || b.x + b.width <= a.x, where).toBe(true)
      }
    }
  }
})

test('the fund’s own cost has a colour of its own, and so does every part', () => {
  expect(new Set(Object.values(GAIN_COLORS)).size).toBe(4)
})

test('a part the bar can’t draw is still named', () => {
  const [bar] = gainBars([row('a', -500, 0, 300, 100)], 400, measure)
  expect(bar.segments.every(({ width }) => width >= 0 && width <= 1)).toBe(true)
  expect(bar.segments.map(({ which }) => which)).toEqual(['Fees', 'Fund'])
  expect(bar.undrawn).toEqual([partText({ label: names.Kept, amount: '₪-500' })])
})
