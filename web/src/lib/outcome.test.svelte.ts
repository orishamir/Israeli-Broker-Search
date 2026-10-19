import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState } from './app.svelte'
import * as core from './core/core'
import type { AnswerOutcomeData } from './core/core'
import { shekels } from './format'
import {
  assumedText,
  at,
  fieldRow,
  LANE,
  layoutRow,
  legendKeys,
  MARKS,
  notes,
  place,
  rowLabel,
  ticks,
  type Placed,
} from './outcome'

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})

const words = core.outcomeWords()
/** About as wide as the page's 12 px labels. */
const measure = (text: string) => text.length * 6.5 + 6

/** Every situation's chart, as the core draws it. */
function charts(): [string, AnswerOutcomeData][] {
  return core.situations().map((situation) => {
    let app!: AppState
    $effect.root(() => {
      app = new AppState({})
    })
    flushSync()
    app.setSituation(situation.id)
    flushSync()
    const answer = app.answer ?? core.answer({ ...app.answerRequest, horizons: false })
    return [situation.id, answer.outcome]
  })
}

const overlap = (a: Placed, b: Placed, gap: number) =>
  a.lane === b.lane && a.x < b.x + b.width + gap && b.x < a.x + a.width + gap

test('a value sits on the axis by its share of it', () => {
  const span = { min: 100, max: 300 }
  expect(at(span, 100)).toBe(0)
  expect(at(span, 300)).toBe(1)
  expect(at(span, 200)).toBe(0.5)
  expect(at(span, 50)).toBeLessThan(0)
})

test('the axis has round ticks inside it, spaced apart, each labelled differently', () => {
  for (const [, outcome] of charts())
    for (const width of [280, 500, 900]) {
      const span = { min: outcome.axis[0], max: outcome.axis[1] }
      const marks = ticks(span, width)
      expect(marks.length).toBeGreaterThanOrEqual(2)
      for (const tick of marks) {
        expect(tick.at).toBeGreaterThanOrEqual(0)
        expect(tick.at).toBeLessThanOrEqual(1)
      }
      // Neighbouring labels, centred on their ticks, don't touch.
      const apart = (marks[1].at - marks[0].at) * width
      for (const [i, tick] of marks.slice(1).entries())
        expect(apart).toBeGreaterThan((measure(tick.label) + measure(marks[i].label)) / 2)
      expect(new Set(marks.map(({ label }) => label)).size).toBe(marks.length)
      const step = marks[1].value - marks[0].value
      const power = 10 ** Math.floor(Math.log10(step))
      expect([1, 2, 2.5, 5, 10]).toContain(Math.round((step / power) * 10) / 10)
    }
})

test('labels never cover each other, and stay inside the track', () => {
  const width = 300
  const wanted = [
    { key: 'a', text: 'a', width: 120, anchor: 150, align: 'center' as const, lanes: [1] },
    { key: 'b', text: 'b', width: 120, anchor: 140, align: 'start' as const, lanes: [0, 1] },
    { key: 'c', text: 'c', width: 120, anchor: 300, align: 'end' as const, lanes: [0, 1] },
    { key: 'd', text: 'd', width: 90, anchor: 290, align: 'center' as const, lanes: [1, 0] },
  ]
  const placed = place(wanted, width, 8)
  for (const [i, a] of placed.entries()) {
    expect(a.x).toBeGreaterThanOrEqual(0)
    expect(a.x + a.width).toBeLessThanOrEqual(width)
    for (const b of placed.slice(i + 1)) expect(overlap(a, b, 8)).toBe(false)
  }
  // The first takes its own lane; a crowded one goes further down.
  expect(placed[0].lane).toBe(1)
  expect(Math.max(...placed.map(({ lane }) => lane))).toBeGreaterThan(1)
})

test('every row of every situation is laid out without collisions, at phone and desktop widths', () => {
  for (const [id, outcome] of charts()) {
    const span = { min: outcome.axis[0], max: outcome.axis[1] }
    const field = fieldRow(outcome.rows)
    for (const width of [260, 320, 480, 760]) {
      outcome.rows.forEach((row, index) => {
        const plain = assumedText(10, words)
        const layout = layoutRow(
          row,
          span,
          width,
          words,
          { text: measure, assumed: index === field ? 110 : measure(plain), assumedText: plain },
          index === field,
        )
        const where = `${id} ${row.name} at ${width}px`
        for (const [i, a] of layout.labels.entries()) {
          expect(a.x, where).toBeGreaterThanOrEqual(0)
          expect(a.x + a.width, where).toBeLessThanOrEqual(width + 1e-9)
          for (const b of layout.labels.slice(i + 1))
            expect(overlap(a, b, 8), `${where}: ${a.key}/${b.key}`).toBe(false)
          expect(a.top + LANE, where).toBeLessThanOrEqual(layout.height + 1e-9)
        }
        expect(layout.height).toBeGreaterThanOrEqual(MARKS)
        if (layout.dot !== undefined) expect(layout.dot).toBeGreaterThanOrEqual(0)
        if (layout.dot !== undefined) expect(layout.dot).toBeLessThanOrEqual(1)
        const mark = row.mark
        if (mark.type === 'market') {
          const bar = layout.bar!
          expect(bar.from).toBeLessThanOrEqual(bar.to)
          // A tail past the axis is cut, with its label saying how far.
          expect(bar.cutLow).toBe(mark.worst.today < span.min)
          expect(bar.cutHigh).toBe(mark.best.today > span.max)
          const keys = layout.labels.map(({ key }) => key)
          expect(keys).toContain('worst')
          expect(keys).toContain('best')
          // Every tick drawn is labelled.
          for (const tick of layout.ticks) expect(keys, where).toContain(tick.role)
        }
        if (mark.type === 'rateRange') {
          const bar = layout.bar!
          expect(bar.from).toBeLessThanOrEqual(layout.dot!)
          expect(layout.dot!).toBeLessThanOrEqual(bar.to)
        }
      })
    }
  }
})

test('the past’s losses are said once, and the return is set in one place only', () => {
  for (const [, outcome] of charts()) {
    const said = notes(outcome.rows).flat()
    expect(new Set(said).size).toBe(said.length)
    for (const row of outcome.rows) if (row.mark.type === 'market') expect(said).toContain(row.mark.losses)
    const field = fieldRow(outcome.rows)
    if (field >= 0) {
      expect(outcome.rows[field].excluded).toBeUndefined()
      expect(['market', 'expected']).toContain(outcome.rows[field].mark.type)
    }
  }
})

test('a row is read out with its name, figure and, for the index, the return and the past’s range', () => {
  for (const [, outcome] of charts())
    for (const row of outcome.rows.filter(({ excluded }) => !excluded)) {
      const label = rowLabel(row, words, 10)
      expect(label).toContain(row.name)
      expect(label).toContain(shekels(row.value.today))
      if (row.mark.type === 'market') {
        expect(label).toContain(words.worst)
        expect(label).toContain('10%')
      }
    }
})

test('an excluded row is read out with its reason, and no figure', () => {
  const excluded = [...charts()].flatMap(([, outcome]) => outcome.rows.filter(({ excluded }) => excluded))
  expect(excluded.length).toBeGreaterThan(0)
  for (const row of excluded) {
    const label = rowLabel(row, words, 10)
    expect(label).toContain(row.excluded)
    expect(label).not.toContain(shekels(row.value.today))
  }
})

test('the legend keys only the marks drawn', () => {
  for (const [, outcome] of charts()) {
    const keys = legendKeys(outcome)
    expect(keys.market).toBe(outcome.rows.some(({ mark }) => mark.type === 'market'))
  }
})
