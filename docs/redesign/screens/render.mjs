// Renders each mock screen to a PNG: node render.mjs <mock.html> <out dir>
import { chromium } from '/home/ori/dev/broker-search/web/node_modules/playwright/index.mjs'
import { mkdirSync } from 'node:fs'
import { resolve } from 'node:path'

const [html, out] = process.argv.slice(2)
mkdirSync(out, { recursive: true })
const browser = await chromium.launch()
const page = await browser.newPage({ viewport: { width: 1300, height: 900 }, deviceScaleFactor: 2 })
page.on('pageerror', (e) => console.error('page error:', e.message))
await page.goto('file://' + resolve(html))
await page.waitForFunction(() => window.__ready === true)
await page.waitForTimeout(300)
for (const id of await page.$$eval('.shot', (els) => els.map((e) => e.id))) {
  await page.locator('#' + id).screenshot({ path: `${out}/${id}.png` })
  console.log('wrote', id)
}
await browser.close()
