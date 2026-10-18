import { expect, test } from 'vitest'
import { aroundWords } from './around'
import { t } from './text'
import type { CrossingData, PlanKey } from './core/core'

const altshuler: PlanKey = { kind: 'listed', broker: 0, plan: 1 }
const meitav: PlanKey = { kind: 'listed', broker: 4, plan: 1 }
const labelOf = (key: PlanKey) => (key === altshuler ? 'אלטשולר · מבצע הצטרפות' : 'מיטב · מבצע הצטרפות')
const monthly = (below?: CrossingData, above?: CrossingData) =>
  aroundWords({ from: 100, to: 32_000, below, above }, 'Monthly', labelOf)

// Which of the line's sentences, with which amounts and plans: the words
// themselves are the text module's.
test('the best plan at other deposits, in words', () => {
  const [altshulerLabel, meitavLabel] = [labelOf(altshuler), labelOf(meitav)]
  expect(monthly()).toBe(t.cheapestThroughout('₪100', '₪32,000', 'Monthly'))
  expect(monthly({ amount: 600, key: altshuler })).toBe(t.cheaperBelow('₪600', altshulerLabel, 'Monthly'))
  expect(monthly(undefined, { amount: 8_000, key: meitav })).toBe(
    t.cheaperAbove('₪8,000', meitavLabel, 'Monthly'),
  )
  expect(monthly({ amount: 600, key: altshuler }, { amount: 8_000, key: meitav })).toBe(
    t.cheaperBothWays('₪600', altshulerLabel, '₪8,000', meitavLabel, 'Monthly'),
  )
  const once = { from: 1_000, to: 4_600_000, below: { amount: 50_000, key: altshuler }, above: undefined }
  expect(aroundWords(once, 'OneTime', labelOf)).toBe(t.cheaperBelow('₪50,000', altshulerLabel, 'OneTime'))
  expect(aroundWords({ ...once, below: undefined }, 'OneTime', labelOf)).toBe(
    t.cheapestThroughout('₪1,000', '₪4,600,000', 'OneTime'),
  )
})
