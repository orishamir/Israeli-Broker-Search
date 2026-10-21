import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState } from './app.svelte'
import * as core from './core/core'
import type { AnswerOutcomeData } from './core/core'
import { kindColor } from './colors'
import { shekels } from './format'
import {
  assumedText,
  at,
  fieldRow,
  LANE,
  layoutRow,
  LINE_LANES,
  lineLabels,
  MARKS,
  markText,
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
    app.pickExample(situation.id)
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
    for (const width of [260, 288, 320, 343, 375, 480, 760]) {
      outcome.rows.forEach((row) => {
        const layout = layoutRow(row, span, width, { text: measure, assumedText: assumedText(10, words) })
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
          // Every mark on the band is labelled with its amount (§4.6).
          expect(row.marks.map(({ which }) => which).sort()).toEqual(['Assumed', 'Best', 'Median', 'Worst'])
          const texts = layout.labels.map(({ text }) => text)
          for (const each of row.marks) expect(texts, where).toContain(markText(each))
          for (const each of row.marks) expect(markText(each)).toContain(each.amount)
          // Every tick drawn is labelled.
          const keys = layout.labels.map(({ key }) => key)
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

test('the two lines are labelled with their amounts, apart, on the track, in two lanes kept for them', () => {
  for (const [id, outcome] of charts()) {
    expect(outcome.lines.map(({ which }) => which)).toEqual(['Deposited', 'CurrentAccount'])
    const span = { min: outcome.axis[0], max: outcome.axis[1] }
    for (const width of [260, 320, 375, 760]) {
      const labels = lineLabels(outcome.lines, span, width, measure)
      const where = `${id} at ${width}px`
      expect(labels.map(({ text }) => text)).toEqual(
        outcome.lines.map((line) => `${line.label}\u00a0${line.amount}`),
      )
      for (const label of labels) {
        expect(label.x, where).toBeGreaterThanOrEqual(0)
        expect(label.x + label.width, where).toBeLessThanOrEqual(width + 1e-9)
        expect(label.lane).toBeLessThan(LINE_LANES)
        // Beside its own line: it starts or ends a few px from it.
        const line = label.at * width
        expect(
          Math.min(Math.abs(label.x - line), Math.abs(label.x + label.width - line)),
          where,
        ).toBeLessThan(12)
      }
      expect(overlap(labels[0], labels[1], 0), where).toBe(false)
    }
  }
})

test('money needed any day draws only its own options, with no stock row', () => {
  const [, outcome] = charts().find(([id]) => id === 'cushion')!
  expect(outcome.rows.length).toBeGreaterThan(1)
  expect(outcome.rows.every((row) => row.mark.type !== 'market' && row.marks.length === 0)).toBe(true)
  expect(fieldRow(outcome.rows)).toBe(-1)
  expect(outcome.lines).toHaveLength(2)
})

test('every row has a colour of its own kind', () => {
  for (const [, outcome] of charts()) for (const row of outcome.rows) expect(kindColor(row.id)).toMatch(/^#/)
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
      expect(label).toContain(row.amount)
      if (row.mark.type === 'market') {
        for (const each of row.marks.filter(({ which }) => which !== 'Assumed'))
          expect(label).toContain(each.label)
        expect(label).toContain('10%')
      }
    }
})

// The core draws no row for an option the rules leave out (REDESIGN.md §4.6);
// should one come, it is read out with its reason, and no figure.
test('an excluded row is read out with its reason, and no figure', () => {
  const excluded = [...charts()].flatMap(([, outcome]) => outcome.rows.filter(({ excluded }) => excluded))
  expect(excluded).toEqual([])
  for (const row of excluded) {
    const label = rowLabel(row, words, 10)
    expect(label).toContain(row.excluded)
    expect(label).not.toContain(shekels(row.value.today))
  }
})
