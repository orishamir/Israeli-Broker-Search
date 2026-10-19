// Names for the guide's kinds and ways that links and lists can hold as
// plain text. The core is the one that says what they are; these only turn
// a value into a string and back, checking against what the core lists.

import * as core from './core/core'
import type { Arena, Kind, Way } from './core/core'

/** A way as text, stable across versions: "IsraeliEtf|Fund|Usa". */
export const wayKey = (way: Way): string => `${way.product}|${String(way.vehicle)}|${way.exchange ?? ''}`

/** The listed way with this key, if the core still lists it. */
export const wayByKey = (key: string): Way | undefined =>
  core.ways().find(({ way }) => wayKey(way) === key)?.way

/** A kind as text, for links and for telling kinds apart in lists. */
export function kindKey(kind: Kind): string {
  if (typeof kind === 'string') return kind
  if ('GovernmentBond' in kind) return kind.GovernmentBond.linked ? 'GovernmentBond:linked' : 'GovernmentBond'
  return `Way:${wayKey(kind.Way)}`
}

/** The kind `kindKey` made `key` from, if it names one the core lists. */
export function kindByKey(key: string): Kind | undefined {
  if (
    key === 'MoneyMarketFund' ||
    key === 'FixedDeposit' ||
    key === 'LinkedDeposit' ||
    key === 'CurrentAccount'
  )
    return key
  if (key === 'GovernmentBond') return { GovernmentBond: { linked: false } }
  if (key === 'GovernmentBond:linked') return { GovernmentBond: { linked: true } }
  if (key.startsWith('Way:')) {
    const way = wayByKey(key.slice('Way:'.length))
    return way && { Way: way }
  }
  return undefined
}

/** The arena a kind belongs to: the index arena's kinds are ways; the
 * current account belongs to neither. */
export function arenaOf(kind: Kind): Arena | undefined {
  if (kind === 'CurrentAccount') return undefined
  return typeof kind === 'object' && 'Way' in kind ? 'Index' : 'Safe'
}
