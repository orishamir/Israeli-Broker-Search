import type { Page } from '@playwright/test'
import { t } from '../src/lib/text'
import { chip, expect, expertSwitch, fillField, setExpert, situationName, test, workbench } from './fixtures'

// The page's address follows the saver (a refresh restores, Back steps
// through goals and routes), and the yearly return set in the guide's chart
// is the workbench's too.

const hash = (page: Page) => page.evaluate(() => location.hash)

test('the first visit is labelled an example, until anything changes', async ({ page }) => {
  const example = page.getByText(t.exampleTag, { exact: true })
  await expect(example).toBeVisible()
  expect(await hash(page)).toBe('')
  await fillField(page, 'now', '12345')
  await expect(example).toBeHidden()
})

test('a refresh restores the answers, and a restored page is no example', async ({ page }) => {
  await fillField(page, 'monthly', '777')
  await expect.poll(() => hash(page)).toContain('m=777')
  await page.reload()
  await expect(page.locator('#monthly')).toHaveValue('777')
  await expect(page.getByText(t.exampleTag, { exact: true })).toBeHidden()
})

test('Back steps through the goals and the route, and back to the example', async ({ page }) => {
  await chip(page, situationName('retirement')).click()
  await expect(chip(page, situationName('retirement'))).toHaveAttribute('aria-pressed', 'true')
  await setExpert(page)
  await page.goBack()
  await expect(expertSwitch(page)).not.toBeChecked()
  await expect(chip(page, situationName('retirement'))).toHaveAttribute('aria-pressed', 'true')
  await page.goBack()
  await expect(chip(page, situationName('goal'))).toHaveAttribute('aria-pressed', 'true')
  await expect(page.getByText(t.exampleTag, { exact: true })).toBeVisible()
  await page.goForward()
  await expect(chip(page, situationName('retirement'))).toHaveAttribute('aria-pressed', 'true')
})

test(
  'the yearly return set on the chart shows in the workbench, and in the address',
  { tag: '@phone' },
  async ({ page }) => {
    await chip(page, situationName('salary')).click()
    await page.locator('#outcome-return').fill('7')
    await expect.poll(() => hash(page)).toContain('r=7')
    await page.reload()
    await expect(page.locator('#outcome-return')).toHaveValue('7')
    await workbench(page)
    // The index kinds' levers open behind the workbench's "change".
    await page.locator('section.workbench').getByRole('button', { name: t.changeAssumptions }).click()
    await expect(page.locator('#yearly-return')).toHaveValue('7')
  },
)
