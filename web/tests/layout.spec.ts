import { expect, test, type Page } from './fixtures'

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
  await expect(page).toHaveScreenshot(`${state}.png`, { fullPage: true })
}

test('at the start', async ({ page }) => {
  await check(page, 'start')
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

test("with a plan's details open", async ({ page }) => {
  await page.getByRole('button', { name: 'About Pepper' }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  expect(await layoutProblems(page), 'dialog').toEqual([])
  await expect(page.getByRole('dialog')).toHaveScreenshot('plan-dialog.png')
})
