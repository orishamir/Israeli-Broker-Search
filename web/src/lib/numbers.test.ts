import { expect, test } from 'vitest'
import { formatNumber, parseNumber, stepped } from './numbers'

test('amounts show thousands separators and up to four decimals', () => {
  expect(formatNumber(10_000)).toBe('10,000')
  expect(formatNumber(3.0338)).toBe('3.0338')
  expect(formatNumber(0.123456)).toBe('0.1235')
  expect(formatNumber(null)).toBe('')
})

test('typed text is read with or without commas, and only as a number', () => {
  expect(parseNumber('25,000')).toBe(25_000)
  expect(parseNumber(' 2000 ')).toBe(2000)
  expect(parseNumber('0.5')).toBe(0.5)
  expect(parseNumber('-5')).toBe(-5)
  expect(parseNumber('')).toBeNull()
  expect(parseNumber('abc')).toBeNull()
  expect(parseNumber('1e999')).toBeNull()
})

test('the arrow keys step without floating-point drift', () => {
  expect(stepped(0.2, 0.1, 1)).toBe(0.3)
  expect(stepped(10_000, 1000, 1)).toBe(11_000)
  expect(stepped(null, 100, -1)).toBe(-100)
})
