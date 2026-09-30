// The page is shown in Hebrew, right to left. The English texts stay beside
// the Hebrew ones (`text/en.ts`, and every `Text` in the core): the page
// shows English names under the Hebrew ones, and links and saved plans name
// plans in English.

import type { Text } from './text/en'
import { he } from './text/he'

/** Every text the web side shows. */
export const t: Text = he
