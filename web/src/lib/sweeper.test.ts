import { expect, test } from 'vitest'
import * as core from './core/core'
import { ask } from './sweeper'
import type { AnswerRequest } from './core/core'
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

/** The answer's request for a situation, without its horizons. */
const answerRequest = (id: string): AnswerRequest => ({ ...request(id), choices: {}, horizons: false })

test('kinds of job do not supersede each other', async () => {
  const answers: string[] = []
  ask('answer', answerRequest('goal'), () => answers.push('answer'))
  ask('horizons', answerRequest('goal'), () => answers.push('horizons'))
  await new Promise((done) => setTimeout(done, 50))
  expect(answers.sort()).toEqual(['answer', 'horizons'])
})

test('the answer comes without its horizons, and the horizons are a job of their own', async () => {
  const asked = answerRequest('bonus')
  let answer: core.AnswerData | undefined
  let horizons: core.HorizonRowData[] | undefined
  ask('answer', asked, (worked) => (answer = worked.value))
  // A horizons request superseded by a newer one is never answered.
  ask('horizons', answerRequest('salary'), () => {
    throw new Error('superseded')
  })
  ask('horizons', asked, (worked) => (horizons = worked.value))
  await new Promise((done) => setTimeout(done, 100))
  expect(answer?.horizons).toEqual([])
  expect(horizons).toEqual(core.answer({ ...asked, horizons: true }).horizons)
  expect(horizons?.length).toBeGreaterThan(0)
})

test('a request the core refuses is answered with its words', async () => {
  const bad = request('goal')
  bad.settings = { ...settings, ilsPerUsd: null }
  let error: string | undefined
  ask('recommend', bad, (worked) => (error = worked.error))
  await new Promise((done) => setTimeout(done, 20))
  expect(error).toBeTruthy()
})
