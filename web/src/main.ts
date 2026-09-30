import { mount } from 'svelte'
// Evens out browser defaults first; the app's own styles go on top.
import 'modern-normalize/modern-normalize.css'
import './app.css'
import App from './App.svelte'
import init, { setLang } from './lib/core/core'
import { reducedMotion } from './lib/motion'
import { todaysRates } from './lib/rates'
import { t } from './lib/text'

// Asked for now, so the answer comes while the core downloads; the app
// handles a failure when it looks.
todaysRates().catch(() => undefined)

// Loads and compiles the Rust core (WebAssembly) once; every call after
// that is synchronous.
await init()

// The core answers in Hebrew from here on.
setLang('He')
document.title = t.title

mount(App, { target: document.getElementById('app')! })

// The page's shape, drawn by index.html while all this loaded, fades away
// over the app.
const skeleton = document.getElementById('skeleton')
skeleton?.classList.add('done')
setTimeout(() => skeleton?.remove(), reducedMotion ? 0 : 250)
