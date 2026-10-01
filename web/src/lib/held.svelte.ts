import { untrack } from 'svelte'
import { drawAt } from './glide'
import { lastTyped } from './typing'

/** A value that, while a number is being typed, keeps what it was and shows
 * the latest once the typing pauses, when the charts draw (`drawAt`): words
 * that change length with every keystroke moved the page each time. Made
 * while a component starts, as its effect needs one. */
export class HeldWhileTyping<T> {
  current = $state.raw<T>() as T

  constructor(source: () => T, same: (a: T, b: T) => boolean = Object.is) {
    this.current = untrack(source)
    let firstWaiting: number | null = null
    let timer: ReturnType<typeof setTimeout> | undefined
    const settle = () => {
      const now = performance.now()
      const at = drawAt({ now, settled: now, lastTyped: lastTyped(), firstWaiting: firstWaiting ?? now })
      if (at > now) {
        timer = setTimeout(settle, at - now)
        return
      }
      this.current = untrack(source)
      firstWaiting = null
    }
    $effect(() => {
      const next = source()
      untrack(() => {
        clearTimeout(timer)
        if (same(next, this.current)) {
          firstWaiting = null
          return
        }
        firstWaiting ??= performance.now()
        settle()
      })
    })
    $effect(() => () => clearTimeout(timer))
  }
}
