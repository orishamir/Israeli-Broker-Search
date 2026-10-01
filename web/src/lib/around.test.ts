import { expect, test } from 'vitest'
import { aroundWords } from './around'
import type { CrossingData, PlanKey } from './core/core'

const altshuler: PlanKey = { kind: 'listed', broker: 0, plan: 1 }
const meitav: PlanKey = { kind: 'listed', broker: 4, plan: 1 }
const labelOf = (key: PlanKey) => (key === altshuler ? 'אלטשולר · לקוחות חדשים' : 'מיטב · מבצע הצטרפות')
const monthly = (below?: CrossingData, above?: CrossingData) =>
  aroundWords({ from: 100, to: 32_000, below, above }, 'Monthly', labelOf)

test('the best plan at other deposits, in words', () => {
  expect(monthly()).toBe('הזול ביותר בעמלות בכל הפקדה חודשית מ-₪100 עד ₪32,000')
  expect(monthly({ amount: 600, key: altshuler })).toBe(
    'בהפקדה חודשית של פחות מ-₪600, אלטשולר · לקוחות חדשים זול יותר',
  )
  expect(monthly(undefined, { amount: 8_000, key: meitav })).toBe(
    'בהפקדה חודשית של יותר מ-₪8,000, מיטב · מבצע הצטרפות זול יותר',
  )
  expect(monthly({ amount: 600, key: altshuler }, { amount: 8_000, key: meitav })).toBe(
    'בהפקדה חודשית של פחות מ-₪600, אלטשולר · לקוחות חדשים זול יותר; ביותר מ-₪8,000, מיטב · מבצע הצטרפות זול יותר',
  )
  const once = { from: 1_000, to: 4_600_000, below: { amount: 50_000, key: altshuler }, above: undefined }
  expect(aroundWords(once, 'OneTime', labelOf)).toBe(
    'בהפקדה חד\u2011פעמית של פחות מ-₪50,000, אלטשולר · לקוחות חדשים זול יותר',
  )
  expect(aroundWords({ ...once, below: undefined }, 'OneTime', labelOf)).toBe(
    'הזול ביותר בעמלות בכל הפקדה חד\u2011פעמית מ-₪1,000 עד ₪4,600,000',
  )
})
