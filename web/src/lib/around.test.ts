import { expect, test } from 'vitest'
import { aroundWords } from './around'
import type { CrossingData, PlanKey } from './core/core'

const altshuler: PlanKey = { kind: 'listed', broker: 0, plan: 1 }
const meitav: PlanKey = { kind: 'listed', broker: 4, plan: 1 }
const labelOf = (key: PlanKey) => (key === altshuler ? 'אלטשולר · מבצע הצטרפות' : 'מיטב · מבצע הצטרפות')
const monthly = (below?: CrossingData, above?: CrossingData) =>
  aroundWords({ from: 100, to: 32_000, below, above }, 'Monthly', labelOf)

test('the best plan at other deposits, in words', () => {
  expect(monthly()).toBe('הזול ביותר בעמלות בכל הפקדה חודשית מ\u2011₪100 עד ₪32,000')
  expect(monthly({ amount: 600, key: altshuler })).toBe(
    'בהפקדה חודשית של פחות מ\u2011₪600, זול יותר: אלטשולר · מבצע הצטרפות',
  )
  expect(monthly(undefined, { amount: 8_000, key: meitav })).toBe(
    'בהפקדה חודשית של יותר מ\u2011₪8,000, זול יותר: מיטב · מבצע הצטרפות',
  )
  expect(monthly({ amount: 600, key: altshuler }, { amount: 8_000, key: meitav })).toBe(
    'בהפקדה חודשית של פחות מ\u2011₪600, זול יותר: אלטשולר · מבצע הצטרפות; ביותר מ\u2011₪8,000: מיטב · מבצע הצטרפות',
  )
  const once = { from: 1_000, to: 4_600_000, below: { amount: 50_000, key: altshuler }, above: undefined }
  expect(aroundWords(once, 'OneTime', labelOf)).toBe(
    'בהפקדה חד\u2011פעמית של פחות מ\u2011₪50,000, זול יותר: אלטשולר · מבצע הצטרפות',
  )
  expect(aroundWords({ ...once, below: undefined }, 'OneTime', labelOf)).toBe(
    'הזול ביותר בעמלות בכל הפקדה חד\u2011פעמית מ\u2011₪1,000 עד ₪4,600,000',
  )
})
