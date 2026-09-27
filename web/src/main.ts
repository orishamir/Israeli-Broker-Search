import { mount } from 'svelte'
// Evens out browser defaults first; the app's own styles go on top.
import 'modern-normalize/modern-normalize.css'
import './app.css'
import App from './App.svelte'
import init from './lib/core/core'

// Loads and compiles the Rust core (WebAssembly) once; every call after
// that is synchronous.
await init()

mount(App, { target: document.getElementById('app')! })
