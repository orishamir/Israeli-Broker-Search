// The outcome chart's layout (REDESIGN.md §2.6), as pure functions so it's
// tested without a browser: where each mark sits on the shared ₪ axis, the
// axis's ticks, and where each label goes so none covers another. The
// component only draws what these return. The track runs left to right, as
// every chart on the page does; its labels are Hebrew.

import type { AnswerOutcomeData, Kind, OutcomeWordsData, RowData } from './core/core'
import { compactShekelsApart, shekels } from './format'
import { kindKey } from './kinds'

/** The axis, in today's ₪. */
export interface Span {
  min: number
  max: number
}

/** Where `value` sits: 0 at the axis's left end, 1 at its right; beyond
 * them outside the axis. */
export const at = ({ min, max }: Span, value: number): number =>
  max > min ? (value - min) / (max - min) : 0.5

const inside = (fraction: number) => fraction >= 0 && fraction <= 1

// ── The axis ──

export interface Tick {
  value: number
  at: number
  label: string
}

/** Round amounts inside the axis, 1, 2, 2.5 or 5 times a power of ten
 * apart: the finest step that leaves `gap` px between them on a track
 * `width` px wide, as long as two fit; else the widest that leaves two. */
export function ticks(span: Span, width: number, gap = 84): Tick[] {
  const range = span.max - span.min
  if (!(range > 0) || !(width > 0)) return []
  const among = (step: number) => {
    const values: number[] = []
    for (let value = Math.ceil(span.min / step) * step; value <= span.max + step * 1e-9; value += step)
      values.push(Math.round(value / step) * step)
    return values
  }
  const steps: number[] = []
  for (let power = 10 ** Math.floor(Math.log10(range / 50)); power <= range * 10; power *= 10)
    steps.push(...[1, 2, 2.5, 5].map((times) => power * times))
  const twoFit = steps.filter((step) => among(step).length >= 2)
  const step = twoFit.find((step) => (step / range) * width >= gap) ?? twoFit.at(-1) ?? steps[0]
  const values = among(step)
  const label = compactShekelsApart(values)
  return values.map((value) => ({ value, at: at(span, value), label: label(value) }))
}

/** Where a tick's label box starts, px: centred on the tick, but kept
 * inside the track, so the end ones don't stick out past the card. */
export const tickLabelX = (tick: Tick, width: number, labelWidth: number): number =>
  Math.min(Math.max(0, tick.at * width - labelWidth / 2), Math.max(0, width - labelWidth))

// ── Labels that don't collide ──

/** A label to place: its box's width, the point it belongs to (px from the
 * track's left), which side of the point the box goes (`start`: it begins
 * there and runs right; `end`: it ends there), and the lanes it may take,
 * best first. Lane 0 is above the mark; 1, 2… below it, downwards. */
export interface Wanted {
  key: string
  text: string
  width: number
  anchor: number
  align: 'start' | 'center' | 'end'
  lanes: number[]
}

export interface Placed extends Wanted {
  /** The box's left edge, px. */
  x: number
  lane: number
}

/** Each label, in the order given, takes the first of its lanes where it
 * doesn't come within `gap` px of a label already there, kept inside the
 * track; failing those, the first free lane below. */
export function place(wanted: Wanted[], width: number, gap = 8): Placed[] {
  const placed: Placed[] = []
  const clear = (x: number, w: number, lane: number) =>
    placed.every((other) => other.lane !== lane || x + w + gap <= other.x || other.x + other.width + gap <= x)
  for (const label of wanted) {
    const ideal =
      label.align === 'start'
        ? label.anchor
        : label.align === 'end'
          ? label.anchor - label.width
          : label.anchor - label.width / 2
    const x = Math.min(Math.max(0, ideal), Math.max(0, width - label.width))
    // Below its own lanes, as many as could ever be needed.
    const lanes = [...label.lanes]
    for (let lane = 1; lane <= placed.length + 1; lane++) if (!lanes.includes(lane)) lanes.push(lane)
    const lane = lanes.find((each) => clear(x, label.width, each)) ?? lanes.at(-1)!
    placed.push({ ...label, x, lane })
  }
  return placed
}

// ── A row ──

/** Heights, px: a lane of labels, the lane that holds the return's field,
 * and the band the marks are drawn in. */
export const LANE = 16
export const FIELD_LANE = 32
export const MARKS = 26
/** Room for the arrow at a bar's cut end, px. */
export const ARROW = 10

export interface RowLayout {
  /** The dot, as a share of the track's width. */
  dot?: number
  /** The bar, as shares of the width; a cut end has an arrow (the tail goes
   * past the axis, and its label says how far). */
  bar?: { kind: 'rate' | 'market'; from: number; to: number; cutLow: boolean; cutHigh: boolean }
  ticks: { role: 'median' | 'assumed'; at: number }[]
  labels: (Placed & { top: number })[]
  /** The marks' middle, px from the track's top, and the track's height. */
  middle: number
  height: number
}

export interface Measures {
  /** The width of a label's text, px. */
  text: (text: string) => number
  /** The width of the return's label: the field and "בשנה" after it, or
   * the plain "10% בשנה" where the row has no field. */
  assumed: number
  /** The plain return label's text, "10% בשנה". */
  assumedText: string
}

