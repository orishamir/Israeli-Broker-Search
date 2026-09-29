// Numbers as the fields show and read them: "10,000", "0.16".

/** With thousands separators and up to four decimals; empty for no number. */
export const formatNumber = (number: number | null): string =>
  number === null ? '' : number.toLocaleString('en-US', { maximumFractionDigits: 4 })

/** The number typed into a field, ignoring commas typed or left in; null
 * while the field is empty or isn't a number. */
export function parseNumber(text: string): number | null {
  const cleaned = text.replaceAll(',', '').trim()
  if (cleaned === '') return null
  const number = Number(cleaned)
  return Number.isFinite(number) ? number : null
}

/** `value` moved one `step` up or down, rounded so that 0.1 steps don't
 * drift to 0.30000000000000004. An empty field steps from 0. */
export const stepped = (value: number | null, step: number, direction: 1 | -1): number =>
  Math.round(((value ?? 0) + direction * step) * 1e6) / 1e6
