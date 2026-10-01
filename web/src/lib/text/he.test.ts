import { expect, test } from 'vitest'
import { he } from './he'

test('one and two years and months have words of their own', () => {
  expect(he.overYears(1, false)).toBe('לאורך שנה')
  expect(he.overYears(2, false)).toBe('לאורך שנתיים')
  expect(he.overYears(20, true)).toBe('לאורך 20 שנים, בכסף של היום')
  expect(he.after(1, 0)).toBe('אחרי שנה')
  expect(he.after(2, 1)).toBe('אחרי שנתיים וחודש')
  expect(he.after(15, 5)).toBe('אחרי 15 שנים ו-5 חודשים')
  expect(he.monthsCount(2)).toBe('חודשיים')
  expect(he.forMonths(1)).toBe('לחודש אחד')
})
