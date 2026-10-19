import { createRawSnippet, flushSync, mount, unmount } from 'svelte'
import { beforeEach, expect, test, vi } from 'vitest'
import { AppState } from './app.svelte'
import { OPEN_CARD } from './cards'
import * as core from './core/core'
import type { Term } from './core/core'
import { cardOf } from './glossary'
import Tip from './Tip.svelte'

beforeEach(() => {
  localStorage.clear()
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => Response.json({ date: '2026-09-25', rates: { ILS: 3.4594, USD: 1.1403 } })),
  )
})

function start(): AppState {
  let app!: AppState
  $effect.root(() => {
    app = new AppState({})
  })
  flushSync()
  return app
}

test('every term of the glossary opens its own card, and its comparisons open too', () => {
  const app = start()
  expect(app.glossary.length).toBeGreaterThan(20)
  for (const card of app.glossary) {
    expect(cardOf(app.glossary, card.term)).toBe(card)
    app.closeCard()
    app.openCard(card.term)
    expect(app.cardTerm).toBe(card.term)
    expect(app.cardFor(card.term).name).toBe(card.name)
    expect(card.name).not.toBe('')
    for (const other of card.compareWith) {
      expect(cardOf(app.glossary, other), `${card.term} compares with ${other}`).toBeDefined()
    }
  }
})

test('a card opened from a card stacks, and back returns to the first', () => {
  const app = start()
  const [first, second] = app.glossary
  app.openCard(first.term)
  app.openCard(second.term)
  expect(app.cardStack).toEqual([first.term, second.term])
  app.cardBack()
  expect(app.cardTerm).toBe(first.term)
  app.closeCard()
  expect(app.cardTerm).toBeUndefined()
})

test('the kinds the core says have a card are in the glossary', () => {
  const app = start()
  for (const ranked of app.recommendation?.ranked ?? []) {
    if (ranked.kind.card) expect(cardOf(app.glossary, ranked.kind.card)).toBeDefined()
  }
  expect(core.glossary().length).toBe(app.glossary.length)
})

/** A tip with `card`, as a component would use it, in a context that opens cards. */
function tipWith(card: Term | undefined) {
  const opened: Term[] = []
  const target = document.createElement('div')
  document.body.append(target)
  const children = createRawSnippet(() => ({ render: () => '<p>words</p>' }))
  const component = mount(Tip, {
    target,
    props: { about: 'x', card, children },
    context: new Map([[OPEN_CARD, (term: Term) => opened.push(term)]]),
  })
  flushSync()
  return { target, opened, done: () => (unmount(component), target.remove()) }
}

test('a tip with a card has "עוד על זה ›", which opens the card', () => {
  const { target, opened, done } = tipWith('etf')
  const more = target.querySelector<HTMLButtonElement>('.popover .more')
  expect(more?.textContent).toBe('עוד על זה ›')
  more?.click()
  expect(opened).toEqual(['etf'])
  done()
})

test('a tip without a card has no link', () => {
  const { target, done } = tipWith(undefined)
  expect(target.querySelector('.more')).toBeNull()
  done()
})
