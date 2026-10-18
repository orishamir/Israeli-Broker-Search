// Numbers as the fields show and read them: "10,000", "0.16".

/** With thousands separators and up to four decimals; empty for no number. */
export const formatNumber = (number: number | null): string =>
  number === null ? '' : number.toLocaleString('en-US', { maximumFractionDigits: 4 })

/** The number typed into a field; null while the field is empty or isn't
 * a number. Spaces and a ₪, $, € or % pasted in are ignored, and so are
 * commas that group thousands ("25,000"). A lone comma before one or two
 * digits ("3,25") is a decimal point, as many keyboards type it; any other
 * comma makes it no number, rather than a guess a hundred times too big.
 * Only plain decimals are read: not "0x10", "1e6" or "Infinity", which
 * `Number` would take. */
export function parseNumber(text: string): number | null {
  const cleaned = text.replaceAll(/[\s₪$€%]/gu, '')
  if (cleaned === '') return null
  const decimalComma = /^(-?\d+),(\d{1,2})$/u.exec(cleaned)
  const plain = decimalComma
    ? `${decimalComma[1]}.${decimalComma[2]}`
    : /^-?\d{1,3}(,\d{3})+(\.\d*)?$/u.test(cleaned)
      ? cleaned.replaceAll(',', '')
      : cleaned
  return /^-?(\d+\.?\d*|\.\d+)$/u.test(plain) ? Number(plain) : null
}

/** `value` moved one `step` up or down, rounded so that 0.1 steps don't
 * drift to 0.30000000000000004. An empty field steps from 0. */
export const stepped = (value: number | null, step: number, direction: 1 | -1): number =>
  Math.round(((value ?? 0) + direction * step) * 1e6) / 1e6
