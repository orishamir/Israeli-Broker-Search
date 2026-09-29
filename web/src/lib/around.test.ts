import { expect, test } from 'vitest'
import { aroundWords } from './around'
import type { CrossingData, PlanKey } from './core/core'

const altshuler: PlanKey = { kind: 'listed', broker: 0, plan: 1 }
const meitav: PlanKey = { kind: 'listed', broker: 4, plan: 1 }
const labelOf = (key: PlanKey) => (key === altshuler ? 'Altshuler · New customers' : 'Meitav · Typical offer')
const monthly = (below?: CrossingData, above?: CrossingData) =>
  aroundWords({ from: 100, to: 32_000, below, above }, 'Monthly', labelOf)

test('the best plan at other deposits, in words', () => {
  expect(monthly()).toBe('The cheapest at any monthly deposit from ₪100 to ₪32,000')
  expect(monthly({ amount: 600, key: altshuler })).toBe(
    'Below ₪600 a month, Altshuler · New customers is cheaper',
  )
  expect(monthly(undefined, { amount: 8_000, key: meitav })).toBe(
    'Above ₪8,000 a month, Meitav · Typical offer is cheaper',
  )
  expect(monthly({ amount: 600, key: altshuler }, { amount: 8_000, key: meitav })).toBe(
    'Below ₪600 a month, Altshuler · New customers is cheaper; above ₪8,000, Meitav · Typical offer',
  )
  const once = { from: 1_000, to: 4_600_000, below: { amount: 50_000, key: altshuler }, above: undefined }
  expect(aroundWords(once, 'OneTime', labelOf)).toBe(
    'For a one-time deposit below ₪50,000, Altshuler · New customers is cheaper',
  )
  expect(aroundWords({ ...once, below: undefined }, 'OneTime', labelOf)).toBe(
    'The cheapest for any one-time deposit from ₪1,000 to ₪4,600,000',
  )
})
