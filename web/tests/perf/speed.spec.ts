import { expect, test as base, type Page } from '@playwright/test'
import { dateText } from '../../src/lib/format'
import { t } from '../../src/lib/text'
import { brokerName, exchangeName, listed, RATES, securityName, situations } from '../core'

// What the page costs to use, measured as a person feels it:
// - Each interaction's time to the next paint, from the browser's Event
//   Timing API (the same measure as Chrome's INP). Anything over about 50 ms
//   is felt; 16 ms is the least it reports, rounded to 8.
// - Smoothness during the animations: the longest gap between frames while
//   rows glide and charts redraw, sampled with requestAnimationFrame. A gap
//   over two frames (34 ms) is a visible stutter.
// - The JavaScript heap after many changes, for leaks.
// The phone project throttles the CPU 4×, about a real Galaxy.

/** The most an interaction may take, in ms, and the heap may grow, in MB. */
const BUDGETS = {
  desktop: {
    load: 2000,
    interaction: 80,
    view: 120,
    dialog: 80,
    tip: 50,
    zoom: 60,
    frameGap: 60,
    heapGrowth: 10,
  },
  phone: {
    load: 4000,
    interaction: 160,
    view: 220,
    dialog: 160,
    tip: 80,
    zoom: 100,
    frameGap: 120,
    heapGrowth: 10,
  },
}

declare global {
  interface Window {
    /** [name, duration] of every event that took 16 ms or more to paint. */
    events: [string, number][]
    /** Gaps between frames while sampling, in ms. */
    frameGaps: number[]
    sampleFrames: (ms: number) => void
  }
}

const test = base.extend<{ throttle: number }>({
  /** How many times slower than this machine the page's CPU is; set per project. */
  throttle: [1, { option: true }],
  page: async ({ page, throttle }, use) => {
    await page.route('https://api.frankfurter.dev/**', (route) =>
      route.fulfill({ json: { date: RATES.date, rates: { ILS: 3.4594, USD: 1.1403 } } }),
    )
    await page.addInitScript(() => {
      window.events = []
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) window.events.push([entry.name, entry.duration])
      }).observe({ type: 'event', durationThreshold: 16 } as PerformanceObserverInit)
      window.frameGaps = []
      window.sampleFrames = (ms) => {
        window.frameGaps = []
        const start = performance.now()
        let last = start
        const frame = (now: number) => {
          window.frameGaps.push(now - last)
          last = now
          if (now - start < ms) requestAnimationFrame(frame)
        }
        requestAnimationFrame(frame)
      }
    })
    const cdp = await page.context().newCDPSession(page)
    await cdp.send('Emulation.setCPUThrottlingRate', { rate: throttle })
    await use(page)
  },
})

/** The worst time to paint among the events `act` causes, in ms. */
async function worst(page: Page, act: () => Promise<void>): Promise<number> {
  await page.evaluate(() => (window.events = []))
  await act()
  // The observer reports an event once its paint is done.
  await page.waitForTimeout(350)
  const events = await page.evaluate(() => window.events)
  return Math.max(0, ...events.map(([, duration]) => duration))
}

/** `worst` three times over; the median, so one hiccup doesn't decide. */
async function typical(page: Page, act: () => Promise<void>): Promise<number> {
  const runs = [await worst(page, act), await worst(page, act), await worst(page, act)]
  return runs.toSorted((a, b) => a - b)[1]
}

/** The longest gap between frames during `act` and the `ms` after it. */
async function longestFrameGap(page: Page, act: () => Promise<void>, ms = 600): Promise<number> {
  await page.evaluate((ms) => window.sampleFrames(ms), ms)
  await act()
  await page.waitForTimeout(ms + 100)
  const gaps = await page.evaluate(() => window.frameGaps)
  // The first sample is the wait for the first frame, not a gap.
  return Math.max(0, ...gaps.slice(1))
}

/** The JavaScript heap in use after collecting garbage, in MB. */
async function heapMb(page: Page): Promise<number> {
  const cdp = await page.context().newCDPSession(page)
  await cdp.send('HeapProfiler.collectGarbage')
  const { usedSize } = await cdp.send('Runtime.getHeapUsage')
  return usedSize / 1e6
}

