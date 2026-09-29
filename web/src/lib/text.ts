// The language the page is shown in, chosen once as the page loads: the one
// remembered from the switch, else the browser's, Hebrew for a Hebrew
// browser. Switching reloads the page, with the comparison kept in its
// address (see `Share`), so every text, the core's included, is asked for
// once in one language, and nothing needs to change under the user.

import { en, type Text } from './text/en'
import { he } from './text/he'

export type Lang = 'en' | 'he'

const KEY = 'lang'

function remembered(): Lang | null {
  try {
    const saved = localStorage.getItem(KEY)
    return saved === 'en' || saved === 'he' ? saved : null
  } catch {
    return null
  }
}

function browsers(): Lang {
  const languages = typeof navigator === 'undefined' ? [] : navigator.languages
  return languages.some((language) => language.toLowerCase().startsWith('he')) ? 'he' : 'en'
}

export const lang: Lang = remembered() ?? browsers()

/** Every text the web side shows, in the page's language. */
export const t: Text = lang === 'he' ? he : en

/** Hebrew runs right to left. */
export const rtl = lang === 'he'

/** As the core names the language. */
export const coreLang = lang === 'he' ? ('He' as const) : ('En' as const)

/** Remembers `next` and reloads the page in it, at `hash`: the comparison
 * as it is, so it survives the reload. */
export function switchLang(next: Lang, hash: string) {
  try {
    localStorage.setItem(KEY, next)
  } catch {
    // Not remembered: the browser's language decides next time.
  }
  location.hash = hash
  location.reload()
}
