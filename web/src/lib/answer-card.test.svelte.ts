// The answer card on the core's real answers (REDESIGN.md §2.2–§2.4): the
// chip, the decision component's accessibility and behaviour, and the
// cushion's expenses field. The worked examples' structure is the core's
// (crates/core/tests/answer.rs); here, that the page shows it as the rules say.
import { flushSync, mount, unmount } from 'svelte'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { AppState } from './app.svelte'
import AnswerCard from './AnswerCard.svelte'
import { t } from './text'

let mounted: ReturnType<typeof mount> | undefined

beforeEach(() => {
  localStorage.clear()
  // jsdom has none; the card's height glide only keeps track of sizes with it.
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      disconnect() {}
    },
  )
  // jsdom has no Web Animations either: a field that comes and goes
  // (`transition:reveal`) is done at once.
  Element.prototype.animate ??= function () {
    const animation = {
      cancel() {},
      currentTime: 0,
      set onfinish(done: (() => void) | null) {
        if (done) queueMicrotask(done)
      },
    }
    return animation as unknown as Animation
  }
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})
afterEach(() => {
  if (mounted) unmount(mounted)
  mounted = undefined
  document.body.innerHTML = ''
})

/** The answers worked out off the page's thread arrive a microtask later. */
async function settle() {
  for (let round = 0; round < 5; round++) {
    await Promise.resolve()
    flushSync()
  }
}

async function start(situation: string): Promise<{ app: AppState; card: HTMLElement }> {
  if (mounted) unmount(mounted)
  document.body.innerHTML = ''
  let app!: AppState
  $effect.root(() => {
    app = new AppState({})
  })
  flushSync()
  app.setSituation(situation)
  await settle()
  mounted = mount(AnswerCard, { target: document.body, props: { app } })
  flushSync()
  const card = document.querySelector<HTMLElement>('section.answer')
  if (!card) throw new Error('no answer card')
  return { app, card }
}

const groups = (card: HTMLElement) => [...card.querySelectorAll<HTMLElement>('[role=group]')]
const buttonsOf = (group: HTMLElement) => [...group.querySelectorAll<HTMLButtonElement>('button')]

test('every decision is a group named by its question, of real buttons, none pressed at first', async () => {
  for (const situation of ['goal', 'bonus', 'salary']) {
    const { app, card } = await start(situation)
    const decisions = app.answer!.lines.flatMap((line) =>
      line.state.type === 'decision' ? [line.state.decision] : [],
    )
    expect(decisions.length, situation).toBeGreaterThan(0)
    expect(groups(card)).toHaveLength(decisions.length)
    for (const [at, group] of groups(card).entries()) {
      const question = document.getElementById(group.getAttribute('aria-labelledby')!)
      expect(question?.textContent).toBe(decisions[at].question)
      const buttons = buttonsOf(group)
      expect(buttons.map((button) => button.textContent?.trim())).toEqual(
        decisions[at].options.map((option) => option.button),
      )
      for (const button of buttons) expect(button.getAttribute('aria-pressed')).toBe('false')
    }
  }
})

test('only the sure answer wears the accent', async () => {
  for (const situation of ['goal', 'bonus', 'salary']) {
    const { app, card } = await start(situation)
    const state = app.answer!.confidence.state
    expect(card.classList.contains('state-A')).toBe(state === 'A')
    expect(card.querySelector('.chip')?.textContent).toBe(t.confidenceChip[state])
  }
})

test('choosing marks the option, keeps the others, opens what depended on it, and can change', async () => {
  const { app, card } = await start('bonus')
  expect(card.querySelectorAll('.waiting').length).toBeGreaterThan(0)
  const [first] = groups(card)
  const [safe, growth] = buttonsOf(first)
  safe.click()
  await settle()
  expect(safe.getAttribute('aria-pressed')).toBe('true')
  expect(growth.getAttribute('aria-pressed')).toBe('false')
  expect(growth.closest('.option')?.classList.contains('other')).toBe(true)
  // The way's decision appears only after this one (principle 8).
  expect(groups(card).length).toBeGreaterThan(1)
  expect(card.textContent).toContain(t.pillChosen)
  // And the choice can be changed, while a decision is still open.
  growth.click()
  await settle()
  expect(app.answers.flexibility).toBe('CanWait')
})

