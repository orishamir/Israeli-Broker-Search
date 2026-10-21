// The questions card's fields as the visitor sees them, and the examples
// that fill them (REDESIGN.md §4.2, principle 19). An example is a starting
// point: it fills only these fields, and the page calls the numbers an
// example exactly while the fields still hold its numbers.

import type { AnswersInput, Horizon, SituationData } from './core/core'
import { shekels } from './format'
import { t } from './text'

/** "מתי תצטרכו את הכסף?": the four answers of the 2×2 group. */
export type When = 'date' | 'retirement' | 'anyDay' | 'unknown'
export const WHENS: readonly When[] = ['date', 'retirement', 'anyDay', 'unknown']

/** What the questions card shows: every field on screen, and nothing else.
 * Empty fields are null. The date's months count only while "בעוד זמן
 * ידוע" is chosen, and the ages only while "בפרישה" is. */
export interface Fields {
  now: number | null
  monthly: number | null
  when: When
  months: number | null
  age: number | null
  retireAt: number | null
  /** "הכסף הזה כבר מושקע, בתיק בבנק שלי". */
  invested: boolean
}

/** What the core's answers say about the fields on screen. A horizon of
 * months with "money needed any day" is the any-day answer: the months
 * stay in the date's field, which it doesn't show. */
export function fieldsOf(answers: AnswersInput): Fields {
  const { horizon } = answers
  const months = typeof horizon === 'object' && 'Months' in horizon ? horizon.Months : null
  const ages = typeof horizon === 'object' && 'Retirement' in horizon ? horizon.Retirement : undefined
  const when: When = answers.needsAnyDay
    ? 'anyDay'
    : ages
      ? 'retirement'
      : horizon === 'Unknown'
        ? 'unknown'
        : 'date'
  return {
    now: answers.now,
    monthly: answers.monthly,
    when,
    months,
    age: ages?.age ?? null,
    retireAt: ages?.retire_at ?? null,
    invested: typeof answers.situation === 'object' && 'ExistingPortfolio' in answers.situation,
  }
}

/** The horizon the core is asked for what the fields say. An empty field
 * asks for 0, which the core says can't be worked out, so the page keeps
 * the last answer, faded, under its words. Money needed any day has no
 * date: the field that would say one isn't on screen. */
export function horizonOf(fields: Fields): Horizon {
  switch (fields.when) {
    case 'date':
      return { Months: fields.months ?? 0 }
    case 'retirement':
      return { Retirement: { age: fields.age ?? 0, retire_at: fields.retireAt ?? 0 } }
    default:
      return 'Unknown'
  }
}

/** Whether two sets of fields say the same: what isn't on screen (the
 * date while retirement is chosen, the unit it's shown in) doesn't count. */
export function sameFields(a: Fields, b: Fields): boolean {
  if (a.now !== b.now || a.monthly !== b.monthly || a.when !== b.when || a.invested !== b.invested)
    return false
  if (a.when === 'date') return a.months === b.months
  if (a.when === 'retirement') return a.age === b.age && a.retireAt === b.retireAt
  return true
}

/** The examples the "דוגמאות" list offers, in the core's order: every
 * situation but the workbench's, which is the link under the fields. */
export const examplesOf = (situations: SituationData[]): SituationData[] =>
  situations.filter((each) => !each.workbench)

/** An example's numbers, in one line: "₪100,000 ועוד ₪2,000 בחודש, 3 שנים". */
export function numbersLine(fields: Fields): string {
  const now = fields.now ? shekels(fields.now) : undefined
  const monthly = fields.monthly ? shekels(fields.monthly) : undefined
  const when =
    fields.when === 'date'
      ? t.exampleIn(fields.months ?? 0)
      : fields.when === 'retirement'
        ? t.exampleRetirement(fields.age ?? 0, fields.retireAt ?? 0)
        : fields.when === 'anyDay'
          ? t.whenAnyDay
          : t.exampleNoDate
  return `${t.exampleMoney(now, monthly, fields.invested)}, ${when}`
}
