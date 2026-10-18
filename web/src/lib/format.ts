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

/** The browser's "K" and "M" in Hebrew words: "₪242 אלף", "₪1.42 מיליון". */
const inWords: Record<string, string> = { K: '\u00a0אלף', M: '\u00a0מיליון', B: '\u00a0מיליארד' }
const compactIn = (format: Intl.NumberFormat) => (amount: number) =>
  format.format(amount).replace(/[KMB]$/, (unit) => inWords[unit])

/** "₪1.42 מיליון", "₪889 אלף", "₪42 אלף": short enough for chart labels. */
export const compactShekels = (amount: number): string => compactIn(compact[0])(amount)

/** Of `formats`, from the shortest, the first that prints `values` that
 * differ differently: lines ending at ₪103,196 and ₪102,930 both read
 * "₪103K" at three digits. "Differ" is as the last format sees it. */
function fewestApart(values: number[], formats: Intl.NumberFormat[]): (value: number) => string {
  const kinds = (format: Intl.NumberFormat) => new Set(values.map((value) => format.format(value))).size
  const most = kinds(formats.at(-1)!)
  return formats.find((format) => kinds(format) === most)!.format
}

/** Compact amounts, with a digit more (up to five) wherever two of `amounts`
 * would otherwise read the same: "₪103.20K", "₪102.93K". For labels that
 * stand side by side, such as where lines end. */
export const compactShekelsApart = (amounts: number[]): ((amount: number) => string) => {
  const format = fewestApart(amounts, compact)
  return (amount) => format(amount).replace(/[KMB]$/, (unit) => inWords[unit])
}

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

/** A plan's yearly cost as `percent` writes it, but "less than 0.01%" for a
 * cost that rounds to nothing: "0.00%" read as no fees at all. */
export const yearlyCost = (value: number): string =>
  value > 0 && value < 0.005 ? t.underAHundredth : percent(value)

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

const asTyped = new Intl.NumberFormat('en-IL', { style: 'percent', maximumFractionDigits: 4 })

/** "3.25%", "12.5%", "2%": a percentage the user typed, as typed, from a
 * number of percent. Rounded to two digits, 3.25% read 3.3%. */
export const typedPercent = (value: number): string => asTyped.format(value / 100)
