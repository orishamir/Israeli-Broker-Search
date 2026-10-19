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
  // And the choice can be changed.
  growth.click()
  await settle()
  expect(growth.getAttribute('aria-pressed')).toBe('true')
  expect(safe.getAttribute('aria-pressed')).toBe('false')
  expect(app.answers.flexibility).toBe('CanWait')
})

test('the plan shows once every decision is made, and the count until then', async () => {
  const { app, card } = await start('goal')
  expect(card.querySelector('.left')?.textContent).toBe(t.decisionsLeft(app.answer!.openDecisions))
  expect(card.querySelector('.plan')).toBeNull()
  buttonsOf(groups(card)[0])[0].click()
  await settle()
  expect(app.answer!.openDecisions).toBe(0)
  expect(card.querySelector('.plan p')?.textContent).toBe(app.answer!.plan)
  expect(card.querySelector('.left')).toBeNull()
})

test("typing the monthly expenses in the cushion's “not yet” answers it", async () => {
  const { app, card } = await start('salary')
  const field = card.querySelector<HTMLInputElement>('#expenses')
  expect(field, 'the field sits in the cushion decision').not.toBeNull()
  expect(field!.closest('[role=group]')).not.toBeNull()
  field!.value = '10000'
  field!.dispatchEvent(new Event('input', { bubbles: true }))
  await settle()
  expect(app.answers.cushion).toEqual({ kind: 'hasNot', monthlyExpenses: 10000 })
})
