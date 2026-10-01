import { readFileSync } from 'node:fs'
import { expect, test } from 'vitest'
import { EASE_IN, EASE_OUT, easeIn, easeOut, SETTLE } from './motion'

test("what script moves keeps to the page's own curves and time", () => {
  // Relative to web/, where vitest runs.
  const css = readFileSync('src/app.css', 'utf8')
  expect(css).toContain(`--ease-out: ${EASE_OUT};`)
  expect(css).toContain(`--ease-in: ${EASE_IN};`)
  expect(css).toContain(`--settle: ${SETTLE}ms var(--ease-out);`)
})

test('the curves start at rest and end in place: arriving settles, leaving speeds up', () => {
  for (const curve of [easeOut, easeIn]) {
    expect(curve(0)).toBe(0)
    expect(curve(1)).toBe(1)
  }
  expect(easeOut(0.5)).toBeCloseTo(0.88, 2)
  expect(easeIn(0.5)).toBeLessThan(0.5)
})
