// The sweep behind the chart by deposit and the best plan's line about
// other deposits runs every ticked plan at a range of deposits: a
// comparison's work 10 to 15 times over, a tenth to half a second on a
// phone, too long for the page's own thread at every change. A worker does
// it, one request at a time: only the latest request waiting is sent, and
// only the answer to the latest request is passed on.

import type { Inputs, SweepData, Swept } from './core/core'
import type { SweepAnswer, SweepMessage } from './sweep.worker'

/** The inputs without the deposit the sweep varies, and which one that is. */
export interface SweepRequest {
  swept: Swept
  inputs: Inputs
}

type Answer = (sweep: SweepData | undefined) => void

// Made as soon as the page loads, so it loads its core while the page does.
const worker = new Worker(new URL('./sweep.worker.ts', import.meta.url), { type: 'module' })
let asked = 0
let waiting: { id: number; request: SweepRequest; answer: Answer } | undefined
let working: { id: number; answer: Answer } | undefined

worker.onmessage = ({ data }: MessageEvent<SweepAnswer>) => {
  const done = working
  working = undefined
  if (done?.id === data.id && !waiting) done.answer(data.sweep)
  send()
}

/** Asks for a sweep; `answer` gets it, unless another request comes first. */
export function askSweep(request: SweepRequest, answer: Answer) {
  waiting = { id: ++asked, request, answer }
  if (!working) send()
}

function send() {
  if (!waiting) return
  const { id, request, answer } = waiting
  waiting = undefined
  working = { id, answer }
  worker.postMessage({ id, ...request } satisfies SweepMessage)
}
