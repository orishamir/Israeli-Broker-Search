import { choice, expect, fitScreenToPage, test, type Page } from './fixtures'

// Runs on every device in playwright.config.ts, including the layout-only
// ones: rules that hold at any screen size, in every state of the page. The
// start state also gets a picture per device, with the numbers and charts
// masked out, so a tariff change doesn't change the picture.

/** What breaks the layout on this screen, in words; empty if nothing. */
const layoutProblems = (page: Page) =>
  page.evaluate(() => {
    const problems: string[] = []
    const name = (element: Element) =>
      `<${element.tagName.toLowerCase()} class="${element.className}"> "${element.textContent?.trim().slice(0, 30)}"`
    // Not hidden in any way, including in a closed <details> or popover.
    const shown = (element: Element) => element.checkVisibility({ visibilityProperty: true })

    const pageWidth = document.documentElement.scrollWidth
    if (pageWidth > innerWidth)
      problems.push(`the page scrolls sideways: ${pageWidth}px on a ${innerWidth}px screen`)

    // Inside boxes meant to scroll sideways (the table), sticking out is fine.
    const scrollsSideways = (element: Element, within: Element) => {
      for (let at = element.parentElement; at && at !== within; at = at.parentElement) {
        if (['auto', 'scroll', 'hidden'].includes(getComputedStyle(at).overflowX)) return true
      }
      return false
    }
    for (const box of document.querySelectorAll('.card, dialog[open]')) {
      if (!shown(box)) continue
      const { left, right } = box.getBoundingClientRect()
      for (const element of box.querySelectorAll('*')) {
        // A tip floats over the page; its own rule is below.
        if (!shown(element) || scrollsSideways(element, box) || element.closest('.popover')) continue
        const bounds = element.getBoundingClientRect()
        if (bounds.left < left - 1 || bounds.right > right + 1) {
          problems.push(`${name(element)} sticks out of ${name(box).slice(0, 40)}`)
        }
      }
    }

    // A box that clips (overflow hidden or clip) holding more than it shows:
    // text cut off inside it. Not form controls, which scroll their own
    // text, and not the charts, whose boxes hold ECharts' hidden tooltips.
    for (const element of document.querySelectorAll('*')) {
      if (!shown(element) || element.closest('.visually-hidden, .popover, .chart, .bars, .over-time'))
        continue
      if (element.matches('input, select, textarea')) continue
      const { overflowX, overflowY } = getComputedStyle(element)
      const clipsX = overflowX === 'hidden' || overflowX === 'clip'
      const clipsY = overflowY === 'hidden' || overflowY === 'clip'
      if (
        (clipsX && element.scrollWidth > element.clientWidth + 1) ||
        (clipsY && element.scrollHeight > element.clientHeight + 1)
      )
        problems.push(`${name(element)} cuts its content off`)
    }

    // Text drawn over other text: the line boxes of two pieces of text never
    // cross, unless one is part of the other. Not inside a sideways scroller,
    // where the sticky plan column is meant to cover what scrolls under it,
    // and not the phone's bar with the best plan, which floats over the page.
    const hasText = (element: Element) =>
      [...element.childNodes].some(
        (node) => node.nodeType === Node.TEXT_NODE && node.textContent!.trim() !== '',
      )
    const texts = [...document.querySelectorAll('body *')].filter(
      (element) =>
        hasText(element) &&
        shown(element) &&
        !element.closest('.visually-hidden, .popover, option, script, style, .best-bar') &&
        !scrollsSideways(element, document.body),
    )
    const crosses = (a: DOMRect, b: DOMRect) =>
      Math.min(a.right, b.right) - Math.max(a.left, b.left) > 1 &&
      Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top) > 1
    for (const [index, a] of texts.entries()) {
      for (const b of texts.slice(index + 1)) {
        if (a.contains(b) || b.contains(a)) continue
        const overlap = [...a.getClientRects()].some((box) =>
          [...b.getClientRects()].some((other) => crosses(box, other)),
        )
        if (overlap) problems.push(`${name(a)} overlaps ${name(b)}`)
      }
    }

    // An open tip stays on the screen.
    for (const popover of document.querySelectorAll('.popover.open')) {
      const bounds = popover.getBoundingClientRect()
      if (bounds.left < 0 || bounds.top < 0 || bounds.right > innerWidth || bounds.bottom > innerHeight)
        problems.push(`${name(popover)} runs off the screen`)
    }

    // iPhones zoom the page into any field whose text is smaller.
    for (const field of document.querySelectorAll(
      'input:not([type=radio], [type=checkbox], [type=range]), select, textarea',
    )) {
      const size = parseFloat(getComputedStyle(field).fontSize)
      if (shown(field) && size < 16)
        problems.push(`#${field.id} is ${size}px; iPhones zoom into fields under 16px`)
    }

    // A dropdown kept narrower than its longest option cuts its text off.
    for (const select of document.querySelectorAll('select')) {
      if (!shown(select)) continue
      const probe = select.cloneNode(true) as HTMLSelectElement
      // As wide as its longest option, whatever the page makes it.
      Object.assign(probe.style, {
        width: 'auto',
        minWidth: '0',
        maxWidth: 'none',
        position: 'absolute',
        visibility: 'hidden',
      })
      select.after(probe)
      const needed = probe.getBoundingClientRect().width
      probe.remove()
      const width = select.getBoundingClientRect().width
      if (width < needed - 1)
        problems.push(
          `${select.getAttribute('aria-label') ?? select.id} cuts its options off: ${Math.round(width)}px of ${Math.round(needed)}px`,
        )
    }

    // Except a group that wraps by design, into two rows of two (`wraps`).
    for (const group of document.querySelectorAll('.choices:not(.wraps)')) {
      const rows = new Set(
        [...group.querySelectorAll('label')].map((label) => label.getBoundingClientRect().top),
      )
      if (shown(group) && rows.size > 1)
        problems.push(`"${group.getAttribute('aria-label')}" choices wrap onto ${rows.size} rows`)
    }
    return problems
  })

