// Every text the web side shows, in English. `he.ts` has the same keys in
// Hebrew: its type is this object's, so a text missing there fails
// svelte-check. The core's own texts (fees, caveats, the about page) come
// from the core in the language it was set to; these are the page's frame.
// Texts with a number or a name in them are functions.

import type { Swept } from '../core/core'

export const en = {
  // The page
  title: "What you'll have left",
  share: 'Share',
  linkCopied: 'Link copied',
  linkToComparison: 'Link to this comparison',
  shareTip:
    'Copies a link to this comparison: what you buy, your deposits and expectations, and the plans you ticked. Your own plans among them travel with the link, so whoever opens it sees them too.',

  // The summary
  youDeposit: 'You deposit',
  overYears: (years: number, todaysMoney: boolean) =>
    `over ${years} years${todaysMoney ? ', in today’s money' : ''}`,
  withNoFees: 'With no fees',
  ifSoldBeforeTax: 'if sold at the end, before tax',
  heldAtEnd: 'held at the end',
  best: (label: string) => `Best: ${label}`,
  lostAndYearly: (lost: string, yearly: string) => `${lost} lost to fees · ${yearly} a year`,
  yearlyAndLost: (yearly: string, lost: string) => `${yearly} a year · ${lost} lost to fees`,
  /** Under the best plan's amount: what it is, and the tax it's after. */
  leftAfter: (tax: string | undefined) => (tax ? `left after ${tax} of tax` : 'left, with no tax to pay'),
  // The best plan at other values of the deposit the chart by deposit varies.
  cheapestThroughout: (from: string, to: string, swept: Swept) =>
    swept === 'Monthly'
      ? `The cheapest in fees at any monthly deposit from ${from} to ${to}`
      : `The cheapest in fees for any one-time deposit from ${from} to ${to}`,
  cheaperBelow: (amount: string, plan: string, swept: Swept) =>
    swept === 'Monthly'
      ? `Below ${amount} a month, ${plan} is cheaper`
      : `For a one-time deposit below ${amount}, ${plan} is cheaper`,
  cheaperAbove: (amount: string, plan: string, swept: Swept) =>
    swept === 'Monthly'
      ? `Above ${amount} a month, ${plan} is cheaper`
      : `For a one-time deposit above ${amount}, ${plan} is cheaper`,
  cheaperBothWays: (below: string, belowPlan: string, above: string, abovePlan: string, swept: Swept) =>
    swept === 'Monthly'
      ? `Below ${below} a month, ${belowPlan} is cheaper; above ${above}, ${abovePlan}`
      : `For a one-time deposit below ${below}, ${belowPlan} is cheaper; above ${above}, ${abovePlan}`,
  checkInputs: (error: string) => `Check your inputs: ${error}.`,
  tickABroker: 'Tick at least one plan to compare.',

  // The table
  rank: 'Rank',
  plan: 'Plan',
  leftAfterTax: 'Left after tax',
  leftAfterTaxTip:
    'What you end up with, and what the table is ranked by: what selling everything at the end brings in shekels, after the sell fee and converting back, less the tax on the gain.',
  yearlyCost: 'Yearly fees',
  yearlyCostTip:
    'What the fees come to as a yearly charge on your holdings, the way a fund states its management fee: paying this share of your holdings every year, and nothing else, would leave you the same before tax. Compare it with a fund’s fee, or with the same plan at another deposit. 100% when nothing is left.',
  lostToFees: 'Lost to fees',
  lostToFeesTip: (selling: boolean) =>
    `How much less you end up with${selling ? ', after selling and before tax,' : ''} than with no fees and buying every month: the fees, plus the growth they and money waiting for a purchase would have earned. ₪10 a month in fees over 20 years is ₪2,400 paid, but about ₪7,000 lost at 10% a year.`,
  tax: 'Tax',
  taxTip: (inflation: string) =>
    `A quarter of the real gain: what selling brings, less what the holdings cost, the cost raised with prices (${inflation} a year). What was paid to buy counts as cost; of what was paid to keep the account, only the year of the sale's comes off, so of two plans that leave about the same, the one charging on trades pays less tax. A provident fund for investment taken as a pension from 60 pays none.`,
  noTax: 'none',
  feesPaid: 'Fees paid',
  feesPaidTip: (selling: boolean) =>
    `Every fee charged: purchases, conversions, keeping the account or a fund's management fee${selling ? ', and selling at the end' : ''}. Over 20 years of buying every month, that is 240 purchases and, abroad, 240 conversions.`,
  valueHeld: 'Value held',
  valueHeldTip: 'What the investment is worth at the end, without selling.',
  inTodaysMoneyNote: ' In today’s money: divided by how much prices will have risen by then.',
  feesLink: (label: string, amount: string) => `${label} fees: ${amount}, see what they went to`,
  notOfferedFor: (purchase: string) => `Not offered for ${purchase}`,
  notOffered: 'Not offered',
  overTheCeiling: 'Your deposits are over its yearly ceiling',
  theCeiling: 'The ceiling',
  stillLocked: 'Its money is still locked when your years are up',
  theLock: 'The lock',

  // The charts
  chart: 'Chart',
  valueView: 'Value',
  valueViewTip: 'What each plan is worth, year by year, before selling.',
  lostView: 'Lost to fees',
  lostViewTip:
    'How much less each plan has than with no fees and buying every month, year by year. Shows where plans overtake each other. The end is after selling, as in the table.',
  byDepositView: 'By deposit',
  byDepositViewTip:
    "Each plan's yearly cost if you deposited more or less than you do: where two lines cross, their ranking by fees flips. Plans with minimum fees cost a lot for small deposits and little for large ones, and a fund's line ends where a year's deposits pass its ceiling. The dashed line is your deposit. Tax isn't in this chart.",
  breakdownView: 'Fee breakdown',
  breakdownViewTip:
    'What each plan pays in fees, by kind, and how they pile up over the years. Tax isn’t a fee, and isn’t here.',
  pinHintMouse: (bars: boolean) => `Click a row or a ${bars ? 'bar' : 'line'} to pin it`,
  pinHintTouch: (bars: boolean) => `Tap a row or a ${bars ? 'bar' : 'line'} to pin it`,
  zoomHintMouse: 'Wheel: zoom years · Drag: move · R: reset',
  zoomHintTouch: "Drag the slider's ends to zoom",
  unpinAll: 'Unpin all',
  years: 'Years',
  noFees: 'No fees',
  yourDeposit: 'Your deposit',
  you: (deposit: string) => `You: ${deposit}`,
  depositAMonth: 'Deposit a month',
  oneTimeDeposit: 'One-time deposit',
  aMonth: 'a month',
  aYear: 'a year',
  atOnce: 'at once',
  /** "After 15 years, 5 months": a chart's time, from whole months. */
  after: (years: number, months: number) => {
    const plural = (count: number, unit: string) => `${count} ${unit}${count === 1 ? '' : 's'}`
    const whole = plural(years, 'year')
    return months === 0 ? `After ${whole}` : `After ${whole}, ${plural(months, 'month')}`
  },

  // The fee breakdown
  purchases: 'Purchases',
  purchasesTip: 'The trade fee on every purchase.',
  conversions: 'Conversions',
  conversionsTip: "Converting shekels to the security's currency: the fee and the markup.",
  account: 'Keeping the account',
  accountTip: 'A share of what you hold, a fixed amount a month, or both.',
  management: 'Management fee',
  managementTip:
    'What a fund or a policy takes: a share of the balance every year, and at some, a share of each deposit.',
  selling: 'Selling',
  sellingTip: 'Selling everything at the end, and converting back to shekels.',
  clickAFee: 'Click',
  tapAFee: 'Tap',
  aFeeToCompare: 'a fee to compare the plans by it',
  compareBy: 'Compare the plans by',
  barsAria: 'Fees paid by each plan over the whole period, by kind',
  yearByYear: 'Year by year:',
  hoverOrPin: '· hover or pin another plan to see it',
  tapAnotherBar: "· tap another plan's bar to see it",
  overTimeAria: (label: string) => `${label}'s fees piling up year by year, by kind`,

  // The inputs
  tryAnExample: 'Try an example',
  moreOptions: 'More options',
  moreOptionsTip: (inflation: string) =>
    `More inputs, for a closer picture: deposits that grow every year as a salary does, another inflation, amounts in today's money, and whether everything is sold at the end or kept. Off, the app takes deposits that stay the same, prices rising ${inflation} a year, amounts as they will be, and selling at the end.`,
  whatYouBuy: 'What you buy',
  security: 'Security',
  tradedOn: 'Traded on',
  exchange: 'Exchange',
  downloadingRates: "· downloading today's…",
  couldntDownloadRates: "· couldn't download today's",
  couldntDownloadCheck: (error: string) =>
    `Couldn't download today's rates (${error}). Check the rates below.`,
  deposits: 'Deposits',
  everyMonth: 'Every month',
  buyEvery: 'Buy every',
  buyEveryTip:
    "Deposits wait as cash until the next purchase. Buying less often means paying fewer minimum fees, but the cash doesn't grow while it waits. Putting in ₪2,000 a month at a ₪5 minimum fee, buying every month costs ₪60 a year in fees; every 3 months costs ₪20, but up to ₪4,000 sits idle for up to two months at a time.",
  intervals: { 1: 'month', 2: '2 months', 3: '3 months', 6: '6 months', 12: 'year' } as Record<
    number,
    string
  >,
  growingBy: 'Growing by',
  growingByTip:
    'How much more you deposit each month than a year earlier, as a salary grows: at 3%, ₪2,000 a month becomes ₪2,060 in the second year and about ₪3,500 in the twentieth. 0 keeps the deposits the same.',
  percentAYear: '% a year',
  expectations: 'Expectations',
  yearlyReturn: 'Yearly return',
  yearlyReturnTip:
    'How much the security grows a year, in its own currency. The S&P 500 has averaged about 10%; a bond grows by its interest, say 4%.',
  sharePrice: 'Share price',
  sharePriceTip:
    "Today's price of one share. Most brokers sell whole shares only, so a deposit too small for a share waits for the next purchase: at $500 a share, a ₪2,000 deposit buys one share and the rest waits. It grows with the yearly return.",
  inflation: 'Inflation',
  inflationTip:
    "How much prices rise a year. Tax is paid only on the gain beyond it, so a higher inflation means less tax. Israel's target is 1–3%, and the app starts from its middle.",
  todaysMoney: "Show amounts in today's money",
  todaysMoneyTip:
    "Prices rise, so a shekel in 20 years buys less than one today. Ticked, every amount is shown in today's shekels: divided by how much prices will have risen by then, at the inflation above. The ranking doesn't change, only how the numbers read.",
  atTheEnd: 'At the end',
  atTheEndTip:
    'Whether everything is sold when the years are up, paying a last trade fee and, abroad, a last conversion, or kept. Selling is what comparisons usually assume.',
  sell: 'Sell',
  sellTip:
    'Everything is sold at the end and, abroad, converted back to shekels: one more trade fee and one more conversion. The table ranks by what that leaves after tax.',
  keep: 'Keep',
  keepTip:
    "Nothing is sold: the table ranks by what the holdings are worth, and nothing is paid for selling or as tax. For money you'll draw on slowly, or pass on.",
  whatsCompared: "What's compared",
  tickedAtFirst:
    'Ticked at first: at each bank and investment house the plan a new customer gets, and the provident fund for investment.',
  addOrRemove: 'Add or remove',
  comparedOf: (count: number, total: number) => `${count} of ${total} plans`,
  doneComparing: (count: number) => `Done · ${count} compared`,
  countOf: (count: number, total: number) => `${count} of ${total}`,
  banksAndHouses: 'Banks and investment houses',
  fundsAndPolicies: 'Funds and policies',
  fundsAndPoliciesTip:
    "Instead of buying securities yourself at a broker, you can hand the money to a manager who invests it for a share of it: a fund or a policy. It's taxed by its own rules, and the table counts that. A fund's fee is agreed person by person, so each kind is listed by what its savers really pay, such as the average, beside the most it can charge.",
  takingTheMoneyOut: 'Taking the money out',
  takingTheMoneyOutTip: (age: number) =>
    `A provident fund for investment can be taken all at once, like selling at a broker, or from the age of ${age} as a monthly pension, which isn't taxed. A broker, a study fund and a savings policy pay no pension: for them nothing changes.`,
  allAtOnce: 'All at once',
  allAtOnceTip: 'Everything is taken out when the years are up, and the gain is taxed.',
  asAPension: 'As a pension',
  asAPensionTip: (age: number) =>
    `From ${age}, the money in a provident fund for investment can move to a fund that pays a monthly pension: the gain isn't taxed, and neither is the pension. The table counts what is moved.`,
  yourAge: 'Your age today',
  yourAgeTip: 'A pension opens from an age, so the table needs to know how old you will be at the end.',
  oldEnough: (then: number, from: number) =>
    `You'd be ${then} at the end: old enough for the pension, which opens at ${from}.`,
  tooYoung: (then: number, from: number) =>
    `You'd be ${then} at the end. The pension opens at ${from}, so the fund is taxed as if taken all at once.`,
  usual: 'usual',
  mayCostMoreAt: (broker: string) => `May cost more at ${broker}: see why`,
  about: (name: string) => `About ${name}`,
  changeACopy: (label: string) => `Change a copy of ${label}'s fees`,
  yourPlans: 'Your plans',
  thinkLowerFees:
    "Think you can get lower fees, or use a broker that isn't listed? ✎ on a plan changes a copy of it.",
  newPlan: '+ New plan',
  change: (name: string) => `Change ${name}`,
  copyOf: (name: string, subtitle: string) => `Copy of ${name} · ${subtitle}`,
  copyOfWord: 'Copy of',
  yourDeal: (subtitle: string) => `Your deal · ${subtitle}`,
  yourOwn: 'Your own',
  brokerYourOwn: (broker: string) => `${broker} · your own`,
  yourPlan: 'Your plan',
  yourPlanNumbered: (count: number) => `Your plan ${count}`,
  ratesServiceAnswered: (status: number) => `the rates service answered ${status}`,

  // Names in the other language
  inEnglish: 'In English',
  alsoCalled: 'Also called',
  whatMeans: (about: string) => `What “${about}” means`,
  why: 'Why:',
  sources: 'Sources',

  // A plan's details
  aboutTheNumbers: 'About the numbers',
  close: 'Close',
  tariffPdf: 'Tariff (PDF)',
  forPurchase: (purchase: string) => `For ${purchase}:`,
  changeTheseFees: '✎ Change these fees',
  caveatsFor: (purchase: string) => `Caveats for ${purchase}`,
  allPrices: 'All prices',
  andCaveatsAboutOthers: (count: number) =>
    `, and ${count} caveat${count === 1 ? '' : 's'} about other choices or amounts`,
  buyingAndSelling: 'Buying and selling',
  fractionsOfAShare: 'Fractions of a share',
  soldOn: (exchanges: string) => `sold on ${exchanges}`,
  tracksYouChoose: (covers: string | undefined) => `Tracks${covers ? ` for ${covers}` : ''}: you choose one`,
  buyingByStandingOrder: 'Buying by standing order',
  everything: 'Everything',
  none: 'none',
  ofTheBalance: 'Of the balance',
  ofEachDeposit: 'Of each deposit',
  shareOfHoldings: 'As a share of what you hold',
  fixedAmount: 'As a fixed amount',
  conversion: 'Conversion',
  fee: 'Fee',
  orIfLess: 'Or, if less',
  byStandingOrder: 'By standing order',
  markup: 'Markup',
  caveatsAboutOthers: 'Caveats about other choices or amounts',
  howTheNumbersAreMade: "How the numbers are made, and what isn't counted ↗",
  plans: 'Plans',
  previewHintYours: '✎ changes its fees',
  previewHint: 'ℹ shows the full details and caveats · ✎ changes a copy of its fees',

  // The editor
  simple: 'Simple',
  simpleTip: 'The fees for what you buy: each one’s price and minimum.',
  fullPriceList: 'Full price list',
  fullPriceListTip:
    'Every row of the price list, for every security and exchange, with maximums and how often the share of holdings is charged.',
  view: 'View',
  planName: 'Plan name',
  broker: 'Broker',
  optional: 'optional',
  cancel: 'Cancel',
  addPlan: 'Add plan',
  deleteQuestion: (name: string) => `Delete “${name}”?`,
  delete: 'Delete',
  keepPlan: 'Keep',
  deletePlan: 'Delete plan',
  done: 'Done',
  notOfferedHere: 'not offered',
  addAFee: '+ Add a fee',
  sellsFractions: 'Sells fractions of a share',
  sellsFractionsTip:
    "All of each deposit is invested, even when it's less than a share's price: at $500 a share, a ₪2,000 deposit buys a share and a bit instead of one share with the rest waiting. A price per share counts a fraction as a whole share.",
  fractionsTipFull:
    "Stocks and ETFs abroad are bought in whole shares, unless the broker sells fractions: then all of each deposit is invested, even when it's less than a share's price. At $500 a share, a ₪2,000 deposit buys a share and a bit instead of one share with the rest waiting. A price per share counts a fraction as a whole share.",
  addRow: '+ Row',
  removeRow: (covers: string) => `Remove the row for ${covers}`,
  removeShareRow: (covers: string) => `Remove the share-of-holdings row for ${covers}`,
  neverUsed: '⚠ Never used: more specific rows cover all of it.',
  nothingCanBeBought: 'Nothing can be bought: add a row.',
  noShareOfHoldings: 'Nothing as a share of what you hold.',
  leastFirstDeposit: 'Least first deposit',
  leastFirstDepositTip:
    'The least the account can be opened with. The table warns when your one-time deposit is less.',
  securities: 'Securities',
  exchanges: 'Exchanges',
  noneTickedMeansAll: 'None ticked means all.',
  whatItCovers: 'What it covers',
  price: 'Price',
  plus: 'Plus',
  perShare: 'per share',
  min: 'Min',
  max: 'Max',
  unit: 'unit',
  rate: 'Rate',
  period: 'period',
  charged: 'Charged',
  freeFor: 'Free for',
  months: 'months',
  afterOpening: 'after opening',
  freeMonths: 'free months',
  lessTradeFees: "Less that month's trade fees",
  markupPercent: 'percent',
  markupPerDollar: 'per dollar',
  countedAs0: 'counted as 0',
  notPublished: 'not published',
  conversionByStandingOrder: (label: string) => `Conversion ${label}`,
  backTo: (original: string) => `Back to ${original}'s fee`,
  was: (original: string, what?: string) => (what ? `${original}: ${what}` : `${original}:`),
  // The two calculators
  calculators: 'For how long',
  longTerm: 'Long-term investing',
  longTermKinds: 'ETFs and index funds, bonds, stocks, provident funds',
  shortTerm: 'Short-term saving',
  shortTermKinds: 'Bank deposits, money market funds',
  calculatorsTip:
    "Two calculators, because a fee calculator can only rank what earns the same thing. For the long term, everything earns the return you expect, and differs in fees, tax and how long the money is locked. For the short term, everything earns about the Bank of Israel's rate, and differs in how much of it reaches you: a fund's fee, or a bank paying less than the rate. An ETF and a deposit don't earn the same thing, so they're never in one table.",

  // The short term's inputs
  forHowLong: 'For how long',
  monthsCount: (months: number) =>
    months === 1 ? 'a month' : months === 2 ? '2 months' : `${months} months`,
  boiRate: "Bank of Israel's rate, on average",
  boiRateTip:
    "The rate you expect the Bank of Israel to set, on average over the months. A money market fund earns about this rate, less its fee, and follows it as it changes. A fixed-rate deposit pays its bank's rate for the whole term, whatever the Bank of Israel does, so the rate you expect decides how the funds compare with the deposits.",
  boiRateNote: (today: string) =>
    `Today it's ${today}. A fixed-rate deposit locks in its rate on the day it's opened; a money market fund changes with the Bank of Israel's rate.`,
  shortTickedAtFirst: 'Ticked at first: the averages, the cheapest fund and the five big banks.',
  shortWhatsComparedTip:
    'Money market funds are listed as a kind: the average fee, the cheapest fund and the dearest, since their fees are close. Deposits are listed by bank, at the rates the Bank of Israel published for each. Under “Add or remove” you can also add a deposit at a rate you were offered.',
  shortMoreOptionsTip: (inflation: string) =>
    `One more setting: inflation, which a fund's tax depends on. With the switch off, the calculator takes ${inflation} a year.`,
  shortInflationTip:
    "How much prices rise a year. A fund's tax is on the gain beyond it, so the higher it is, the less tax a fund pays; a deposit's tax is on all its interest, whatever prices do.",

  // The places to compare
  yourDeposits: 'Your deposits',
  yourDepositsTip:
    "A bank may offer you more than it gives on average, especially for a large sum. Add the deposit you were offered, with its yearly rate, and it's compared like the banks': its tax is 15% of the interest, and the money is locked until the end.",
  addYourDeposit: '+ Your deposit',
  depositName: 'Name',
  depositRate: 'Rate',
  deleteDeposit: (name: string) => `Delete ${name}`,
  yourDepositName: 'Your deposit',
  yourDepositNumbered: (count: number) => `Your deposit ${count}`,
  ratesFor: (term: string) => `The rates for a deposit of ${term}`,
  rateOf: (rate: string) => `${rate} interest`,
  feeOf: (fee: string) => `${fee} fee`,
  noRate: 'no rate for these months',
  monthlyNotForDeposits: "A fixed-rate deposit takes one sum: with money every month, it isn't compared.",
  mayLeaveLess: (name: string) => `${name}: may leave you less. Why?`,
  whatItPays: 'What it pays',
  ratesByTerm: 'Its rates, by how long the deposit is',
  yourRateIs: (rate: string) => `The rate you typed: ${rate} a year, for any term.`,
  taxRule: 'Tax',
  caveats: 'Caveats',
  aboutKind: 'About the kind',

  // The short term's results
  forMonths: (months: number) => (months === 1 ? 'for a month' : `for ${months} months`),
  atTheRate: "At the Bank of Israel's rate",
  noCostsNoTax: 'with no costs and no tax',
  netYearly: (rate: string) => `${rate} a year, after costs and tax`,
  canTakeOut: (when: string) => `Can be taken out: ${when.toLowerCase()}`,
  place: 'Place',
  netYearlyColumn: 'A year, net',
  netYearlyTip:
    'What the money earned after costs and tax, as a yearly rate: the rate at which the same deposits would come to the same sum.',
  costYearly: 'Cost a year',
  costYearlyTip:
    "How much of the Bank of Israel's rate the place keeps, as a yearly percentage, like a fund's fee: a fund's fee, or how much less than the rate a bank pays. Below zero, the bank pays more than the rate.",
  shortTaxTip: (inflation: string) =>
    `A deposit pays 15% of all its interest. A fund pays 25% of its gain beyond inflation (${inflation} a year), when it's sold.`,
  whenOut: 'When it can come out',
  whenOutTip:
    "A fund's units can be sold any business day, and the money arrives a day or two later. A deposit locks the money until its term ends: taking it out earlier loses interest.",
  tickAPlace: 'Tick at least one place to compare.',
  splitView: 'Where the interest goes',
  splitViewTip:
    "The interest the Bank of Israel's rate would pay on your money, split three ways: what you keep, what the fund or the bank keeps, and the tax. A bank that pays more than the rate goes past the line.",
  valueOverTime: 'Value over time',
  valueOverTimeTip: 'What the money is worth in each place, month by month, before tax.',
  rateLine: (amount: string) => `The line: the Bank of Israel's rate on your money, ${amount}`,
  yoursPart: 'Yours',
  keptPart: 'Kept by the fund or the bank',
  taxPart: 'Tax',
  splitAria: "Where each place's interest goes: to you, to the place, and to tax",
  placePinHintMouse: (bars: boolean) => `Click a row or a ${bars ? 'bar' : 'line'} to pin a place`,
  placePinHintTouch: (bars: boolean) => `Tap a row or a ${bars ? 'bar' : 'line'} to pin a place`,
  monthsAxis: 'Months',
  afterMonths: (months: number) => (months === 1 ? 'After a month' : `After ${months} months`),
  bestBarNote: (rate: string, when: string) => `${rate} a year, after tax · ${when.toLowerCase()}`,
}

export type Text = typeof en
