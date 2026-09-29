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

const dayMonthYear = new Intl.DateTimeFormat('en-GB', { day: '2-digit', month: '2-digit', year: 'numeric' })

/** "28/09/2026", as Israelis write dates, from an ISO date ("2026-09-28"). */
export const dateText = (iso: string): string => dayMonthYear.format(new Date(`${iso}T00:00:00`))

/** "₪1.42M", "₪889K", "₪42K": short enough for chart labels. */
export const compactShekels = (amount: number): string => compact.format(amount)

/** Black or white, whichever reads better on `color` (a "#rrggbb"). */
export function readableOn(color: string): string {
  const [r, g, b] = [1, 3, 5].map((start) => parseInt(color.slice(start, start + 2), 16))
  return 0.299 * r + 0.587 * g + 0.114 * b > 140 ? '#000' : '#fff'
}

/** "After 15 years, 5 months": a chart's time, from a count of years. */
export function elapsed(years: number): string {
  const months = Math.round(years * 12)
  const plural = (count: number, unit: string) => `${count} ${unit}${count === 1 ? '' : 's'}`
  const whole = plural(Math.floor(months / 12), 'year')
  return months % 12 === 0 ? `After ${whole}` : `After ${whole}, ${plural(months % 12, 'month')}`
}

const twoDecimals = new Intl.NumberFormat('en-IL', {
  style: 'percent',
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
})

const roundPercent = new Intl.NumberFormat('en-IL', {
  style: 'percent',
  maximumSignificantDigits: 2,
})

/** "0.42%", "12.50%": a yearly cost, from a number of percent (0.42). */
export const percent = (value: number): string => twoDecimals.format(value / 100)

/** "0.1%", "1%", "10%", "0.42%": short enough for an axis, from a number of
 * percent. */
export const compactPercent = (value: number): string => roundPercent.format(value / 100)
