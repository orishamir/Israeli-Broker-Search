import { expect, fitScreenToPage, test, type Page } from './fixtures'

// Runs on every device in playwright.config.ts, including the layout-only
// ones: rules that hold at any screen size, then a picture to compare with the
// last approved one (`npx playwright test --update-snapshots` approves).

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
        if (!shown(element) || scrollsSideways(element, box)) continue
        const bounds = element.getBoundingClientRect()
        if (bounds.left < left - 1 || bounds.right > right + 1) {
          problems.push(`${name(element)} sticks out of ${name(box).slice(0, 40)}`)
        }
      }
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

    for (const group of document.querySelectorAll('.choices')) {
      const rows = new Set(
        [...group.querySelectorAll('label')].map((label) => label.getBoundingClientRect().top),
      )
      if (shown(group) && rows.size > 1)
        problems.push(`"${group.getAttribute('aria-label')}" choices wrap onto ${rows.size} rows`)
    }
    return problems
  })

/** The rules, then the picture. */
async function check(page: Page, state: string) {
  expect(await layoutProblems(page), state).toEqual([])
  await fitScreenToPage(page)
  await expect(page).toHaveScreenshot(`${state}.png`, { fullPage: true })
}

test('at the start', async ({ page }) => {
  await check(page, 'start')
})

// A chosen choice is bold, and so wider: every one must still fit.
test('with every security and exchange chosen', async ({ page }) => {
  for (const group of ['Security', 'Exchange']) {
    const choices = page.getByRole('radiogroup', { name: group }).locator('label')
    for (const choice of await choices.all()) {
      await choice.click()
      // Away, or the choice's hover tip opens, sometimes in time for the check.
      await page.mouse.move(0, 0)
      expect(await layoutProblems(page), `${group}: ${await choice.textContent()}`).toEqual([])
    }
  }
})

test('on Tel Aviv', async ({ page }) => {
  await page.getByRole('radiogroup', { name: 'Exchange' }).getByText('Tel Aviv').click()
  // Away, or the choice's hover tip opens, sometimes in time for the picture.
  await page.mouse.move(0, 0)
  await check(page, 'tel-aviv')
})

test('with the exchange rates open', async ({ page }) => {
  await page.getByText('$1 = ₪3.0338').click()
  await check(page, 'rates')
})

test('on the fee breakdown', async ({ page }) => {
  await page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click()
  await page.mouse.move(0, 0)
  await check(page, 'breakdown')
})

test("with a plan's details open", async ({ page }) => {
  await page.getByRole('button', { name: 'About Leumi · Pepper' }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'dialog').toEqual([])
  await expect(page.getByRole('dialog')).toHaveScreenshot('plan-dialog.png')
})

test('with a plan of your own', async ({ page }) => {
  await page.getByRole('button', { name: "Change a copy of Leumi · Pepper's fees" }).click()
  await page.getByRole('dialog').getByRole('button', { name: 'Add plan' }).click()
  await page.mouse.move(0, 0)
  await check(page, 'your-plans')
})

test('in the plan editor', async ({ page }) => {
  await page.getByRole('button', { name: "Change a copy of Leumi · Pepper's fees" }).click()
  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  expect(await layoutProblems(page), 'simple').toEqual([])
  await expect(dialog).toHaveScreenshot('editor.png')

  await dialog.getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'full price list').toEqual([])
  await expect(dialog).toHaveScreenshot('editor-full.png')

  await dialog.getByRole('button', { name: 'Anything on USA, Europe ▾' }).click()
  expect(await layoutProblems(page), 'checklist').toEqual([])
})

// Second prices sit under the fee they're part of, in details and in the editor.
test('with second prices', async ({ page }) => {
  const dialog = page.getByRole('dialog')
  await page.getByRole('button', { name: "About Leumi · Online, 'Leumi 18+'" }).click()
  await expect(dialog).toBeVisible()
  // Away, or a ? now under the pointer opens its tip, which Esc would close
  // instead of the dialog.
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'details').toEqual([])
  await expect(dialog).toHaveScreenshot('second-conversion-details.png')
  await page.keyboard.press('Escape')

  await page.getByRole('button', { name: "Change a copy of Leumi · Online, 'Leumi 18+''s fees" }).click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'second conversion fee').toEqual([])
  await expect(dialog).toHaveScreenshot('editor-second-conversion.png')
  await dialog.getByRole('button', { name: 'Cancel' }).click()

  await page.getByRole('radiogroup', { name: 'Security' }).getByText('Mutual fund', { exact: true }).click()
  await page.getByRole('radiogroup', { name: 'Exchange' }).getByText('Tel Aviv').click()
  await page
    .getByRole('button', { name: "Change a copy of Leumi · Online, monthly standing order's fees" })
    .click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'standing order').toEqual([])
  await expect(dialog).toHaveScreenshot('editor-standing-order.png')

  await dialog.getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'standing order, full price list').toEqual([])
  await expect(dialog).toHaveScreenshot('editor-standing-order-full.png')
})

test('with every plan ticked', async ({ page }) => {
  for (const broker of ['Altshuler Shaham Trade', 'Bank Leumi', 'Excellence Trade', 'IBI', 'Meitav Trade']) {
    await page.getByRole('checkbox', { name: broker, exact: true }).check()
  }
  await page.mouse.move(0, 0)
  await check(page, 'every-plan')
  await page.getByRole('radiogroup', { name: 'Chart' }).getByText('Breakdown').click()
  await page.mouse.move(0, 0)
  await check(page, 'every-plan-breakdown')
})

// Tracks, a handling fee and a price per share plus a percentage, in details
// and in the editor; then fractions and a conversion by standing order.
test('with tracks and a handling fee', async ({ page }) => {
  const dialog = page.getByRole('dialog')
  await page.getByRole('checkbox', { name: 'IBI', exact: true }).check()
  await page.getByRole('button', { name: 'About IBI · Full tariff' }).click()
  await dialog.getByText('All prices').click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'details').toEqual([])
  await expect(dialog).toHaveScreenshot('tracks-details.png')
  await page.keyboard.press('Escape')

  await page.getByRole('button', { name: "Change a copy of IBI · Full tariff's fees" }).click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'editor').toEqual([])
  await expect(dialog).toHaveScreenshot('editor-tracks.png')
  await dialog.getByRole('radiogroup', { name: 'View' }).getByText('Full price list').click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'editor, full price list').toEqual([])
  await dialog.getByRole('button', { name: 'Cancel' }).click()

  await page.getByRole('button', { name: "Change a copy of Interactive · Standard's fees" }).click()
  await dialog.getByRole('radiogroup', { name: 'View' }).getByText('Simple').click()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'fractions and conversion by standing order').toEqual([])
  await expect(dialog).toHaveScreenshot('editor-standing-order-conversion.png')
})

test('with the about page open', async ({ page }) => {
  await page.getByRole('button', { name: 'How the numbers are made' }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.mouse.move(0, 0)
  expect(await layoutProblems(page), 'about').toEqual([])
  await expect(page.getByRole('dialog')).toHaveScreenshot('about-dialog.png')
})