/** The rules, in `state`. */
async function checkLayout(page: Page, state: string) {
  // Away, or the hover tip of what was last clicked opens in time for the check.
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), state).toEqual([])
}

const dialog = (page: Page) => page.getByRole('dialog')

// The rules themselves: each must notice the bug it's for, so a rule that
// never fires can't pass for a clean page.
test('the rules notice text drawn over text, text cut off, and a tip off the screen', async ({ page }) => {
  await page.evaluate(() => {
    const card = document.querySelector('.card')!
    const over = document.createElement('p')
    over.className = 'planted'
    over.textContent = 'planted over the heading'
    Object.assign(over.style, { position: 'absolute', top: '0', left: '0', margin: '0' })
    card.querySelector('h3')!.after(over)
    const clipped = document.createElement('div')
    clipped.className = 'planted'
    clipped.textContent = 'planted '.repeat(40)
    Object.assign(clipped.style, { width: '40px', height: '20px', overflow: 'hidden', whiteSpace: 'nowrap' })
    card.append(clipped)
    const tip = document.createElement('div')
    tip.className = 'popover open planted'
    tip.textContent = 'planted tip'
    Object.assign(tip.style, { position: 'fixed', top: '-40px', left: '0', display: 'block' })
    document.body.append(tip)
  })
  const problems = await layoutProblems(page)
  expect(
    problems.find((problem) => problem.includes('planted over') && problem.includes('overlaps')),
  ).toBeDefined()
  expect(problems.find((problem) => problem.includes('cuts its content off'))).toBeDefined()
  expect(problems.find((problem) => problem.includes('runs off the screen'))).toBeDefined()
})

