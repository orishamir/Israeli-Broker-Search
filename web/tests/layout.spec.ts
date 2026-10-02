import {
  brokerName,
  exchangeName,
  exchanges,
  place,
  securities,
  securityName,
  shortTermExamples,
} from './core'
import {
  aboutPlan,
  choice,
  closePlans,
  copyPlan,
  details,
  expect,
  fitScreenToPage,
  planInList,
  test,
  tickBrokers,
  tickPlan,
  type Page,
} from './fixtures'
import { t } from '../src/lib/text'

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

    // Cards stacked in a column keep a gap: "Taking the money out" once sat
    // flush under "What's compared", a dialog between them in the page.
    for (const column of new Set([...document.querySelectorAll('.card')].map((card) => card.parentElement))) {
      const cards = [...(column?.children ?? [])].filter(
        (card) =>
          card.classList.contains('card') &&
          shown(card) &&
          !['fixed', 'sticky'].includes(getComputedStyle(card).position),
      )
      for (const [index, card] of cards.slice(1).entries()) {
        const above = cards[index].getBoundingClientRect()
        const bounds = card.getBoundingClientRect()
        const sideBySide = bounds.top < above.bottom - 1
        if (!sideBySide && bounds.top - above.bottom < 1)
          problems.push(`${name(card).slice(0, 40)} touches ${name(cards[index]).slice(0, 40)}`)
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

const dialog = details

// The rules themselves: each must notice the bug it's for, so a rule that
// never fires can't pass for a clean page.
test('the rules notice text drawn over text, text cut off, a tip off the screen, and cards touching', async ({
  page,
}) => {
  await page.evaluate(() => {
    const card = document.querySelector('.card')!
    const second = document.querySelectorAll<HTMLElement>('aside > .card')[1]
    second.style.marginTop = '0'
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
  expect(problems.find((problem) => problem.includes('touches'))).toBeDefined()
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
  for (const group of [t.security, t.exchange]) {
    const choices = page.getByRole('radiogroup', { name: group }).locator('label')
    for (const choice of await choices.all()) {
      await choice.click()
      await checkLayout(page, `${group}: ${await choice.textContent()}`)
    }
  }
  // An ETF in Tel Aviv, with each of its two funds.
  await page.getByRole('radiogroup', { name: t.security }).locator('label').first().click()
  await page.getByRole('radiogroup', { name: t.exchange }).locator('label').first().click()
  for (const fund of await page.getByRole('radiogroup', { name: t.theFundYouBuy }).locator('label').all()) {
    await fund.click()
    await checkLayout(page, `${t.theFundYouBuy}: ${await fund.textContent()}`)
  }
})

// What's bought, and where, changes the notes under the plans and the
// warning beside the best one's amount: their rows grow taller, nothing moves
// sideways, and the table doesn't move down.
test("the table's columns and the summary's height stay put whatever is bought, and where", async ({
  page,
}) => {
  // Wide enough that no line of the summary wraps; on a narrower screen a
  // longer one may still take two ("with a monthly deposit of more than…").
  await page.setViewportSize({ width: 2000, height: 1000 })
  const measure = () =>
    page.evaluate(() => ({
      summary: document.querySelector('.stats')!.getBoundingClientRect().height,
      columns: [...document.querySelectorAll('.results table th')].map(
        (th) => th.getBoundingClientRect().width,
      ),
    }))
  const first = await measure()
  for (const security of securities()) {
    await choice(page, t.security, security.name).click()
    for (const exchange of exchanges()) {
      await choice(page, t.exchange, exchange.name).click()
      const now = await measure()
      const where = `${security.value} on ${exchange.value}`
      expect(now.summary, where).toBeCloseTo(first.summary, 0)
      expect(now.columns, where).toHaveLength(first.columns.length)
      now.columns.forEach((width, column) =>
        expect(width, `${where}, column ${column}`).toBeCloseTo(first.columns[column], 0),
      )
    }
  }
})

test.describe('while the rows slide', () => {
  test.use({ reducedMotion: 'no-preference' })

  // Rows sliding to their new places reach past the table's bottom for a
  // moment: that showed a vertical scroll bar in its box.
  test('no box but the page scrolls up and down', async ({ page }) => {
    await page.evaluate(() => {
      const scrolled = new Set<string>()
      Object.assign(window, { scrolled })
      const frame = () => {
        for (const box of document.querySelectorAll('body *')) {
          const { overflowY } = getComputedStyle(box)
          if (/auto|scroll/.test(overflowY) && box.scrollHeight > box.clientHeight + 1) {
            scrolled.add(`${box.tagName.toLowerCase()}.${[...box.classList].join('.')}`)
          }
        }
        requestAnimationFrame(frame)
      }
      requestAnimationFrame(frame)
    })
    for (const security of securities()) {
      await choice(page, t.security, security.name).click()
      for (const exchange of exchanges()) {
        await choice(page, t.exchange, exchange.name).click()
        // Each slide to its end before the next change.
        await page.waitForFunction(() => document.getAnimations().length === 0)
      }
    }
    const scrolled = await page.evaluate(() => [...(window as unknown as { scrolled: Set<string> }).scrolled])
    expect(scrolled).toEqual([])
  })
})

test('with the exchange rates open, and a tip open', { tag: '@phone' }, async ({ page }) => {
  await page.getByText('$1 = ₪3.0338').click()
  await checkLayout(page, 'rates')
  await page.getByLabel(t.moreOptions, { exact: true }).check()
  await page.getByRole('button', { name: t.whatMeans(t.buyEvery) }).click()
  expect(await layoutProblems(page), 'tip').toEqual([])
})

test('on the fee breakdown', { tag: '@phone' }, async ({ page }) => {
  await choice(page, t.chart, t.breakdownView).click()
  await checkLayout(page, 'breakdown')
})

test('with more options, and on the chart by deposit', { tag: '@phone' }, async ({ page }) => {
  await page.getByLabel(t.moreOptions, { exact: true }).check()
  await checkLayout(page, 'more options')
  await choice(page, t.chart, t.byDepositView).click()
  await checkLayout(page, 'by deposit')
  await page.getByRole('button', { name: t.share, exact: true }).hover()
  expect(await layoutProblems(page), 'share tip').toEqual([])
})

test("with a plan's details open, and its broker's", { tag: '@phone' }, async ({ page }) => {
  await aboutPlan(page, 'Leumi · Pepper')
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'plan')
  await dialog(page).getByText(t.allPrices).click()
  await checkLayout(page, 'plan, all prices')
  await dialog(page)
    .getByRole('button', { name: brokerName('Bank Leumi') })
    .click()
  await checkLayout(page, 'broker')
})

test('with a plan of your own', { tag: '@phone' }, async ({ page }) => {
  await copyPlan(page, 'Leumi · Pepper')
  await dialog(page).getByRole('button', { name: t.addPlan }).click()
  await checkLayout(page, 'your plans')
})

test('in the plan editor, in both views, with a checklist open', { tag: '@phone' }, async ({ page }) => {
  await copyPlan(page, 'Leumi · Pepper')
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'simple')
  await choice(dialog(page), t.view, t.fullPriceList).click()
  await checkLayout(page, 'full price list')
  await dialog(page)
    .getByRole('button', { name: `הכול ב${exchangeName('Usa')}, ${exchangeName('Europe')} ▾` })
    .click()
  await checkLayout(page, 'checklist')
})

// Second prices sit under the fee they're part of, in details and in the editor.
test('with second prices', { tag: '@phone' }, async ({ page }) => {
  await aboutPlan(page, "Leumi · Online, 'Leumi 18+'")
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'details')
  await page.keyboard.press('Escape')

  await copyPlan(page, "Leumi · Online, 'Leumi 18+'")
  await checkLayout(page, 'second conversion fee')
  await dialog(page).getByRole('button', { name: t.cancel }).click()
  await closePlans(page)

  await choice(page, t.security, securityName('IndexFund')).click()
  await choice(page, t.exchange, exchangeName('Tlv')).click()
  await copyPlan(page, 'Leumi · Online, monthly standing order')
  await checkLayout(page, 'standing order')
  await choice(dialog(page), t.view, t.fullPriceList).click()
  await checkLayout(page, 'standing order, full price list')
})

test('with every plan ticked, on both charts', { tag: '@phone' }, async ({ page }) => {
  await tickBrokers(
    page,
    ['Altshuler Shaham Trade', 'Bank Leumi', 'Excellence Trade', 'IBI', 'Meitav Trade'].map(brokerName),
  )
  await checkLayout(page, 'every plan')
  await choice(page, t.chart, t.breakdownView).click()
  await checkLayout(page, 'every plan, breakdown')
})

// The chart is right under the bar with the choices and the pin hint, so
// the bar must keep its height: pinning the first plan swaps the hint for a
// button, and the hint's words change with the view.
test('pinning and switching views leave the chart where it is', { tag: '@phone' }, async ({ page }) => {
  // Every view, the chart by deposit too.
  await page.getByLabel(t.moreOptions, { exact: true }).check()
  const bar = page.locator('.chart-bar')
  const heights: Record<string, number> = {}
  for (const view of [t.lostView, t.valueView, t.byDepositView, t.breakdownView]) {
    await choice(page, t.chart, view).click()
    heights[view] = (await bar.boundingBox())!.height
    await page.locator('tbody tr td.rank').first().click()
    await expect(page.getByRole('button', { name: t.unpinAll })).toBeVisible()
    heights[`${view}, pinned`] = (await bar.boundingBox())!.height
    await page.getByRole('button', { name: t.unpinAll }).click()
  }
  expect(new Set(Object.values(heights)).size, JSON.stringify(heights)).toBe(1)
})

// Tracks, a handling fee and a price per share plus a percentage, in details
// and in the editor; then fractions and a conversion by standing order.
test('with tracks and a handling fee', { tag: '@phone' }, async ({ page }) => {
  await tickBrokers(page, [brokerName('IBI')])
  await aboutPlan(page, 'IBI · Full tariff')
  await dialog(page).getByText(t.allPrices).click()
  await checkLayout(page, 'details')
  await page.keyboard.press('Escape')

  await copyPlan(page, 'IBI · Full tariff')
  await checkLayout(page, 'editor')
  await choice(dialog(page), t.view, t.fullPriceList).click()
  await checkLayout(page, 'editor, full price list')
  await dialog(page).getByRole('button', { name: t.cancel }).click()

  await copyPlan(page, 'Interactive · Standard')
  await choice(dialog(page), t.view, t.simple).click()
  await checkLayout(page, 'fractions and conversion by standing order')
})

// A fund: how its money is taken out, its details with no price list to
// unfold, its one fee in the editor, and its row when the deposits are over
// its ceiling.
test(
  'with funds: the pension, their details and editor, and a ceiling passed',
  { tag: '@phone' },
  async ({ page }) => {
    await tickPlan(page, 'Savings policy · Average fee')
    await choice(page, t.takingTheMoneyOut, t.asAPension).click()
    await checkLayout(page, 'as a pension')
    await choice(page, t.chart, t.breakdownView).click()
    await checkLayout(page, 'breakdown with funds')

    await aboutPlan(page, 'Provident fund · Average fee')
    await expect(dialog(page)).toBeVisible()
    await checkLayout(page, 'fund')
    await dialog(page)
      .getByRole('button', { name: brokerName('Provident fund for investment') })
      .click()
    await checkLayout(page, 'kind of fund')
    await page.keyboard.press('Escape')

    await copyPlan(page, 'Provident fund · Average fee')
    await checkLayout(page, 'fund editor')
    await dialog(page).getByRole('button', { name: t.addPlan }).click()
    await closePlans(page)
    await page.getByLabel(t.everyMonth).fill('8000')
    await expect(page.getByText(t.overTheCeiling).first()).toBeVisible()
    await checkLayout(page, 'over the ceiling')

    // A study fund, kept for fewer years than it's locked for.
    await tickPlan(page, 'Study fund · Average fee')
    await aboutPlan(page, 'Study fund · Average fee')
    await checkLayout(page, 'study fund')
    await page.keyboard.press('Escape')
    await closePlans(page)
    await page.locator('#years').fill('5')
    await expect(page.getByText(t.stillLocked)).toBeVisible()
    await checkLayout(page, 'locked')
  },
)

test('with the list of plans open, and a bank unfolded', { tag: '@phone' }, async ({ page }) => {
  await planInList(page, 'Leumi · Pepper')
  await checkLayout(page, 'list of plans')
})

test('with the about page open', { tag: '@phone' }, async ({ page }) => {
  await page.getByRole('button', { name: 'איך המספרים מחושבים' }).click()
  await expect(dialog(page)).toBeVisible()
  await checkLayout(page, 'about')
})

test(
  'in the short-term calculator: its list with a deposit of your own, a place, both charts',
  {
    tag: '@phone',
  },
  async ({ page }) => {
    await page.getByText(t.shortTerm, { exact: true }).click()
    await expect(page.getByRole('heading', { name: t.forHowLong })).toBeVisible()
    await checkLayout(page, 'short term')
    await page.getByRole('button', { name: t.addOrRemove }).click()
    await page.getByRole('button', { name: t.addYourDeposit }).click()
    await checkLayout(page, 'list of places')
    await page
      .getByRole('button', { name: t.about(place('Fixed-rate deposit · Bank Leumi').place.name) })
      .click()
    await expect(dialog(page)).toBeVisible()
    await checkLayout(page, "a place's details")
    await page.keyboard.press('Escape')
    await page.keyboard.press('Escape')
    await choice(page, t.chart, t.valueOverTime).click()
    await page.getByRole('button', { name: shortTermExamples()[2].name }).click()
    await checkLayout(page, 'value over time, saving monthly')
  },
)

// Pictures, on the two extreme screens only (playwright.config.ts): what
// the rules can't see, such as a color or an alignment, in the states with
// the most in them. What comes from the tariffs is masked, so they change
// only when the page itself does; the bars can't be, since the amounts are
// the picture.
test.describe('pictures', { tag: '@pictures' }, () => {
  test("of a plan's details", async ({ page }) => {
    await aboutPlan(page, 'Leumi · Pepper')
    await expect(dialog(page)).toBeVisible()
    await dialog(page).getByText(t.allPrices).click()
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
    await copyPlan(page, 'Leumi · Pepper')
    await expect(dialog(page)).toBeVisible()
    await page.mouse.move(0, 0)
    const masks = () => [
      dialog(page).locator('input:not([type=radio], [type=checkbox])'),
      dialog(page).locator('.was'),
    ]
    await expect(dialog(page)).toHaveScreenshot('editor.png', { mask: masks() })
    await choice(dialog(page), t.view, t.fullPriceList).click()
    await page.mouse.move(0, 0)
    await expect(dialog(page)).toHaveScreenshot('editor-full.png', { mask: masks() })
  })

  test('of the fee breakdown', async ({ page }) => {
    await choice(page, t.chart, t.breakdownView).click()
    await page.mouse.move(0, 0)
    const card = page.locator('section.card', { has: page.locator('.bars') })
    await fitScreenToPage(page)
    await expect(card).toHaveScreenshot('breakdown.png')
  })
})
