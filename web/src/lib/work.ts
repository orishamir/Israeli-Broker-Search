// What the worker computes, named once so the worker and the fallback (where
// there is no Worker: the unit tests) run the same functions.

import {
  alternatives,
  ladder,
  recommend,
  sweep,
  type AlternativesData,
  type AnswersInput,
  type GuideSettings,
  type Inputs,
  type RecommendationData,
  type RungData,
  type SweepData,
  type Swept,
} from './core/core'

/** The guide's questions with the settings they are asked under. */
export interface GuideRequest {
  answers: AnswersInput
  settings: GuideSettings
}

/** The sweep: the inputs without the deposit it varies, and which one that is. */
export interface SweepRequest {
  swept: Swept
  inputs: Inputs
}

/** Each job: what it's asked, and what it answers. */
export interface Jobs {
  sweep: { request: SweepRequest; value: SweepData }
  recommend: { request: GuideRequest; value: RecommendationData }
  ladder: { request: GuideRequest; value: RungData[] }
  alternatives: { request: GuideRequest; value: AlternativesData }
}
export type JobName = keyof Jobs

/** The core's answer, or the core's words for why it can't answer (the
 * page's comparison says why for the sweep). */
export type Worked<T> = { value: T; error?: undefined } | { value?: undefined; error: string }

export function run<J extends JobName>(job: J, request: Jobs[J]['request']): Worked<Jobs[J]['value']> {
  try {
    // One switch, so each job's request and answer types meet in one place.
    const value = (() => {
      switch (job) {
        case 'sweep': {
          const { inputs, swept } = request as SweepRequest
          return sweep(inputs, swept)
        }
        case 'recommend':
          return recommend((request as GuideRequest).answers, (request as GuideRequest).settings)
        case 'ladder':
          return ladder((request as GuideRequest).answers, (request as GuideRequest).settings)
        default:
          return alternatives((request as GuideRequest).answers, (request as GuideRequest).settings)
      }
    })()
    return { value: value as Jobs[J]['value'] }
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error) }
  }
}
