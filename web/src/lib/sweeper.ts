// The heavy jobs run off the page's thread: the sweep (a comparison's work 16
// to 23 times over, behind the chart by deposit and the line about other
// deposits), and the guide's ladder, alternatives and, for long horizons,
// recommendation (see `recommendOnPage`). One worker does them one at a time.
// Each kind of job keeps only its latest request waiting, and only the answer
// to its latest request is passed on. Where there is no Worker (the unit
// tests), the same functions run on the page's thread a moment later.

import type { JobName, Jobs, Worked } from './work'
import { run } from './work'
import type { WorkerAnswer, WorkerMessage } from './sweep.worker'
import type { SweepData } from './core/core'

export type { GuideRequest, SweepRequest } from './work'
import type { SweepRequest } from './work'

type Answer<J extends JobName> = (worked: Worked<Jobs[J]['value']>) => void
type Waiting = { [J in JobName]?: { id: number; request: Jobs[J]['request']; answer: Answer<J> } }

// The order they're sent in when several wait: what the answer on screen
// needs first, the chart by deposit last.
const ORDER: JobName[] = ['answer', 'recommend', 'alternatives', 'ladder', 'sweep']

let worker: Worker | undefined
/** Made when first asked, so a page that never asks, and the tests, have none. */
function theWorker(): Worker | undefined {
  if (typeof Worker === 'undefined') return undefined
  worker ??= (() => {
    const made = new Worker(new URL('./sweep.worker.ts', import.meta.url), { type: 'module' })
    made.onmessage = ({ data }: MessageEvent<WorkerAnswer>) => {
      const done = working
      working = undefined
      // Passed on only if no newer request of its kind is waiting.
      if (done?.id === data.id && !waiting[data.job]) (done.answer as Answer<JobName>)(data.worked)
      send()
    }
    return made
  })()
  return worker
}

let asked = 0
const waiting: Waiting = {}
let working: { id: number; answer: Answer<JobName> } | undefined

/** Starts the worker (and its core) early, while the page loads. */
export function warmUp() {
  theWorker()
}

/** Asks for a job; `answer` gets it, unless a newer request of its kind
 * comes first. */
export function ask<J extends JobName>(job: J, request: Jobs[J]['request'], answer: Answer<J>) {
  const id = ++asked
  if (!theWorker()) {
    // No worker: on this thread, a moment later, still dropping what's superseded.
    waiting[job] = { id, request, answer } as Waiting[J]
    queueMicrotask(() => {
      const mine = waiting[job]
      if (mine?.id !== id) return
      delete waiting[job]
      ;(mine.answer as Answer<J>)(run(job, request))
    })
    return
  }
  waiting[job] = { id, request, answer } as Waiting[J]
  if (!working) send()
}

/** Asks for a sweep; `answer` gets it, or `undefined` for inputs it can't run. */
export function askSweep(request: SweepRequest, answer: (sweep: SweepData | undefined) => void) {
  ask('sweep', request, ({ value }) => answer(value))
}

function send() {
  const job = ORDER.find((name) => waiting[name])
  if (!job) return
  const next = waiting[job]!
  delete waiting[job]
  working = { id: next.id, answer: next.answer as Answer<JobName> }
  worker!.postMessage({ id: next.id, job, request: next.request } satisfies WorkerMessage)
}
