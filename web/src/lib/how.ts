// "איך עושים את זה" and where to buy (REDESIGN.md §2.8, §2.9): picking the
// row's How out of the answer, grouping the places into ties, and the
// marks of a placeholder. The words and the figures are the core's.
import type { AnswerData, HowData, Kind, ProviderData } from './core/core'
import { shekels } from './format'
import { kindKey } from './kinds'
import { t } from './text'

/** The How of the row the page shows, if the answer has one for it. */
export function howFor(answer: AnswerData | undefined, row: Kind | undefined): HowData | undefined {
  if (!answer || row === undefined) return undefined
  const key = kindKey(row)
  return answer.how.find((how) => kindKey(how.row) === key)
}

/** Places that tie, in the core's order: a group per tie number, so the
 * page draws a tie as one box and never ranks inside it (principle 13). */
export function tieGroups(rows: ProviderData[]): ProviderData[][] {
  const groups: ProviderData[][] = []
  for (const row of rows) {
    const last = groups.at(-1)
    if (last && last[0].tie === row.tie) last.push(row)
    else groups.push([row])
  }
  return groups
}

/** What a row shows against the visitor's bank: "+₪2,994", "−₪597", the
 * same, or that it is the reference itself. Empty when there's no bank. */
export function versusText(row: ProviderData): string {
  if (row.yours) return t.referenceRow
  const versus = row.versusYours?.today
  if (versus === undefined) return ''
  const rounded = Math.round(versus)
  if (rounded === 0) return t.sameAmount
  return `${rounded > 0 ? '+' : '−'}${shekels(Math.abs(rounded))}`
}

/** A sentence that opens with where it's about, "באינטראקטיב: בין …",
 * cut after that opening, to set it in bold. */
export function leadOf(sentence: string): { lead: string; rest: string } {
  const colon = sentence.indexOf(': ')
  return colon < 0
    ? { lead: '', rest: sentence }
    : { lead: sentence.slice(0, colon + 1), rest: sentence.slice(colon + 1) }
}

/** The marks the core uses for what the research hasn't filled yet
 * (principle 16): a bond series' number, a fund's name. */
const PLACEHOLDER = /(_{3,}|\[—\])/

/** A text cut into its plain parts and its placeholders, to mark them. */
export function placeholderParts(text: string): { text: string; placeholder: boolean }[] {
  return text
    .split(PLACEHOLDER)
    .filter((part) => part !== '')
    .map((part) => ({ text: part, placeholder: PLACEHOLDER.test(part) }))
}