test('the plan shows once every decision is made, and the count until then', async () => {
  const { app, card } = await start('goal')
  expect(card.querySelector('.left')?.textContent).toBe(t.decisionsLeft(app.answer!.openDecisions))
  expect(card.querySelector('.plan')).toBeNull()
  buttonsOf(groups(card)[0])[0].click()
  await settle()
  const answer = app.answer!
  expect(answer.openDecisions).toBe(0)
  // The card becomes the plan (§4.5): its sentence is the card's heading,
  // its amount under it, and no lines or decisions are left on it.
  expect(card.querySelector('#answer-title')?.textContent).toBe(answer.plan)
  expect(card.querySelector('.plan .amount')?.textContent).toBe(answer.planAmount!.amount)
  expect(card.querySelector('.plan .when')?.textContent).toBe(answer.planAmount!.when)
  expect(card.querySelector('.plan .basis')?.textContent).toContain(answer.planAmount!.basis)
  expect(card.querySelector('.plan-label')?.textContent).toBe(t.thePlan)
  expect(card.querySelector('.left')).toBeNull()
  expect(card.querySelectorAll('.line')).toHaveLength(0)
  expect(groups(card)).toHaveLength(0)
  expect(card.querySelector('.to-how')?.textContent).toBe(t.toHow)
})

const steps = (card: HTMLElement) => [...card.querySelectorAll<HTMLElement>('.path .step')]
const stepOf = (card: HTMLElement, key: string) => card.querySelector<HTMLElement>(`.path [data-step="${key}"]`)!

/** The salary with every decision made, the cushion already there. */
async function salaryPlanned() {
  const started = await start('salary')
  started.app.choose({ Cushion: 'Has' })
  started.app.choose({ Wrapper: 'Anyone' })
  await settle()
  started.app.choose({ Account: 'NewAccount' })
  await settle()
  return started
}

test('the path is the core’s steps, in its order, each kind looking as §4.5 says', async () => {
  const { app, card } = await salaryPlanned()
  const path = app.answer!.path
  expect(app.answer!.plan).toBeDefined()
  expect(steps(card).map((step) => step.querySelector('.label span')?.textContent)).toEqual(
    path.map((step) => step.label),
  )
  for (const [at, step] of path.entries()) {
    const shown = steps(card)[at]
    expect(shown.querySelector('.choice')?.textContent).toBe(step.choice)
    expect(shown.querySelector('.hinge')?.textContent).toBe(step.hinge)
    const tag = shown.querySelector('.tag')?.textContent
    const change = shown.querySelector('button.change')
    const why = shown.querySelector('button.why')
    const node = shown.querySelector('.node')!
    if (step.kind === 'Visitor') {
      expect(node.classList.contains('filled')).toBe(true)
      expect(tag).toBeUndefined()
      expect(change?.textContent).toBe(t.stepChange)
      expect(change?.getAttribute('aria-expanded')).toBe('false')
    } else if (step.kind === 'Site') {
      expect(node.classList.contains('filled')).toBe(true)
      expect(tag).toBe(t.linePill[step.pill!])
      expect(change).toBeNull()
      expect(why === null).toBe(step.whyClear === undefined)
    } else {
      // "איזה מדד": an empty circle, "בחירה שלכם", and nothing to press.
      expect(step.kind).toBe('Outside')
      expect(node.classList.contains('filled')).toBe(false)
      expect(tag).toBe(t.stepYours)
      expect(shown.querySelectorAll('button, a')).toHaveLength(0)
      expect(shown.textContent).not.toContain('?')
    }
  }
  expect(path.some((step) => step.kind === 'Outside')).toBe(true)
})

test('"למה זה ברור?" opens its explanation in place, and closes it', async () => {
  const { app, card } = await salaryPlanned()
  const step = app.answer!.path.find((each) => each.whyClear)!
  const shown = stepOf(card, step.line!)
  const button = shown.querySelector<HTMLButtonElement>('button.why')!
  expect(button.textContent).toBe(t.whyClear)
  expect(shown.querySelector('.why-clear')).toBeNull()
  button.click()
  flushSync()
  const why = shown.querySelector('.why-clear')!
  expect(button.getAttribute('aria-expanded')).toBe('true')
  expect(button.textContent).toBe(t.whyClearHide)
  const clear = step.whyClear!
  for (const words of [clear.lead, clear.count, clear.crises, clear.past]) expect(why.textContent).toContain(words)
  expect([...why.querySelectorAll('dd')].map((each) => each.textContent)).toEqual(
    clear.amounts.map((figure) => figure.amount),
  )
  button.click()
  await vi.waitFor(() => expect(shown.querySelector('.why-clear')).toBeNull())
})

