// Amounts in shekels, formatted by the browser's Intl.

import { t } from './text'

const whole = new Intl.NumberFormat('en-IL', {
  style: 'currency',
  currency: 'ILS',
  maximumFractionDigits: 0,
})

/** Compact amounts at three significant digits, then four and five. The
 * longer ones keep their zeros ("₪1.510M"), so a column of them reads
 * evenly; at three, the usual "₪1.5M". */
const compact = [3, 4, 5].map(
  (digits) =>
    new Intl.NumberFormat('en-IL', {
      style: 'currency',
      currency: 'ILS',
      notation: 'compact',
      minimumSignificantDigits: digits > 3 ? digits : 1,
      maximumSignificantDigits: digits,
    }),
)

/** "₪1,415,944" */
export const shekels = (amount: number): string => whole.format(amount)

const dayMonthYear = new Intl.DateTimeFormat('en-GB', { day: '2-digit', month: '2-digit', year: 'numeric' })

/** "28/09/2026", as Israelis write dates, from an ISO date ("2026-09-28"). */
export const dateText = (iso: string): string => dayMonthYear.format(new Date(`${iso}T00:00:00`))

/** "₪1.42M", "₪889K", "₪42K": short enough for chart labels. */
export const compactShekels = (amount: number): string => compact[0].format(amount)

/** Of `formats`, from the shortest, the first that prints `values` that
 * differ differently: lines ending at ₪103,196 and ₪102,930 both read
 * "₪103K" at three digits. "Differ" is as the last format sees it. */
function fewestApart(values: number[], formats: Intl.NumberFormat[]): (value: number) => string {
  const kinds = (format: Intl.NumberFormat) => new Set(values.map((value) => format.format(value))).size
  const most = kinds(formats.at(-1)!)
  const format = formats.find((format) => kinds(format) === most)!
  return (value) => format.format(value)
}

/** Compact amounts, with a digit more (up to five) wherever two of `amounts`
 * would otherwise read the same: "₪103.20K", "₪102.93K". For labels that
 * stand side by side, such as where lines end. */
export const compactShekelsApart = (amounts: number[]): ((amount: number) => string) =>
  fewestApart(amounts, compact)

/** Black or white, whichever reads better on `color` (a "#rrggbb"). */
export function readableOn(color: string): string {
  const [r, g, b] = [1, 3, 5].map((start) => parseInt(color.slice(start, start + 2), 16))
  return 0.299 * r + 0.587 * g + 0.114 * b > 140 ? '#000' : '#fff'
}

/** "After 15 years, 5 months": a chart's time, from a count of years. */
export function elapsed(years: number): string {
  const months = Math.round(years * 12)
  return t.after(Math.floor(months / 12), months % 12)
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

const percentTo = [2, 3, 4].map(
  (decimals) =>
    new Intl.NumberFormat('en-IL', {
      style: 'percent',
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    }),
)

/** Yearly costs as `percent` writes them, with a decimal more (up to four)
 * wherever two of `values` would otherwise read the same: "0.112%". */
export function percentApart(values: number[]): (value: number) => string {
  const format = fewestApart(
    values.map((value) => value / 100),
    percentTo,
  )
  return (value) => format(value / 100)
}

/** "0.1%", "1%", "10%", "0.42%": short enough for an axis, from a number of
 * percent. */
export const compactPercent = (value: number): string => roundPercent.format(value / 100)
