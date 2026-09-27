// Amounts in shekels, formatted by the browser's Intl.

const whole = new Intl.NumberFormat('en-IL', {
  style: 'currency',
  currency: 'ILS',
  maximumFractionDigits: 0,
})

const compact = new Intl.NumberFormat('en-IL', {
  style: 'currency',
  currency: 'ILS',
  notation: 'compact',
  maximumSignificantDigits: 3,
})

/** "₪1,415,944" */
export const shekels = (amount: number): string => whole.format(amount)

/** "₪1.42M", "₪889K", "₪42K": short enough for chart labels. */
export const compactShekels = (amount: number): string => compact.format(amount)

/** Black or white, whichever reads better on `color` (a "#rrggbb"). */
export function readableOn(color: string): string {
  const [r, g, b] = [1, 3, 5].map((start) => parseInt(color.slice(start, start + 2), 16))
  return 0.299 * r + 0.587 * g + 0.114 * b > 140 ? '#000' : '#fff'
}
