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

test('spaces and a currency or percent sign typed or pasted in are ignored', () => {
  expect(parseNumber('50 000')).toBe(50_000)
  expect(parseNumber('15\u00a0000')).toBe(15_000)
  expect(parseNumber('₪15,000')).toBe(15_000)
  expect(parseNumber('$ 1,200')).toBe(1200)
  expect(parseNumber('3.25%')).toBe(3.25)
})

test('a lone comma before one or two digits is a decimal point; any other comma must group thousands', () => {
  expect(parseNumber('3,25')).toBe(3.25)
  expect(parseNumber('0,5')).toBe(0.5)
  expect(parseNumber('1,500,000')).toBe(1_500_000)
  expect(parseNumber('1,500.5')).toBe(1500.5)
  expect(parseNumber('1,50,000')).toBeNull()
  expect(parseNumber('1,5000')).toBeNull()
  expect(parseNumber('3,25.1')).toBeNull()
})

test('only plain decimals are numbers: no hex, exponents or words', () => {
  expect(parseNumber('0x10')).toBeNull()
  expect(parseNumber('1e6')).toBeNull()
  expect(parseNumber('Infinity')).toBeNull()
  expect(parseNumber('1.5k')).toBeNull()
  expect(parseNumber('5.')).toBe(5)
  expect(parseNumber('.5')).toBe(0.5)
  expect(parseNumber('-')).toBeNull()
})

test('the arrow keys step without floating-point drift', () => {
  expect(stepped(0.2, 0.1, 1)).toBe(0.3)
  expect(stepped(10_000, 1000, 1)).toBe(11_000)
  expect(stepped(null, 100, -1)).toBe(-100)
})
