import { flushSync } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState } from './app.svelte'
import { BRANCH_PLAN, branchPlans, yoursBranch } from './bank-plan'
import * as core from './core/core'
import { t } from './text'

beforeEach(() => {
  localStorage.clear()
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})

function existingPortfolio(): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState({})
  })
  flushSync()
  app.setSituation(app.situations.find((each) => each.opensDialog)!.id)
  return app
}

/** The recommendation as the worker would say it for a long horizon. */
const recommended = (app: AppState) => core.recommend(app.guideRequest.answers, app.settings)

test('the chosen bank’s own branch plan is the one the core marks, and it is found in the list', () => {
  const app = existingPortfolio()
  expect(app.banks.map(({ bank }) => bank)).toEqual([
    'Hapoalim',
    'Leumi',
    'Discount',
    'MizrahiTefahot',
    'Beinleumi',
    'OtsarHahayal',
  ])
  const names = {
    Hapoalim: 'Hapoalim',
    Leumi: 'Leumi',
    Discount: 'Discount',
    MizrahiTefahot: 'Mizrahi',
    Beinleumi: 'Beinleumi',
    OtsarHahayal: 'Otsar Hahayal',
  } as const
  for (const bank of app.banks.map((each) => each.bank)) {
    app.setBank(bank)
    flushSync()
    const found = yoursBranch(app.plans, recommended(app))
    expect(found?.englishLabel).toBe(`${names[bank]} · ${BRANCH_PLAN}`)
  }
})

test('for another bank, the plan the core stands in with is found too', () => {
  const app = existingPortfolio()
  app.setBank(undefined)
  flushSync()
  const rec = recommended(app)
  const found = yoursBranch(app.plans, rec)
  expect(found).toBeDefined()
  expect(branchPlans(app.plans).map((plan) => plan.id)).toContain(found!.id)
  // The guide doesn't show the bank whose price list stands in.
  const own = rec.plans.find((plan) => plan.yours)!
  const bank = found!.label.split('\u00a0· ')[0]
  expect(own.name).toContain(t.otherBank)
  expect(own.name).not.toContain(bank)
  expect(own.ownPlan).toBe(found!.label)
})

test('ticking leaves exactly the saver’s branch plan ticked among the branch plans', () => {
  const app = existingPortfolio()
  // The guide's own answer is in a worker in the browser, and not here.
  for (const plan of branchPlans(app.plans)) app.selected.add(plan.id)
  vi.spyOn(app, 'recommendation', 'get').mockReturnValue(recommended(app))
  app.tickYourBranch()
  expect(branchPlans(app.plans).filter((plan) => app.selected.has(plan.id))).toHaveLength(1)
})
