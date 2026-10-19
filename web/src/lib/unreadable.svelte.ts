import { SvelteSet } from 'svelte/reactivity'

/** The number fields whose text isn't a number (`-5000abc`), by id: the field
 * can only say "no value", which an empty field says too, so the page is told
 * and says why the answer isn't worked out for it (`AppState.inputError`). */
export const unreadable = new SvelteSet<string>()
