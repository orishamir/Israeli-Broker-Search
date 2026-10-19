// Works out the page's heavy jobs (see sweeper.ts): the sweep, and the
// guide's recommendation for long horizons, its ladder and its alternatives'
// lines. It has its own copy of the core, so the page's own thread stays
// free while it does.

import init, { setLang } from './core/core'
import { run, type JobName, type Jobs, type Worked } from './work'

export interface WorkerMessage<J extends JobName = JobName> {
  id: number
  job: J
  request: Jobs[J]['request']
}

export interface WorkerAnswer<J extends JobName = JobName> {
  id: number
  job: J
  worked: Worked<Jobs[J]['value']>
}

// The worker's core is its own copy, which must be told the page's language
// as the page's was (main.ts): the guide's sentences are made here too.
const ready = init().then(() => setLang('He'))

self.onmessage = async ({ data }: MessageEvent<WorkerMessage>) => {
  await ready
  self.postMessage({ id: data.id, job: data.job, worked: run(data.job, data.request) } satisfies WorkerAnswer)
}
