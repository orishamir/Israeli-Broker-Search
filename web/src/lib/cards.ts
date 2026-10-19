import type { Term } from './core/core'

/** The context key under which App.svelte hands every `Tip` the function
 * that opens a learn card, so a tip needn't be given the whole state. */
export const OPEN_CARD = Symbol('open-card')
export type OpenCard = (term: Term) => void
