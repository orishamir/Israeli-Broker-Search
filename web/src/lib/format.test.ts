import { expect, test } from 'vitest'
import {
  compactPercent,
  compactShekels,
  compactShekelsApart,
  dateText,
  elapsed,
  percent,
  percentApart,
  readableOn,
  shekels,
} from './format'

test('shekels are whole, with separators; compact ones fit chart labels', () => {
  expect(shekels(1_415_944.4)).toBe('₪1,415,944')
  expect(shekels(0)).toBe('₪0')
  expect(compactShekels(1_415_944)).toBe('₪1.42M')
  expect(compactShekels(889_000)).toBe('₪889K')
  expect(compactShekels(42_000)).toBe('₪42K')
})

test('black on light colors, white on dark', () => {
  expect(readableOn('#c1e319')).toBe('#000')
  expect(readableOn('#8d73f0')).toBe('#fff')
})

test("a chart's time is in years and months", () => {
  expect(elapsed(0)).toBe('אחרי 0 שנים')
  expect(elapsed(1)).toBe('אחרי שנה')
  expect(elapsed(15 + 5 / 12)).toBe('אחרי 15 שנים ו-5 חודשים')
  expect(elapsed(2 + 1 / 12)).toBe('אחרי שנתיים וחודש')
  expect(elapsed(3 + 2 / 12)).toBe('אחרי 3 שנים וחודשיים')
})

test('a date reads day first, as Israelis write it', () => {
  expect(dateText('2026-09-28')).toBe('28/09/2026')
  expect(dateText('2026-01-05')).toBe('05/01/2026')
})

test('a yearly cost has two decimals; on an axis, two digits', () => {
  expect(percent(0.4213)).toBe('0.42%')
  expect(percent(12.5)).toBe('12.50%')
  expect(percent(100)).toBe('100.00%')
  expect(compactPercent(0.1)).toBe('0.1%')
  expect(compactPercent(1)).toBe('1%')
  expect(compactPercent(12.5)).toBe('13%')
  expect(compactPercent(0.4213)).toBe('0.42%')
})

test('labels side by side get the digits that tell them apart, and no more', () => {
  // Lines ending ₪103,196, ₪102,930 and ₪103,188 all read "₪103K" at three
  // digits, and two still read "₪103.2K" at four.
  const short = compactShekelsApart([103_196, 102_930, 103_188])
  expect([103_196, 102_930, 103_188].map(short)).toEqual(['₪103.20K', '₪102.93K', '₪103.19K'])
  // Apart already at three: as compactShekels writes them.
  const apart = compactShekelsApart([1_415_944, 889_000])
  expect([1_415_944, 889_000].map(apart)).toEqual(['₪1.42M', '₪889K'])
  // Equal amounts stay equal, rather than reaching for digits that can't help.
  expect(compactShekelsApart([42_000, 42_000])(42_000)).toBe('₪42K')
  // Yearly costs: a decimal more where "0.11%" repeats.
  const costs = percentApart([0.112, 0.114, 0.98])
  expect([0.112, 0.114, 0.98].map(costs)).toEqual(['0.112%', '0.114%', '0.980%'])
  expect(percentApart([0.42, 1])(0.42)).toBe(percent(0.42))
})
