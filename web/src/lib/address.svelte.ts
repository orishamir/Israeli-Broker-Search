// The page's address follows the saver: a refresh opens what was on screen,
// and Back steps through the goals and the two routes. Typing and dragging
// only rewrite the current entry (after a pause); choosing a goal, or the
// guide/workbench switch, adds one. The format is the Share button's link.

import type { AppState } from './app.svelte'
import { decodeLocation } from './link'

/** How long the address waits after the last change before it is rewritten. */
export const ADDRESS_PAUSE_MS = 400

/** Starts keeping the address; call it from a component's setup (it uses
 * effects) and it stops with the component. */
export function keepAddress(app: AppState) {
  let timer: ReturnType<typeof setTimeout> | undefined
  // What the effect saw last, to tell a new route from an edit.
  let route = `${app.situationId}|${app.expert}`
  let seen: string | undefined

  const here = () => location.hash.replace(/^#/, '')
  const replace = (hash: string) => {
    if (hash !== here()) history.replaceState(history.state, '', `#${hash}`)
  }

  $effect(() => {
    // Untouched, the example leaves the address as it was.
    if (app.exampleShown) {
      seen = undefined
      return
    }
    const hash = app.address()
    const now = `${app.situationId}|${app.expert}`
    clearTimeout(timer)
    if (now !== route) {
      // The entry we leave keeps what was last on it, even if a pause was pending.
      if (seen !== undefined) replace(seen)
      route = now
      seen = hash
      if (hash !== here()) history.pushState(history.state, '', `#${hash}`)
    } else {
      seen = hash
      timer = setTimeout(() => replace(hash), ADDRESS_PAUSE_MS)
    }
    return () => clearTimeout(timer)
  })

  $effect(() => {
    const back = () => {
      clearTimeout(timer)
      app.restore(decodeLocation(location))
      route = `${app.situationId}|${app.expert}`
      seen = undefined
    }
    addEventListener('popstate', back)
    return () => removeEventListener('popstate', back)
  })
}
