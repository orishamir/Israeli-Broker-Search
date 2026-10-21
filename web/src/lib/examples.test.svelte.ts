import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState } from './app.svelte'
import { decode } from './link'
import { fieldsOf, numbersLine } from './examples'

// The questions card's examples (REDESIGN.md §4.2, principles 19 and 22):
// an example fills only the fields on screen, is marked exactly while they
// hold its numbers, and trying one never loses the visitor's own.

beforeEach(() => {
  localStorage.clear()
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})

function start(shared = {}): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState(shared)
  })
  flushSync()
  return app
}

/** The visitor's own numbers, typed over the first visit's example. */
function typeYours(app: AppState) {
  app.answers.now = 12_345
  app.answers.monthly = 678
  app.setWhen('retirement')
  app.setHorizonRetirement(38, 62)
  app.setBank('Leumi')
  app.choose({ Flexibility: 'CanWait' })
  flushSync()
}

test('the first visit holds an example, marked, with nothing to go back to', () => {
  const app = start()
  expect(app.exampleOnScreen?.id).toBe('goal')
  expect(app.canGoBack).toBe(false)
})

test('changing a field unmarks the example, and changing it back marks it again', () => {
  const app = start()
  const monthly = app.answers.monthly
  app.answers.monthly = 2001
  flushSync()
  expect(app.exampleOnScreen).toBeUndefined()
  app.answers.monthly = monthly
  flushSync()
  expect(app.exampleOnScreen?.id).toBe('goal')
  // The unit the date is shown in isn't a number: still the example.
  app.horizonUnit = 'months'
  flushSync()
  expect(app.exampleOnScreen?.id).toBe('goal')
  // Nor is the date's field, while "מתי" says another answer.
  app.setWhen('unknown')
  flushSync()
  expect(app.exampleOnScreen).toBeUndefined()
  app.setWhen('date')
  flushSync()
  expect(app.exampleOnScreen?.id).toBe('goal')
})

test('picking examples keeps the visitor’s numbers, and going back restores them exactly', () => {
  const app = start()
  typeYours(app)
  expect(app.exampleOnScreen).toBeUndefined()
  // The date's field, hidden while retirement is chosen, is kept too.
  app.setWhen('date')
  app.setHorizonMonths(30)
  app.setWhen('retirement')
  flushSync()
  const yours = $state.snapshot(app.answers)
  const choices = $state.snapshot(app.choices)
  const fields = $state.snapshot(app.fields)
  for (const id of ['salary', 'cushion', 'retirement', 'existing']) {
    app.pickExample(id)
    flushSync()
    expect(app.exampleOnScreen?.id).toBe(id)
    expect(app.canGoBack).toBe(true)
    // The bank is the visitor's, not the example's.
    expect(app.answers.bank).toBe('Leumi')
  }
  app.backToYours()
  flushSync()
  expect(app.answers).toEqual(yours)
  expect(app.choices).toEqual(choices)
  expect(app.fields).toEqual(fields)
  expect(app.datedMonths).toBe(30)
  expect(app.exampleOnScreen).toBeUndefined()
  expect(app.canGoBack).toBe(false)
})

test('a field changed over an example makes the numbers the visitor’s: nothing to go back to', () => {
  const app = start()
  typeYours(app)
  app.pickExample('bonus')
  flushSync()
  app.answers.now = 16_000
  flushSync()
  expect(app.exampleOnScreen).toBeUndefined()
  expect(app.canGoBack).toBe(false)
  // And back to the example's numbers: an example again, with nothing kept.
  app.answers.now = 15_000
  flushSync()
  expect(app.exampleOnScreen?.id).toBe('bonus')
  expect(app.canGoBack).toBe(false)
})

test('going back keeps the bank as chosen since', () => {
  const app = start()
  typeYours(app)
  app.pickExample('salary')
  app.setBank('Hapoalim')
  flushSync()
  app.backToYours()
  flushSync()
  expect(app.answers.bank).toBe('Hapoalim')
})

test('each example sets only fields on screen, and answers no decision', () => {
  const app = start()
  for (const example of app.examples) {
    app.choose({ Flexibility: 'Fixed' })
    app.pickExample(example.id)
    flushSync()
    const { answers } = app
    expect(answers.flexibility, example.id).toBeUndefined()
    expect(answers.cushion, example.id).toBeUndefined()
    expect(app.choices, example.id).toEqual({})
    // "Money needed any day" is the "לשעת חירום" answer, and only that.
    expect(answers.needsAnyDay, example.id).toBe(app.when === 'anyDay')
    // The portfolio at a bank is the box ticked, and only that.
    expect(typeof answers.situation === 'object', example.id).toBe(app.invested)
    // Any day has no date: the field that would say one isn't on screen.
    if (app.when === 'anyDay' || app.when === 'unknown') expect(answers.horizon, example.id).toBe('Unknown')
    expect(app.exampleOnScreen?.id, example.id).toBe(example.id)
  }
})

test('only "לשעת חירום" sets money needed any day, and the date waits for its answer', () => {
  const app = start()
  const months = app.datedMonths
  app.setWhen('anyDay')
  flushSync()
  expect(app.answers.needsAnyDay).toBe(true)
  expect(app.answers.horizon).toBe('Unknown')
  for (const when of ['unknown', 'retirement', 'date'] as const) {
    app.setWhen(when)
    expect(app.answers.needsAnyDay).toBe(false)
  }
  expect(app.answers.horizon).toEqual({ Months: months })
})

test('an empty date or age is said, not read as a date not known', () => {
  const app = start()
  app.setHorizonMonths(null)
  flushSync()
  expect(app.answers.horizon).not.toBe('Unknown')
  expect(app.inputError).toBeDefined()
  app.setHorizonMonths(24)
  app.setWhen('retirement')
  flushSync()
  expect(app.inputError).toBeDefined()
  app.setHorizonRetirement(40, 65)
  flushSync()
  expect(app.inputError).toBeUndefined()
})

test('a link carries "מתי" and the portfolio, even where they differ from its example', () => {
  const app = start()
  app.pickExample('cushion')
  app.setWhen('date')
  app.setHorizonMonths(18)
  flushSync()
  const fromCushion = start(decode(app.address()))
  expect(fromCushion.when).toBe('date')
  expect(fromCushion.answers.needsAnyDay).toBe(false)
  app.pickExample('existing')
  app.setInvested(false)
  flushSync()
  expect(start(decode(app.address())).invested).toBe(false)
  app.setInvested(true)
  app.setWhen('anyDay')
  flushSync()
  const again = start(decode(app.address()))
  expect(again.invested).toBe(true)
  expect(again.when).toBe('anyDay')
})

test('an example’s numbers read in one line', () => {
  const app = start()
  const line = (id: string) => numbersLine(fieldsOf(app.examples.find((each) => each.id === id)!.answers))
  expect(line('goal')).toBe('₪100,000 ועוד ₪2,000 בחודש, 3 שנים')
  expect(line('salary')).toBe('₪500 בחודש, 30 שנה')
  expect(line('cushion')).toBe('₪30,000, לשעת חירום')
  expect(line('retirement')).toBe('₪3,000 בחודש, מגיל 45 עד 65')
  expect(line('existing')).toBe('₪200,000 מושקעים, 20 שנה')
  expect(line('unknown')).toBe('₪3,000 בחודש, בלי תאריך')
})
