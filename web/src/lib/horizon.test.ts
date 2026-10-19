import { expect, test } from 'vitest'
import { defaultUnit, monthsOfTyped, shown } from './horizon'

test('whole years read in years, anything else in months', () => {
  expect(defaultUnit(60)).toBe('years')
  expect(defaultUnit(18)).toBe('months')
  expect(defaultUnit(undefined)).toBe('years')
})

test('the field shows the months in its unit and reads them back', () => {
  expect(shown(60, 'years')).toBe(5)
  expect(shown(18, 'years')).toBe(1.5)
  expect(shown(18, 'months')).toBe(18)
  expect(shown(undefined, 'years')).toBeNull()
  expect(monthsOfTyped(5, 'years')).toBe(60)
  expect(monthsOfTyped(1.5, 'years')).toBe(18)
  expect(monthsOfTyped(7, 'months')).toBe(7)
  expect(monthsOfTyped(null, 'months')).toBeNull()
})
