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
  await page.getByRole('button', { name: 'About Pepper' }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  expect(await layoutProblems(page), 'dialog').toEqual([])
  await expect(page.getByRole('dialog')).toHaveScreenshot('plan-dialog.png')
})

test('with a plan of your own', async ({ page }) => {
  await page.getByRole('button', { name: "Change a copy of Pepper's fees" }).click()
  await page.getByRole('dialog').getByRole('button', { name: 'Add plan' }).click()
  await page.mouse.move(0, 0)
  await check(page, 'your-plans')
})

test('in the plan editor', async ({ page }) => {
  await page.getByRole('button', { name: "Change a copy of Pepper's fees" }).click()
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
