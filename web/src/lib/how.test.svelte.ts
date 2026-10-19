import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState } from './app.svelte'
import type { ProviderData } from './core/core'
import { howFor, placeholderParts, rangeText, tieGroups, versusText } from './how'
import { kindKey } from './kinds'
import { t } from './text'

beforeEach(() => {
  localStorage.clear()
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})

function start(): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState({})
  })
  flushSync()
  return app
}

const row = (name: string, left: number, tie: number, extra: Partial<ProviderData> = {}): ProviderData => ({
  name,
  left: { today: left, nominal: left },
  yours: false,
  tie,
  versusYours: undefined,
  warning: undefined,
  ...extra,
})

test('each row of the chart has its own how, and nothing is shown before a row is chosen', () => {
  const app = start()
  const answer = app.answer!
  expect(howFor(answer, undefined)).toBeUndefined()
  for (const chartRow of answer.outcome.rows) {
    expect(kindKey(howFor(answer, chartRow.id)!.row)).toBe(kindKey(chartRow.id))
  }
})

test('places that tie stay together, in the core’s order, and every place is shown once', () => {
  const rows = [row('a', 300, 0), row('b', 301, 0), row('c', 250, 1), row('d', 249, 1), row('e', 100, 2)]
  const groups = tieGroups(rows)
  expect(groups.map((group) => group.map((each) => each.name))).toEqual([['a', 'b'], ['c', 'd'], ['e']])
  expect(groups.flat()).toEqual(rows)
  expect(tieGroups([])).toEqual([])
})

test('a place is said against the visitor’s bank as a difference, with its sign', () => {
  const yours = { today: 100, nominal: 100 }
  expect(versusText(row('yours', 100, 1, { yours: true }))).toBe(t.referenceRow)
  expect(versusText(row('more', 2_994, 0, { versusYours: { ...yours, today: 2_994 } }))).toMatch(/^\+₪2,994$/)
  expect(versusText(row('less', 0, 0, { versusYours: { ...yours, today: -597 } }))).toBe('−₪597')
  expect(versusText(row('same', 0, 0, { versusYours: { ...yours, today: 0.3 } }))).toBe(t.sameAmount)
  // No bank named: no difference to say.
  expect(versusText(row('none', 0, 0))).toBe('')
})

test('the banks’ range reads low to high, and one figure when they agree', () => {
  expect(rangeText([189_130, 190_551])).toBe('₪189,130–₪190,551')
  expect(rangeText([5_000.2, 4_999.9])).toBe('₪5,000')
})

test('a slot the research hasn’t filled is marked, the rest is plain', () => {
  expect(placeholderParts('ממשלתי צמוד ____ (הסדרה)')).toEqual([
    { text: 'ממשלתי צמוד ', placeholder: false },
    { text: '____', placeholder: true },
    { text: ' (הסדרה)', placeholder: false },
  ])
  expect(placeholderParts('למשל [—] (דוגמה, לא המלצה)')[1]).toEqual({ text: '[—]', placeholder: true })
  expect(placeholderParts('כספית שקלית')).toEqual([{ text: 'כספית שקלית', placeholder: false }])
})