/** Where a row's marks and labels go on a track `width` px wide. */
export function layoutRow(
  row: RowData,
  span: Span,
  width: number,
  words: OutcomeWordsData,
  measures: Measures,
  withField: boolean,
): RowLayout {
  const mark = row.mark
  const value = at(span, row.value.today)
  const wanted: Wanted[] = []
  const layout: Omit<RowLayout, 'labels' | 'middle' | 'height'> = { ticks: [] }
  const assumedLabel = (fraction: number) => {
    if (!inside(fraction)) return
    wanted.push({
      key: 'assumed',
      text: measures.assumedText,
      width: measures.assumed,
      anchor: fraction * width,
      align: 'center',
      lanes: [1],
    })
  }
  switch (mark.type) {
    case 'known':
      layout.dot = clamp(value)
      break
    case 'expected':
      layout.dot = clamp(value)
      assumedLabel(value)
      break
    case 'rateRange': {
      layout.dot = clamp(value)
      layout.bar = bar('rate', at(span, mark.low.today), at(span, mark.high.today))
      break
    }
    case 'market': {
      const ends = bar('market', at(span, mark.worst.today), at(span, mark.best.today))
      layout.bar = ends
      const assumed = at(span, mark.assumed.today)
      const median = at(span, mark.median.today)
      if (inside(assumed)) layout.ticks.push({ role: 'assumed', at: assumed })
      if (inside(median)) layout.ticks.push({ role: 'median', at: median })
      const amount = compactShekelsApart([mark.worst.today, mark.median.today, mark.best.today])
      const label = (key: string, word: string, figure: number) => {
        const text = `${word}: ${amount(figure)}`
        return { key, text, width: measures.text(text) }
      }
      assumedLabel(assumed)
      wanted.push({
        ...label('worst', words.worst, mark.worst.today),
        anchor: ends.from * width + (ends.cutLow ? ARROW : 0),
        align: 'start',
        lanes: [0, 1],
      })
      wanted.push({
        ...label('best', words.best, mark.best.today),
        anchor: ends.to * width - (ends.cutHigh ? ARROW : 0),
        align: 'end',
        lanes: [0, 1],
      })
      if (inside(median))
        wanted.push({
          ...label('median', words.median, mark.median.today),
          anchor: median * width,
          align: 'center',
          lanes: [1, 0],
        })
      break
    }
  }
  const placed = place(wanted, width)
  const above = placed.some(({ lane }) => lane === 0) ? LANE : 0
  const below = Math.max(0, ...placed.map(({ lane }) => lane))
  // Lane 1 is taller where the field sits in it.
  const laneHeight = (lane: number) =>
    lane === 1 && withField && wanted.some(({ key }) => key === 'assumed') ? FIELD_LANE : LANE
  const topOf = (lane: number) => {
    if (lane === 0) return 0
    let top = above + MARKS
    for (let each = 1; each < lane; each++) top += laneHeight(each)
    return top
  }
  let height = above + MARKS
  for (let lane = 1; lane <= below; lane++) height += laneHeight(lane)
  return {
    ...layout,
    labels: placed.map((label) => ({ ...label, top: topOf(label.lane) })),
    middle: above + MARKS / 2,
    height,
  }
}

const clamp = (fraction: number) => Math.min(1, Math.max(0, fraction))

function bar(kind: 'rate' | 'market', low: number, high: number) {
  return { kind, from: clamp(low), to: clamp(high), cutLow: low < 0, cutHigh: high > 1 }
}

// ── What the rows say ──

/** The rows' notes, each said once: the reason the rules leave a row out,
 * its own note, and under the index's bar the past's losses, which every
 * holding of the index shares and so is said under the first of them only. */
export function notes(rows: RowData[]): string[][] {
  const said = new Set<string>()
  return rows.map((row) => {
    const lines: string[] = []
    if (row.excluded) lines.push(row.excluded)
    if (row.note) lines.push(row.note)
    if (row.mark.type === 'market' && !said.has(row.mark.losses)) {
      said.add(row.mark.losses)
      lines.push(row.mark.losses)
    }
    return lines
  })
}

/** The one row whose return label is a field: the first that has the label
 * and that the rules don't leave out (the same number is set in one place). */
export function fieldRow(rows: RowData[]): number {
  return rows.findIndex(
    (row) => !row.excluded && (row.mark.type === 'market' || row.mark.type === 'expected'),
  )
}

/** The return label's plain text: "10% בשנה". */
export const assumedText = (percent: number, words: OutcomeWordsData): string =>
  `${formatPercent(percent)}%\u00a0${words.perYear}`

const formatPercent = (percent: number) =>
  new Intl.NumberFormat('en-US', { maximumFractionDigits: 2 }).format(percent)

/** What a screen reader says for a row's button: the row's name, its tags,
 * its figure, and for the index the return it assumes and the past's range. */
export function rowLabel(row: RowData, words: OutcomeWordsData, percent: number): string {
  const parts = [row.name, ...row.tags, shekels(row.value.today)]
  const mark = row.mark
  if (mark.type === 'market' || mark.type === 'expected')
    parts[parts.length - 1] += ` ${words.ifItRises}${formatPercent(percent)}%\u00a0${words.perYear}`
  if (mark.type === 'market') {
    const amount = compactShekelsApart([mark.worst.today, mark.median.today, mark.best.today])
    parts.push(
      `${words.worst}: ${amount(mark.worst.today)}`,
      `${words.median}: ${amount(mark.median.today)}`,
      `${words.best}: ${amount(mark.best.today)}`,
    )
  }
  if (row.excluded) parts.push(row.excluded)
  return parts.join('. ')
}

/** Which keys the legend shows: a mark's only when a row draws it. */
export function legendKeys(outcome: AnswerOutcomeData): { known: boolean; market: boolean } {
  const types = new Set(outcome.rows.map(({ mark }) => mark.type))
  return { known: types.has('known') || types.has('rateRange'), market: types.has('market') }
}

/** Whether `row` is the one `focused` names. */
export const isFocused = (row: RowData, focused: Kind | undefined): boolean =>
  focused !== undefined && kindKey(row.id) === kindKey(focused)