/** There and back, as one action: measured as the worse of the two, and the
 * page is left as it was, so the action can be repeated and the next
 * measure starts from the same page. */
function thereAndBack(there: () => Promise<void>, back: () => Promise<void>): () => Promise<void> {
  return async () => {
    await there()
    await back()
  }
}

interface Measure {
  label: string
  budget: number
  measure: () => Promise<number>
}

/** Everything measured on `project`, in order: loading first, the guide's
 * interactions, then the workbench's (the switch on), the leak check last. */
function measures(page: Page, project: keyof typeof BUDGETS): Measure[] {
  const budgets = BUDGETS[project]
  const choice = (group: string, name: string) =>
    page.getByRole('radiogroup', { name: group }).getByText(name, { exact: true })
  const chip = (id: string) =>
    page
      .getByRole('group', { name: t.whatForTitle })
      .getByRole('button', { name: situations().find((each) => each.id === id)!.name, exact: true })
  const now = page.locator('#now')
  const monthly = page.locator('#monthly')
  const horizon = page.locator('#horizon')
  const horizonRows = page.locator('section.horizons button.row')
  const horizonRow = (years: number) => horizonRows.filter({ hasText: t.horizonsIn(years) }).first()
  const chartRows = page.locator('section.outcome .row:not(.axis) button.press')
  const options = page.locator('section.answer .decision button.choose')
  const returnField = page.locator('#outcome-return')
  const banks = page.locator('section.how .where .banks button')
  const expert = page.locator('label.expert-switch')
  // The plans are ticked in a list of their own, over the inputs.
  const list = page.getByRole('dialog', { name: t.whatsCompared })
  const openList = async () => {
    await page.getByRole('button', { name: t.addOrRemove }).click()
    await expect(list).toBeVisible()
  }
  const closeList = async () => {
    await page.mouse.move(0, 0)
    await page.keyboard.press('Escape')
    await expect(list).toBeHidden()
  }
  /** The list, open, with Leumi's plans unfolded: not measured. */
  const openLeumi = async () => {
    await openList()
    const line = list.getByRole('button', { name: brokerName('Bank Leumi'), exact: true })
    if ((await line.getAttribute('aria-expanded')) === 'false') await line.click()
  }
  const leumi = list.getByRole('checkbox', { name: brokerName('Bank Leumi'), exact: true })
  // Unticking the broker unticks its usual plan too; ticking that back
  // leaves the table as it was.
  // Three banks have an "Online" plan: the checkbox in Leumi's list.
  const online = list
    .getByRole('list', { name: brokerName('Bank Leumi') })
    .getByRole('checkbox', { name: listed('Leumi · Online').plan.name, exact: true })
  const chart = page.locator('#chart .chart')
  const chartBox = async () => {
    await chart.scrollIntoViewIfNeeded()
    return (await chart.boundingBox())!
  }
  /** Opens Pepper's details from the open list, and closes them. */
  const openDetails = async () => {
    const details = page.getByRole('dialog', { name: listed('Leumi · Pepper').plan.name })
    await list.getByRole('button', { name: t.about(listed('Leumi · Pepper').label) }).click()
    await expect(details).toBeVisible()
    await page.mouse.move(0, 0)
    await page.keyboard.press('Escape')
    await expect(details).toBeHidden()
  }
  /** `measure`, with Leumi's plans open in the list, which closes after. */
  const inTheList = async (measure: () => Promise<number>) => {
    await openLeumi()
    const time = await measure()
    await closeList()
    return time
  }
  const wheelZoom = async () => {
    const box = await chartBox()
    await page.mouse.move(box.x + box.width * 0.7, box.y + box.height / 2)
    for (let i = 0; i < 6; i++) await page.mouse.wheel(0, -200)
  }
  const sliderZoom = async () => {
    // The slider along the bottom of the chart.
    const box = await chartBox()
    const y = box.y + box.height - 8 - 16
    await page.mouse.move(box.x + 24, y)
    await page.mouse.down()
    await page.mouse.move(box.x + 24 + (box.width - 114) * 0.5, y, { steps: 10 })
    await page.mouse.up()
  }
  const resetZoom = async () => {
    // Off any field first, or the R goes into it.
    await page.locator('h1').click()
    await page.keyboard.press('r')
  }
  const churn = async (rounds: number) => {
    for (let round = 0; round < rounds; round++) {
      await monthly.fill(String(1000 + round * 10))
      await choice(t.chart, round % 2 ? t.breakdownView : t.lostView).click()
      await inTheList(async () => {
        await openDetails()
        return 0
      })
    }
  }
  const guideChurn = async (rounds: number) => {
    for (let round = 0; round < rounds; round++) {
      await chip(round % 2 ? 'goal' : 'salary').click()
      await now.fill(String(10_000 + round * 100))
      await options.first().click()
      await chartRows.last().click()
    }
  }

  /** A change that sets the guide to the index arena, where the answer is
   * worked out in a worker (a horizon over 5 years). */
  const longHorizon = async () => {
    await chip('salary').click()
    await expect(horizonRows.first()).toBeVisible()
  }
  const thereAndBackWith = (
    there: () => Promise<unknown>,
    back: () => Promise<unknown>,
  ): (() => Promise<number>) => {
    return () =>
      typical(
        page,
        thereAndBack(
          async () => void (await there()),
          async () => void (await back()),
        ),
      )
  }

  const guide: Measure[] = [
    {
      label: 'load: to the first answer (ms)',
      budget: budgets.load,
      measure: () => page.evaluate(() => performance.now()),
    },
    {
      // A short horizon: the answer is worked out on the page's thread.
      label: 'guide: typing the money now (ms)',
      budget: budgets.interaction,
      measure: () =>
        typical(page, async () => {
          await now.fill('')
          await now.pressSequentially('25000', { delay: 120 })
        }),
    },
    {
      label: 'guide: typing the monthly amount (ms)',
      budget: budgets.interaction,
      measure: () =>
        typical(page, async () => {
          await monthly.fill('')
          await monthly.pressSequentially('1500', { delay: 120 })
        }),
    },
    {
      label: 'guide: switching the situation (ms)',
      budget: budgets.view,
      measure: thereAndBackWith(
        () => chip('salary').click(),
        () => chip('goal').click(),
      ),
    },
    {
      // The index arena: the answer and its horizons come from the worker,
      // so the page only shows the last answer meanwhile.
      label: 'guide: typing a long horizon (ms)',
      budget: budgets.interaction,
      measure: async () => {
        await longHorizon()
        const time = await typical(page, async () => {
          await horizon.fill('')
          await horizon.pressSequentially('25', { delay: 120 })
        })
        await chip('goal').click()
        return time
      },
    },
    {
      // The apartment's way: one option, then the other, and back.
      label: 'guide: choosing an option in a decision (ms)',
      budget: budgets.view,
      measure: thereAndBackWith(
        () => options.nth(1).click(),
        () => options.first().click(),
      ),
    },
    {
      label: 'guide: focusing a row of the chart (ms)',
      budget: budgets.view,
      measure: thereAndBackWith(
        () => chartRows.last().click(),
        () => chartRows.first().click(),
      ),
    },
    {
      label: 'guide: choosing a bank where to buy (ms)',
      budget: budgets.view,
      measure: async () => {
        await options.first().click()
        await expect(banks.first()).toBeVisible()
        return typical(
          page,
          thereAndBack(
            () => banks.nth(1).click(),
            () => banks.first().click(),
          ),
        )
      },
    },
    {
      label: 'guide: typing the yearly return (ms)',
      budget: budgets.interaction,
      measure: async () => {
        await chip('salary').click()
        await expect(returnField).toBeVisible()
        const time = await typical(page, async () => {
          await returnField.fill('')
          await returnField.pressSequentially('7', { delay: 120 })
        })
        await returnField.fill('10')
        await chip('goal').click()
        return time
      },
    },
    {
      label: 'guide: a horizon row, and back (ms)',
      budget: budgets.view,
      measure: async () => {
        await chip('bonus').click()
        await expect(horizonRow(1)).toBeVisible()
        const time = await typical(
          page,
          thereAndBack(
            () => horizonRow(1).click(),
            () => horizonRow(5).click(),
          ),
        )
        await chip('goal').click()
        return time
      },
    },
    {
      label: 'guide: opening the glossary (ms)',
      budget: budgets.dialog,
      measure: () =>
        typical(page, async () => {
          await page.getByRole('button', { name: t.glossary }).click()
          await expect(page.locator('dialog[open]')).toBeVisible()
          await page.mouse.move(0, 0)
          await page.keyboard.press('Escape')
          await expect(page.locator('dialog[open]')).toBeHidden()
        }),
    },
    {
      label: 'guide: the switch on and off (ms)',
      budget: budgets.view,
      measure: thereAndBackWith(
        () => expert.click(),
        () => expert.click(),
      ),
    },
    {
      // The rows and the lines are linked: a hover dispatches a highlight.
      label: 'guide: longest frame gap while the money changes (ms)',
      budget: budgets.frameGap,
      measure: async () => {
        const gap = await longestFrameGap(page, () => now.fill('500000'))
        await now.fill('25000')
        return gap
      },
    },
  ]

  /** The workbench, plans on one exchange: the old page's measures. */
  const workbench: Measure[] = [
    {
      label: 'workbench: typing a deposit (ms)',
      budget: budgets.interaction,
      measure: async () => {
        await expert.click()
        await choice(t.whatToBuy, t.thingOther).click()
        await expect(page.locator('section.within .dates')).toContainText(dateText(RATES.date))
        return typical(page, async () => {
          await monthly.fill('')
          await monthly.pressSequentially('2500', { delay: 120 })
        })
      },
    },
    {
      // Every keystroke asks for a new sweep (the monthly deposit's don't:
      // the sweep varies that one), worked out in a worker off the page's
      // thread, which only sends the request.
      label: 'workbench: typing the one-time deposit (ms)',
      budget: budgets.interaction,
      measure: () =>
        typical(page, async () => {
          await now.fill('')
          await now.pressSequentially('10000', { delay: 120 })
        }),
    },
    {
      label: 'workbench: switching what to buy (ms)',
      budget: budgets.view,
      measure: thereAndBackWith(
        () => choice(t.whatToBuy, t.thingSp500).click(),
        () => choice(t.whatToBuy, t.thingOther).click(),
      ),
    },
    {
      label: 'workbench: switching the security (ms)',
      budget: budgets.interaction,
      measure: thereAndBackWith(
        () => choice(t.security, securityName('Bond')).click(),
        () => choice(t.security, securityName('Etf')).click(),
      ),
    },
    {
      label: 'workbench: switching the exchange (ms)',
      budget: budgets.interaction,
      measure: thereAndBackWith(
        () => choice(t.exchange, exchangeName('Tlv')).click(),
        () => choice(t.exchange, exchangeName('Usa')).click(),
      ),
    },
    {
      label: 'workbench: opening the list of plans (ms)',
      budget: budgets.dialog,
      measure: () => typical(page, thereAndBack(openList, closeList)),
    },
    {
      label: "workbench: ticking a broker's plans (ms)",
      budget: budgets.interaction,
      measure: () =>
        inTheList(() =>
          typical(
            page,
            thereAndBack(
              () => leumi.check(),
              async () => {
                await leumi.uncheck()
                await online.check()
              },
            ),
          ),
        ),
    },
    {
      label: 'workbench: switching the chart view (ms)',
      budget: budgets.view,
      measure: thereAndBackWith(
        () => choice(t.chart, t.breakdownView).click(),
        () => choice(t.chart, t.lostView).click(),
      ),
    },
    {
      // Its sweep, every plan over a range of deposits, is worked out in a
      // worker as the inputs change, so the switch only shows and draws it.
      label: 'workbench: switching to the chart by deposit (ms)',
      budget: budgets.view,
      measure: thereAndBackWith(
        () => choice(t.chart, t.byDepositView).click(),
        () => choice(t.chart, t.lostView).click(),
      ),
    },
    {
      label: "workbench: opening a plan's details (ms)",
      budget: budgets.dialog,
      measure: () => inTheList(() => typical(page, openDetails)),
    },
    {
      label: 'workbench: opening a tip (ms)',
      budget: budgets.tip,
      measure: async () => {
        await page.locator('section.workbench').getByRole('button', { name: t.changeAssumptions }).click()
        return typical(page, async () => {
          await page.getByRole('button', { name: t.whatMeans(t.yearlyReturn) }).click()
          await expect(page.getByRole('tooltip').filter({ hasText: 'S&P 500' })).toBeVisible()
          await page.keyboard.press('Escape')
        })
      },
    },
    {
      // The rows glide to their new order when the ranking changes.
      label: 'workbench: longest frame gap while rows reorder (ms)',
      budget: budgets.frameGap,
      measure: async () => {
        const gap = await longestFrameGap(page, () => now.fill('500000'))
        await now.fill('10000')
        return gap
      },
    },
    {
      // The chart by deposit draws a frame after the switch shows; its sweep
      // was worked out in a worker, so the gap is the drawing.
      label: 'workbench: longest frame gap while the chart by deposit draws (ms)',
      budget: budgets.frameGap,
      measure: async () => {
        const gap = await longestFrameGap(page, () => choice(t.chart, t.byDepositView).click())
        await choice(t.chart, t.lostView).click()
        return gap
      },
    },
  ]
  const zooming: Measure[] =
    project === 'desktop'
      ? [
          {
            label: 'wheel zoom, six notches (ms)',
            budget: budgets.zoom,
            measure: () =>
              typical(page, async () => {
                await wheelZoom()
                await resetZoom()
              }),
          },
          {
            label: 'longest frame gap while zooming (ms)',
            budget: budgets.frameGap,
            measure: async () => {
              const gap = await longestFrameGap(page, wheelZoom)
              await resetZoom()
              return gap
            },
          },
          {
            label: "hovering across the table's rows (ms)",
            budget: budgets.interaction,
            measure: () =>
              typical(page, async () => {
                // The first four rows: there are always at least that many.
                for (let index = 0; index < 4; index++) await page.locator('tbody tr').nth(index).hover()
                await page.mouse.move(0, 0)
              }),
          },
        ]
      : [
          {
            label: 'dragging the zoom slider (ms)',
            budget: budgets.zoom,
            measure: () =>
              typical(page, async () => {
                await sliderZoom()
                await resetZoom()
              }),
          },
        ]
  const leaks: Measure[] = [
    {
      label: 'heap growth over 30 rounds, the workbench (MB)',
      budget: budgets.heapGrowth,
      measure: async () => {
        await churn(5)
        const before = await heapMb(page)
        await churn(30)
        return (await heapMb(page)) - before
      },
    },
    {
      label: 'heap growth over 30 rounds, the guide (MB)',
      budget: budgets.heapGrowth,
      measure: async () => {
        await expert.click()
        await guideChurn(5)
        const before = await heapMb(page)
        await guideChurn(30)
        return (await heapMb(page)) - before
      },
    },
  ]
  return [...guide, ...workbench, ...zooming, ...leaks]
}

test('speed and smoothness', async ({ page }, testInfo) => {
  const project = testInfo.project.name as keyof typeof BUDGETS
  await page.goto('/')
  await expect(page.locator('section.answer #answer-title')).toBeVisible()

  const rows: string[] = []
  for (const { label, budget, measure } of measures(page, project)) {
    const value = await measure()
    rows.push(`| ${label} | ${Math.round(value)} | ${budget} |`)
    expect.soft(value, label).toBeLessThanOrEqual(budget)
  }
  const table = [`| ${project} | measured | budget |`, '| --- | ---: | ---: |', ...rows].join('\n')
  console.log(`\n${table}\n`)
  await testInfo.attach('timings', { body: table, contentType: 'text/markdown' })
})
