import { expect, test } from 'vitest'
import { compactPercent, compactShekels, dateText, elapsed, percent, readableOn, shekels } from './format'

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
