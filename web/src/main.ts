import { mount } from 'svelte'
// Evens out browser defaults first; the app's own styles go on top.
import 'modern-normalize/modern-normalize.css'
import './app.css'
import App from './App.svelte'
import init, { setLang } from './lib/core/core'
import { reducedMotion } from './lib/motion'
import { todaysRates } from './lib/rates'
import { warmUp } from './lib/sweeper'
import { t } from './lib/text'

// Asked for now, so the answer comes while the core downloads; the app
// handles a failure when it looks.
todaysRates().catch(() => undefined)
// The worker loads its own copy of the core while the page loads.
warmUp()

// Loads and compiles the Rust core (WebAssembly) once; every call after
// that is synchronous.
try {
  await init()
} catch (error) {
  couldntLoad()
  throw error
}

// The core answers in Hebrew from here on.
setLang('He')
document.title = t.title

mount(App, { target: document.getElementById('app')! })

// The page's shape, drawn by index.html while all this loaded, fades away
// over the app.
const skeleton = document.getElementById('skeleton')
skeleton?.classList.add('done')
setTimeout(() => skeleton?.remove(), reducedMotion ? 0 : 250)

/** Says where the page's shape pulses that the app couldn't load, rather
 * than pulsing on: the core didn't download (a connection lost midway). */
function couldntLoad() {
  const skeleton = document.getElementById('skeleton')
  const layout = skeleton?.querySelector('.layout')
  if (!skeleton || !layout) return
  skeleton.removeAttribute('aria-hidden')
  skeleton.classList.add('failed')
  const message = document.createElement('p')
  message.className = 'card'
  message.setAttribute('role', 'alert')
  message.textContent = t.couldntLoad
  layout.replaceWith(message)
}