test('at the start', { tag: '@phone' }, async ({ page }) => {
  await checkLayout(page, 'start')
  await fitScreenToPage(page)
  // The numbers and the charts change with the tariffs; the rest is layout.
  await expect(page).toHaveScreenshot('start.png', {
    fullPage: true,
    mask: [
      page.locator('td.amount'),
      page.locator('.stat .value'),
      page.locator('.stat.best .label'),
      page.locator('.stat.best .around'),
      page.locator('.best-bar'),
      page.locator('.chart'),
      page.locator('.date'),
    ],
  })
})

// A chosen choice is bold, and so wider: every one must still fit.
test('with every security and exchange chosen', { tag: '@phone' }, async ({ page }) => {
  for (const group of ['Security', 'Exchange']) {
    const choices = page.getByRole('radiogroup', { name: group }).locator('label')
    for (const choice of await choices.all()) {
      await choice.click()
      await checkLayout(page, `${group}: ${await choice.textContent()}`)
    }
  }
})

test('with the exchange rates open, and a tip open', { tag: '@phone' }, async ({ page }) => {
  await page.getByText('$1 = ₪3.0338').click()
  await checkLayout(page, 'rates')
  await page.getByRole('button', { name: 'What “Buy every” means' }).click()
  expect(await layoutProblems(page), 'tip').toEqual([])
})

test('on the fee breakdown', { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click()
  await checkLayout(page, 'breakdown')
})

test('with more options, and on the chart by deposit', { tag: '@phone' }, async ({ page }) => {
  await page.getByLabel('More options', { exact: true }).check()
  await checkLayout(page, 'more options')
  await page.getByRole('radiogroup', { name: 'Chart' }).getByText('By deposit').click()
  await checkLayout(page, 'by deposit')
  await page.getByRole('button', { name: 'Share', exact: true }).hover()
  expect(await layoutProblems(page), 'share tip').toEqual([])
})

test("with a plan's details open, and its broker's", { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('button', { name: 'About Leumi · Pepper' }).click()
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'plan')
  await dialog(page).getByText('All prices').click()
  await checkLayout(page, 'plan, all prices')
  await dialog(page).getByRole('button', { name: 'Bank Leumi' }).click()
  await checkLayout(page, 'broker')
})

test('with a plan of your own', { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('button', { name: "Change a copy of Leumi · Pepper's fees" }).click()
  await dialog(page).getByRole('button', { name: 'Add plan' }).click()
  await checkLayout(page, 'your plans')
})

test('in the plan editor, in both views, with a checklist open', { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('button', { name: "Change a copy of Leumi · Pepper's fees" }).click()
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'simple')
  await dialog(page).getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
  await checkLayout(page, 'full price list')
  await dialog(page).getByRole('button', { name: 'Anything on USA, Europe ▾' }).click()
  await checkLayout(page, 'checklist')
})

// Second prices sit under the fee they're part of, in details and in the editor.
test('with second prices', { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('button', { name: "About Leumi · Online, 'Leumi 18+'" }).click()
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'details')
  await page.keyboard.press('Escape')

  await page.getByRole('button', { name: "Change a copy of Leumi · Online, 'Leumi 18+''s fees" }).click()
  await checkLayout(page, 'second conversion fee')
  await dialog(page).getByRole('button', { name: 'Cancel' }).click()

  await page.getByRole('radiogroup', { name: 'Security' }).getByText('Index fund', { exact: true }).click()
  await page.getByRole('radiogroup', { name: 'Exchange' }).getByText('Tel Aviv').click()
  await page
    .getByRole('button', { name: "Change a copy of Leumi · Online, monthly standing order's fees" })
    .click()
  await checkLayout(page, 'standing order')
  await dialog(page).getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
  await checkLayout(page, 'standing order, full price list')
})

test('with every plan ticked, on both charts', { tag: '@phone' }, async ({ page }) => {
  for (const broker of ['Altshuler Shaham Trade', 'Bank Leumi', 'Excellence Trade', 'IBI', 'Meitav Trade']) {
    await page.getByRole('checkbox', { name: broker, exact: true }).check()
  }
  await checkLayout(page, 'every plan')
  await page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click()
  await checkLayout(page, 'every plan, breakdown')
})

