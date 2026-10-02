import type { LineSeriesOption } from 'echarts/charts'
import { expect, test } from 'vitest'
import { crossoverOption, FLOOR, YOURS, type CostLine, type CrossoverView } from './crossover-chart'

const line = (id: string, costs: (number | null)[], dotted = false): CostLine => ({
  id,
  label: `Plan ${id}`,
  color: '#56b4e9',
  rank: 1,
  dotted,
  costs,
})

const view = (changes: Partial<CrossoverView> = {}): CrossoverView => ({
  amounts: [100, 1000, 10000],
  lines: [line('a', [5, 0.5, 0.2]), line('b', [2, 0.8, 0], true)],
  swept: 'Monthly',
  yours: 2000,
  pinned: new Set(),
  ...changes,
})

const series = (option: ReturnType<typeof crossoverOption>) => option.series as LineSeriesOption[]

test('every plan is a series by its id, at each amount, with costs below the floor raised to it', () => {
  const [a, b, yours] = series(crossoverOption(view()))
  expect([a.id, b.id, yours.id]).toEqual(['a', 'b', YOURS])
  // The first point sits on the axis: its label goes to the right.
  expect(a.data).toEqual([{ value: [100, 5], label: { position: 'right' } }, [1000, 0.5], [10000, 0.2]])
  // A logarithmic axis has no zero.
  expect((b.data as number[][])[2]).toEqual([10000, FLOOR])
  expect(a.lineStyle?.type).toBe('solid')
  expect(b.lineStyle?.type).toBe('dotted')
})

test('both axes are logarithmic, and the deposit axis is named for what varies', () => {
  const monthly = crossoverOption(view())
  expect(monthly.xAxis).toMatchObject({ type: 'log', min: 100, max: 10000, name: 'הפקדה בחודש' })
  expect(monthly.yAxis).toMatchObject({ type: 'log', name: 'עמלות בשנה' })
  const once = crossoverOption(view({ swept: 'OneTime', yours: 200_000 }))
  expect(once.xAxis).toMatchObject({ name: 'הפקדה חד\u2011פעמית', max: 200_000 })
})

test("a fund's line ends where the deposits pass its ceiling", () => {
  const [fund, broker] = series(
    crossoverOption(view({ lines: [line('fund', [0.6, 0.6, null]), line('broker', [5, 0.5, 0.2])] })),
  )
  expect(fund.data).toEqual([{ value: [100, 0.6], label: { position: 'right' } }, [1000, 0.6]])
  expect(broker.data).toHaveLength(3)
})

test("the user's deposit is a dashed marker, and stretches the axis to reach it", () => {
  const marker = series(crossoverOption(view({ yours: 50_000 }))).at(-1)!
  expect(marker.id).toBe(YOURS)
  expect(marker.silent).toBe(true)
  expect(marker.markLine).toMatchObject({
    data: [{ xAxis: 50_000 }],
    lineStyle: { type: 'dashed' },
    label: { formatter: 'אתם: ₪50,000 בחודש' },
  })
  expect(crossoverOption(view({ yours: 50_000 })).xAxis).toMatchObject({ max: 50_000 })
  expect(crossoverOption(view({ yours: 50 })).xAxis).toMatchObject({ min: 50 })
  // No deposit yet: no marker.
  for (const yours of [null, 0]) {
    expect(series(crossoverOption(view({ yours }))).map(({ id }) => id)).toEqual(['a', 'b'])
  }
})

test('pinned lines are wider and dotted at each amount, and the rest fade', () => {
  const [a, b] = series(crossoverOption(view({ pinned: new Set(['a']) })))
  expect(a.lineStyle).toMatchObject({ width: 3.5, opacity: 1 })
  expect(a.showSymbol).toBe(true)
  expect(a.label).toMatchObject({ show: true })
  expect(b.lineStyle).toMatchObject({ width: 2, opacity: 0.4 })
  expect(b.showSymbol).toBe(false)
  expect(b.endLabel).toMatchObject({ opacity: 0.6 })
})

test('the tooltip lists the cheapest first, in percent', () => {
  const option = crossoverOption(view())
  expect(option.tooltip).toMatchObject({ order: 'valueAsc' })
  const { valueFormatter } = option.tooltip as { valueFormatter: (value: number) => string }
  expect(valueFormatter(0.4213)).toBe('0.42%')
})
