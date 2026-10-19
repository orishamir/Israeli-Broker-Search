import type { LearnCard, Term } from './core/core'

/** The glossary's card for `term`: what a "?" or a "מה זה?" opens. */
export const cardOf = (cards: readonly LearnCard[], term: Term): LearnCard | undefined =>
  cards.find((card) => card.term === term)