// The chart is right under the bar with the choices and the pin hint, so
// the bar must keep its height: pinning the first plan swaps the hint for a
// button, and the hint's words change with the view.
test('pinning and switching views leave the chart where it is', { tag: '@phone' }, async ({ page }) => {
  // Every view, the chart by deposit too.
  await page.getByLabel('More options', { exact: true }).check()
  const bar = page.locator('.chart-bar')
  const heights: Record<string, number> = {}
  for (const view of ['Value', 'Lost to fees', 'By deposit', 'Breakdown']) {
    await choice(page, 'Chart', view).click()
    heights[view] = (await bar.boundingBox())!.height
    await page.locator('tbody tr td.rank').first().click()
    await expect(page.getByRole('button', { name: 'Unpin all' })).toBeVisible()
    heights[`${view}, pinned`] = (await bar.boundingBox())!.height
    await page.getByRole('button', { name: 'Unpin all' }).click()
  }
  expect(new Set(Object.values(heights)).size, JSON.stringify(heights)).toBe(1)
})

// Tracks, a handling fee and a price per share plus a percentage, in details
// and in the editor; then fractions and a conversion by standing order.
test('with tracks and a handling fee', { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('checkbox', { name: 'IBI', exact: true }).check()
  await page.getByRole('button', { name: 'About IBI · Full tariff' }).click()
  await dialog(page).getByText('All prices').click()
  await checkLayout(page, 'details')
  await page.keyboard.press('Escape')

  await page.getByRole('button', { name: "Change a copy of IBI · Full tariff's fees" }).click()
  await checkLayout(page, 'editor')
  await dialog(page).getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
  await checkLayout(page, 'editor, full price list')
  await dialog(page).getByRole('button', { name: 'Cancel' }).click()

  await page.getByRole('button', { name: "Change a copy of Interactive · Standard's fees" }).click()
  await dialog(page).getByRole('radiogroup', { name: 'View' }).getByText('Simple').click()
  await checkLayout(page, 'fractions and conversion by standing order')
})

test('with the about page open', { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('button', { name: 'How the numbers are made' }).click()
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'about')
})

// Pictures, on the two extreme screens only (playwright.config.ts): what
// the rules can't see, such as a color or an alignment, in the states with
// the most in them. What comes from the tariffs is masked, so they change
// only when the page itself does; the bars can't be, since the amounts are
// the picture.
test.describe('pictures', { tag: '@pictures' }, () => {
  test("of a plan's details", async ({ page }) => {
    await page.getByRole('button', { name: 'About Leumi · Pepper' }).click()
    await expect(dialog(page)).toBeVisible()
    await dialog(page).getByText('All prices').click()
    await page.mouse.move(0, 0)
    await expect(dialog(page)).toHaveScreenshot('plan-dialog.png', {
      mask: [
        dialog(page).locator('.fees-for dd'),
        dialog(page).locator('.group li'),
        dialog(page).locator('.tariff td:nth-child(2)'),
        dialog(page).locator('.tariff li'),
        dialog(page).locator('.source'),
      ],
    })
  })

  test('of the editor, in both views', async ({ page }) => {
    await page.getByRole('button', { name: "Change a copy of Leumi · Pepper's fees" }).click()
    await expect(dialog(page)).toBeVisible()
    await page.mouse.move(0, 0)
    const masks = () => [
      dialog(page).locator('input:not([type=radio], [type=checkbox])'),
      dialog(page).locator('.was'),
    ]
    await expect(dialog(page)).toHaveScreenshot('editor.png', { mask: masks() })
    await dialog(page).getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
    await page.mouse.move(0, 0)
    await expect(dialog(page)).toHaveScreenshot('editor-full.png', { mask: masks() })
  })

  test('of the fee breakdown', async ({ page }) => {
    await page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click()
    await page.mouse.move(0, 0)
    const card = page.locator('section.card', { has: page.locator('.bars') })
    await fitScreenToPage(page)
    await expect(card).toHaveScreenshot('breakdown.png')
  })
})
