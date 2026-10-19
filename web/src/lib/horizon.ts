// The horizon field: a number and a unit menu (months or years), over the
// months the state keeps.

export type Unit = 'months' | 'years'

/** The unit a horizon reads best in: whole years, otherwise months. */
export const defaultUnit = (months: number | undefined): Unit =>
  months !== undefined && months > 0 && months % 12 !== 0 ? 'months' : 'years'

/** The number the field shows; empty while the horizon is unknown. */
export function shown(months: number | undefined, unit: Unit): number | null {
  if (months === undefined) return null
  return unit === 'years' ? Math.round((months / 12) * 100) / 100 : months
}

/** The months a typed number means; null while the field is empty. */
export function monthsOfTyped(value: number | null, unit: Unit): number | null {
  if (value === null) return null
  return Math.max(0, Math.round(unit === 'years' ? value * 12 : value))
}
