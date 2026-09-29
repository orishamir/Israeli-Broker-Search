// Works out sweeps for the page (see sweeper.ts), with its own copy of the
// core, so the page's own thread stays free while it does.

import init, { sweep } from './core/core'
import type { Inputs, SweepData, Swept } from './core/core'

export interface SweepMessage {
  id: number
  inputs: Inputs
  swept: Swept
}

export interface SweepAnswer {
  id: number
  /** Missing for inputs the sweep can't run: the page's comparison says why. */
  sweep: SweepData | undefined
}

const ready = init()

self.onmessage = async ({ data }: MessageEvent<SweepMessage>) => {
  await ready
  let answer: SweepData | undefined
  try {
    answer = sweep(data.inputs, data.swept)
  } catch {
    answer = undefined
  }
  self.postMessage({ id: data.id, sweep: answer } satisfies SweepAnswer)
}