test('"שינוי" opens the decision in place of its step; choosing closes it and the plan changes', async () => {
  const { app, card } = await salaryPlanned()
  const amount = app.answer!.planAmount!.amount
  const shown = stepOf(card, 'Account')
  const change = shown.querySelector<HTMLButtonElement>('button.change')!
  change.click()
  flushSync()
  expect(change.getAttribute('aria-expanded')).toBe('true')
  expect(change.textContent).toBe(t.stepClose)
  // The one decision component, in the step, the choice made pressed.
  const [group] = groups(card)
  expect(shown.contains(group)).toBe(true)
  expect(shown.querySelector('.choice')).toBeNull()
  const [fresh, bank] = buttonsOf(group)
  expect(fresh.getAttribute('aria-pressed')).toBe('true')
  bank.click()
  await settle()
  expect(app.choices.account).toBe('YourBank')
  expect(groups(card)).toHaveLength(0)
  expect(stepOf(card, 'Account').querySelector('.choice')?.textContent).toBe(
    app.answer!.path.find((step) => step.reopens === 'Account')!.choice,
  )
  expect(card.querySelector('#answer-title')?.textContent).toBe(app.answer!.plan)
  // The amount changed where it is, and what it was stays in sight.
  expect(app.answer!.planAmount!.amount).not.toBe(amount)
  expect(card.querySelector('.plan .before')?.textContent).toBe(t.beforeChange(amount))
  // The option already chosen, chosen again, just closes it.
  stepOf(card, 'Account').querySelector<HTMLButtonElement>('button.change')!.click()
  flushSync()
  buttonsOf(groups(card)[0])[1].click()
  await settle()
  expect(groups(card)).toHaveLength(0)
  expect(app.choices.account).toBe('YourBank')
  // Pressed again, "סגירה" closes it without a change.
  const again = stepOf(card, 'Account').querySelector<HTMLButtonElement>('button.change')!
  again.click()
  flushSync()
  again.click()
  flushSync()
  expect(groups(card)).toHaveLength(0)
})

test('a change on the path that opens a decision keeps the path, with that decision open in it', async () => {
  const { app, card } = await start('bonus')
  buttonsOf(groups(card)[0])[1].click()
  await settle()
  expect(app.answer!.plan, 'growth: nothing left to decide').toBeDefined()
  stepOf(card, 'PlaceType').querySelector<HTMLButtonElement>('button.change')!.click()
  flushSync()
  // Safe: which safe way is still to decide (principle 8).
  buttonsOf(groups(card)[0])[0].click()
  await settle()
  expect(app.answers.flexibility).toBe('Fixed')
  expect(app.answer!.plan).toBeUndefined()
  expect(card.querySelector('.path')).not.toBeNull()
  expect(card.querySelectorAll('.line')).toHaveLength(0)
  const open = app.answer!.path.find((step) => step.kind === 'Open')!
  const shown = stepOf(card, open.reopens!)
  expect(shown.querySelector('.tag')?.textContent).toBe(t.pillYourCall)
  const [group] = groups(card)
  expect(shown.contains(group)).toBe(true)
  buttonsOf(group)[0].click()
  await settle()
  expect(app.answer!.plan).toBeDefined()
  expect(card.querySelector('#answer-title')?.textContent).toBe(app.answer!.plan)
  // A field changed: the facts are new, and the card is the lines again.
  app.answers.now = 20000
  await settle()
  expect(app.pathHeld).toBe(false)
})

test('the cushion’s "not yet" on the path asks the expenses under its step', async () => {
  const { app, card } = await salaryPlanned()
  stepOf(card, 'Cushion').querySelector<HTMLButtonElement>('button.change')!.click()
  flushSync()
  buttonsOf(groups(card)[0])[1].click()
  await settle()
  expect(app.answers.cushion).toEqual({ kind: 'hasNot', monthlyExpenses: null })
  const field = stepOf(card, 'Cushion').querySelector<HTMLInputElement>('#expenses')
  expect(field, 'the field sits under the cushion’s step').not.toBeNull()
  field!.value = '10000'
  field!.dispatchEvent(new Event('input', { bubbles: true }))
  await settle()
  expect(app.answers.cushion).toEqual({ kind: 'hasNot', monthlyExpenses: 10000 })
})

test("the monthly expenses are asked only once the cushion's “not yet” is chosen, and typing them answers it", async () => {
  const { app, card } = await start('salary')
  expect(card.querySelector('#expenses'), 'no field before "not yet" is chosen').toBeNull()
  app.choose({ Cushion: 'HasNot' })
  await settle()
  const field = card.querySelector<HTMLInputElement>('#expenses')
  expect(field, 'the field sits in the cushion decision').not.toBeNull()
  expect(field!.closest('[role=group]')).not.toBeNull()
  // Not typed yet: no amount is assumed.
  expect(app.answers.cushion).toEqual({ kind: 'hasNot', monthlyExpenses: null })
  field!.value = '10000'
  field!.dispatchEvent(new Event('input', { bubbles: true }))
  await settle()
  expect(app.answers.cushion).toEqual({ kind: 'hasNot', monthlyExpenses: 10000 })
  // "I have one" takes the field away again.
  app.choose({ Cushion: 'Has' })
  await settle()
  await vi.waitFor(() => expect(card.querySelector('#expenses')).toBeNull())
})
