import { expect, test } from 'vitest'
import * as core from './core/core'
import { ask } from './sweeper'
import type { GuideRequest } from './work'

const settings = {
  indexReturnPercent: 10,
  inflationPercent: 2,
  boiRatePercent: 4,
  ilsPerUsd: 3.1,
  ilsPerEur: 3.5,
  plans: undefined,
}
const request = (id: string): GuideRequest => ({
  answers: core.situations().find((situation) => situation.id === id)!.answers,
  settings,
})

test('only the answer to the latest request of a kind is passed on, and it is the core’s', async () => {
  const answers: string[] = []
  // Without a Worker (here) the work runs a moment later, on this thread:
  // the first of two requests in a row is superseded, as in the browser.
  ask('recommend', request('salary'), () => answers.push('stale'))
  const latest = request('goal')
  let got: unknown
  ask('recommend', latest, (worked) => {
    answers.push('latest')
    got = worked.value
  })
  await new Promise((done) => setTimeout(done, 20))
  expect(answers).toEqual(['latest'])
  expect(got).toEqual(core.recommend(latest.answers, latest.settings))
})

test('kinds of job do not supersede each other', async () => {
  const answers: string[] = []
  ask('ladder', request('goal'), () => answers.push('ladder'))
  ask('alternatives', request('goal'), () => answers.push('alternatives'))
  await new Promise((done) => setTimeout(done, 20))
  expect(answers.sort()).toEqual(['alternatives', 'ladder'])
})

test('a request the core refuses is answered with its words', async () => {
  const bad = request('goal')
  bad.settings = { ...settings, ilsPerUsd: null }
  let error: string | undefined
  ask('recommend', bad, (worked) => (error = worked.error))
  await new Promise((done) => setTimeout(done, 20))
  expect(error).toBeTruthy()
})
