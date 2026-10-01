import { expect, test } from 'vitest'
import { CHAINED, drawAt, glideFor, MAX_WAIT, settlesIn, TYPING_PAUSE } from './glide'
import { easeOut, SETTLE } from './motion'

const idle = { settled: -Infinity, lastTyped: -Infinity }

test('a change on its own is drawn at once, and glides the full time', () => {
  expect(drawAt({ ...idle, now: 1000, firstWaiting: 1000 })).toBe(1000)
  expect(glideFor(false)).toBe(SETTLE)
})

test('a change in the middle of a glide waits for it to settle, then glides less', () => {
  expect(drawAt({ ...idle, now: 1000, settled: 1250, firstWaiting: 1000 })).toBe(1250)
  expect(glideFor(true)).toBe(CHAINED)
  // Settled: all but the last hundredth of the way is behind it.
  expect(easeOut(settlesIn(SETTLE) / SETTLE)).toBeGreaterThan(0.99)
})

test('while a number is being typed, the chart waits until the typing pauses', () => {
  // The first key, then the next a moment later: each pushes the draw back.
  expect(drawAt({ ...idle, now: 1000, lastTyped: 1000, firstWaiting: 1000 })).toBe(1000 + TYPING_PAUSE)
  expect(drawAt({ ...idle, now: 1150, lastTyped: 1150, firstWaiting: 1000 })).toBe(1150 + TYPING_PAUSE)
  // A change after the typing has paused (the sweep's answer) isn't held up.
  expect(drawAt({ ...idle, now: 2000, lastTyped: 1000, firstWaiting: 2000 })).toBe(2000)
})

test('long typing is drawn now and then, however fast the keys come', () => {
  const firstWaiting = 1000
  expect(drawAt({ ...idle, now: 1850, lastTyped: 1850, firstWaiting })).toBe(firstWaiting + MAX_WAIT)
})
