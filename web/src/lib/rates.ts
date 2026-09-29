import { t } from './text'

/** What a dollar and a euro cost in shekels. */
export interface Rates {
  ilsPerUsd: number
  ilsPerEur: number
}

/** Rates to start with, until today's arrive (or if they can't be fetched). */
export const DEFAULT_RATES: Rates = { ilsPerUsd: 3.7, ilsPerEur: 4.3 }

/**
 * Today's European Central Bank reference rates, from Frankfurter: a free
 * service that republishes the ECB's rates and allows browsers to fetch them
 * (the ECB's own files don't).
 */
export async function fetchRates(): Promise<Rates & { date: string }> {
  const response = await fetch('https://api.frankfurter.dev/v1/latest?base=EUR&symbols=USD,ILS')
  if (!response.ok) {
    throw new Error(t.ratesServiceAnswered(response.status))
  }
  const { date, rates } = (await response.json()) as {
    date: string
    rates: { USD: number; ILS: number }
  }
  // The dollar's rate is derived, so it has many digits; four are plenty.
  const ilsPerUsd = Math.round((rates.ILS / rates.USD) * 10_000) / 10_000
  return { ilsPerEur: rates.ILS, ilsPerUsd, date }
}

let pending: Promise<Rates & { date: string }> | undefined

/** Today's rates, asked for once: main.ts asks as early as it can, while the
 * core still downloads, and the app gets the same answer. */
export function todaysRates(): Promise<Rates & { date: string }> {
  return (pending ??= fetchRates())
}
