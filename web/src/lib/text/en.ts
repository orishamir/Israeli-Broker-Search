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
  overYears: (years: number, todaysMoney: boolean) =>
    `over ${years} year${years === 1 ? '' : 's'}${todaysMoney ? ', in today’s money' : ''}`,
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

  // The table
  rank: 'Rank',
  plan: 'Plan',
  leftAfterTax: 'Left after tax',
  leftAfterTaxTip:
    'What you end up with, and what the table is ranked by: what selling everything at the end brings in shekels, after the sell fee and converting back, less the tax on the gain.',
  underAHundredth: 'less than 0.01%',
  yearlyCost: 'Fees, as a yearly %',
  yearlyCostTip:
    'What the fees come to as a yearly charge on your holdings, the way a fund states its management fee: paying this share of your holdings every year, and nothing else, would leave you the same before tax. Compare it with a fund’s fee, or with the same plan at another deposit. It doesn’t include what the ETF or index fund itself charges, nor, in a fund, its direct expenses (trading commissions and outside managers’ fees). 100% when nothing is left.',
  spreadBetween: (low: string, lowName: string, high: string, highName: string) =>
    `From ${low} (${lowName}) to ${high} (${highName})`,
  spreadAtMost: (name: string, amount: string) => `${name}: ${amount}`,
  ratesOf: (date: string) => `exchange rates of ${date}`,
  reportAMistake: 'Report a mistake (GitHub)',
  accessibilityStatement: 'Accessibility statement',
  needBiggerFirstDeposit: 'Need a bigger one-time deposit',
  lostToFees: 'Lost to fees',
  lostToFeesTip: (selling: boolean) =>
    `How much less you end up with${selling ? ', after selling and before tax,' : ''} than with no fees and buying every month: the fees, plus the growth they and money waiting for a purchase would have earned. ₪10 a month in fees over 20 years is ₪2,400 paid, but about ₪7,000 lost at 10% a year.`,
  tax: 'Tax',
  taxTip: (inflation: string, surtax: string, threshold: string) =>
    `A quarter of the real gain: what selling brings, less what the holdings cost, the cost raised with prices (${inflation} a year), in shekels and in a fund or a policy. At a broker, a security in dollars or euros is measured against the exchange rate, which stays as it is here, so its whole gain is taxed. A large gain in one year also pays the surtax: ${surtax} of what's above ${threshold} (raised with prices from 2028), as if you had no other income that year; a salary adds to it, and selling over several years can avoid it. A shekel bond that isn't linked to prices pays 15% of its whole gain, which the app doesn't tell apart. What was paid to buy counts as cost; of what was paid to keep the account, only the year of the sale's comes off, so of two plans that leave about the same, the one charging on trades pays less tax. A provident fund for investment taken as a pension from 60 pays none.`,
  noTax: 'none',
  feesPaid: 'Fees paid',
  /** `buying`: the monthly purchases, if there are any. */
  feesPaidTip: (selling: boolean, buying?: { years: number; everyMonths: number; abroad: boolean }) => {
    const all = `Every fee charged: purchases, conversions, keeping the account or a fund's management fee${selling ? ', and selling at the end' : ''}.`
    if (!buying) return all
    const { years, everyMonths, abroad } = buying
    const count = Math.floor((years * 12) / everyMonths)
    return `${all} Buying every ${everyMonths === 1 ? 'month' : `${everyMonths} months`} for ${years} years, that is up to ${count} purchases${abroad ? ` and up to ${count} conversions` : ''}.`
  },
  valueHeld: 'Value held',
  valueHeldTip: 'What the investment is worth at the end, without selling.',
  inTodaysMoneyNote: ' In today’s money: divided by how much prices will have risen by then.',
  feesLink: (label: string, amount: string) => `${label} fees: ${amount}, see what they went to`,
  notOfferedFor: (purchase: string) => `Not offered for ${purchase}`,
  notOffered: 'Not offered',
  overTheCeiling: 'Your deposits are over its yearly ceiling',
  theCeiling: 'The ceiling',
  stillLocked: 'Its money is still locked when your years are up',
  priceUnclear: "Not worked out: the tariff's price is unclear",
  notWorkedOut: 'Not worked out',
  theLock: 'The lock',

  // The charts
  chart: 'Chart',
  valueView: 'Portfolio value',
  valueViewTip: 'What each plan is worth, year by year, before selling.',
  lostView: 'Lost to fees',
  lostViewTip:
    'How much less each plan has than with no fees and buying every month, year by year. Shows where plans overtake each other. The end is after selling, as in the table.',
  byDepositView: 'By deposit',
  byDepositViewTip:
    "Each plan's yearly cost if you deposited more or less than you do: where two lines cross, their ranking by fees flips. Plans with minimum fees cost relatively more for small deposits, plans with a fixed fee per order grow cheaper as deposits grow, and a fund's line ends where a year's deposits pass its ceiling. The dashed line is your deposit. Tax isn't in this chart.",
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
  perMonth: 'a month',
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
  holding: 'Management and custody',
  holdingTip:
    'What you pay just for holding the money. At a bank or an investment house: custody (a share of what you hold), a fixed amount a month, or both. At a fund or a policy: its management fee, a share of the balance and at some a share of each deposit.',
  selling: 'Selling',
  sellingTip: 'Selling everything at the end, and converting back to shekels.',
  clickAFee: 'Click',
  tapAFee: 'Tap',
  aFeeToCompare: 'a fee to compare the plans by it',
  compareBy: 'Compare the plans by',
  barsAria: 'Fees paid by each plan over the whole period, by kind',
  yearByYear: 'Year by year:',
  hoverOrPin: '. To see another plan, hover over it or pin it.',
  tapAnotherBar: "· tap another plan's bar to see it",
  overTimeAria: (label: string) => `${label}'s fees piling up year by year, by kind`,

  // The inputs
  whatYouBuy: 'What you buy',
  whatYouBuyTip:
    'ETFs, index funds, bonds and stocks are all securities, and banks and investment houses charge different fees for each kind. An ETF and an index fund can hold exactly the same companies: they differ in how they are bought, and so in what you pay in fees.',
  security: 'Security',
  tradedOn: 'Traded on',
  tradedOnTip: (security: string, example: string) =>
    `${security} can be bought in Tel Aviv, or abroad in dollars or euros. ${example} Banks and investment houses charge different fees on each exchange, and abroad some also charge for converting your shekels.`,
  tradedOnBonds:
    "Israel's government bonds trade in Tel Aviv in shekels, and US Treasuries in New York in dollars.",
  tradedOnStocks: 'Teva, for example, trades in Tel Aviv in shekels and in New York in dollars.',
  tradedOnFunds:
    'Each exchange has another ETF on the same S&P 500: in Tel Aviv an Israeli one in shekels, in New York a US one such as VOO, and in Europe usually an Irish one. Each has a fee of its own, and loses another tax on its dividends: 25% of each dividend in a US fund, 15% inside an Irish one.',
  tradedOnIndexFunds:
    'The same S&P 500 has an Israeli index fund in shekels, and foreign index funds in dollars, bought through your broker. Each has a fee of its own, and another tax on its dividends.',
  exchange: 'Exchange',
  downloadingRates: "· downloading today's…",
  couldntDownloadRates: "· couldn't download today's",
  couldntDownloadCheck: (error: string) =>
    `Couldn't download today's rates (${error}). Check the rates below.`,
  ratesSource:
    "The European Central Bank's rates, within a fraction of a percent of the Bank of Israel's. You can change them to try other rates.",
  deposits: 'Deposits',
  everyMonth: 'Every month',
  buyEvery: 'Buy every',
  buyEveryTip:
    "Deposits wait as cash until the next purchase. Buying less often means paying fewer minimum fees, but the cash doesn't grow while it waits. Putting in ₪2,000 a month at a ₪5 minimum fee, buying every month costs ₪60 a year in fees; every 3 months costs ₪20, but up to ₪4,000 sits idle for up to two months at a time.",
  intervals: { 1: 'month', 2: '2 months', 3: '3 months', 6: '6 months', 12: 'year' } as Record<
    number,
    string
  >,
  growingBy: 'Monthly deposit grows by',
  growingByTip:
    'How much more you deposit each month than a year earlier, as a salary grows: at 3%, ₪2,000 a month becomes ₪2,060 in the second year and about ₪3,500 in the twentieth. 0 keeps the deposits the same.',
  percentAYear: '% a year',
  expectations: 'Expectations',
  yearlyReturn: 'Yearly return',
  yearlyReturnTip:
    'How much the security grows a year on average, in its own currency, dividends or interest included. The S&P 500 has made about 10% a year since 1926, in dollars, before inflation; that promises nothing, and in a shekel fund on a US index the dollar moves the return too. A bond held until it is due earns its yield to maturity, which depends on the price paid as well as on its interest, say 4%.',
  sharePrice: (stock: boolean): string => (stock ? 'Share price' : 'Unit price'),
  sharePriceTip: (stock: boolean): string =>
    stock
      ? "Today's price of one share. Most brokers sell whole shares only, so a deposit too small for a share waits for the next purchase: at $500 a share, a ₪2,000 deposit buys one share and the rest waits. Here it grows with the whole return, as if the dividends stayed in the share."
      : "Today's price of one unit of the fund or the bond. Most brokers sell whole units only, so a deposit too small for a unit waits for the next purchase: at $500 a unit, a ₪2,000 deposit buys one unit and the rest waits. Here it grows with the whole return, as if dividends and interest stayed in the security.",
  inflation: 'Inflation',
  inflationTip:
    "How much prices rise a year. On a security in shekels, and in a fund or a policy, tax is paid only on the gain beyond it, so there a higher inflation means less tax. Israel's target is 1–3%, and the app starts from its middle.",
  todaysMoney: "Show amounts in today's money",
  todaysMoneyTip:
    "Prices rise, so a shekel in 20 years buys less than one today. Ticked, every amount is shown in today's shekels: divided by how much prices will have risen by then, at the inflation above. The ranking doesn't change, only how the numbers read.",
  atTheEnd: 'At the end',
  atTheEndTip:
    'Whether everything is sold when the years are up, paying a last trade fee and, abroad, a last conversion, or kept. Selling is what comparisons usually assume.',
  sell: 'Sell',
  sellTip:
    'Everything is sold at the end and, abroad, converted back to shekels: one more trade fee and one more conversion. The table ranks by what that leaves after tax.',
  keep: 'Hold',
  keepTip:
    "Nothing is sold: the table ranks by what the holdings are worth, and nothing is paid for selling or as tax. For money you'll draw on slowly, or pass on.",
  whatsCompared: "What's compared",
  whatsComparedTip:
    'The same security, on the same exchange, can be bought through many banks and investment houses. Each has several plans, and each plan its own fees. Beside them are funds and policies, which invest the money for you. "Add or remove" picks what is compared, and there you can add a plan of your own.',
  tickedAtFirst:
    'Ticked at first: at each bank and investment house the plan a new customer gets, and the provident fund for investment.',
  addOrRemove: 'Add or remove',
  comparedOf: (count: number, total: number) => `${count} of ${total} plans`,
  doneComparing: (count: number) => `Done · ${count} compared`,
  countOf: (count: number, total: number) => `${count} of ${total}`,
  banksAndHouses: 'Banks and investment houses',
  banksAndHousesTip:
    'The same security, on the same exchange, can be bought through many banks and investment houses. Each has several plans, each with its own fees: Leumi, for example, has a plan for Leumi Trade, one for its 18+ group and one for the Pepper app. Click a name to see its plans, and tick the ones to compare.',
  fundsAndPolicies: 'Funds and policies',
  fundsAndPoliciesTip:
    "Instead of buying securities yourself at a broker, you can hand the money to a manager who invests it for a share of it: a fund or a policy. It's taxed by its own rules, and the table counts that. A fund's fee is agreed person by person, so each kind is listed by what its savers really pay, such as the average, beside the most it can charge.",
  takingTheMoneyOut: 'Taking the money out',
  takingTheMoneyOutTip: (age: number) =>
    `A provident fund for investment can be taken all at once, like selling at a broker, or from the age of ${age} as a monthly pension, which isn't taxed. A broker, a study fund and a savings policy pay no pension: for them nothing changes.`,
  allAtOnce: 'All at once',
  allAtOnceTip: 'Everything is taken out when the years are up, and the gain is taxed.',
  asAPension: 'As an annuity',
  asAPensionTip: (age: number) =>
    `From ${age}, the money in a provident fund for investment can move to a fund that pays a monthly pension: the gain isn't taxed, and neither is the pension. The table counts what is moved.`,
  yourAge: 'Your age today',
  yourAgeTip: 'A pension opens from an age, so the table needs to know how old you will be at the end.',
  oldEnough: (then: number, from: number) =>
    `You'd be ${then} at the end: old enough for the pension, which opens at ${from}.`,
  tooYoung: (then: number, from: number) =>
    `You'd be ${then} at the end. The pension opens at ${from}, so the fund is taxed as if taken all at once.`,
  usual: 'for a new customer',
  mayCostMoreAt: (broker: string) => `May cost more at ${broker}: see why`,
  about: (name: string) => `About ${name}`,
  changeACopy: (label: string) => `Change a copy of ${label}'s fees`,
  yourPlans: 'Your plans',
  yourPlansTip:
    'Banks and investment houses often give a discount on fees to those who haggle: 0.06% instead of 0.07%, for example, or a lower minimum. Click ✎ beside a plan to copy it and change its fees to what you think you could get, or add a plan that isn\'t listed with "+ New plan".',
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
  couldntLoad: 'The calculator didn’t load. Check your internet connection and reload the page.',
  ratesUnreachable: 'the exchange-rate service could not be reached',
  ratesServiceAnswered: (status: number) => `the rates service answered ${status}`,

  // Names in the other language
  inEnglish: 'In English',
  alsoCalled: (count: number): string => (count === 1 ? 'Also called' : 'Other names'),
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
  account: 'Keeping the account',
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
  saveTheCopy: 'Save the copy',
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
  custody: 'Custody',
  handling: 'Handling fee',
  fieldOf: (fee: string, field: string) => `${fee}: ${field}`,
  lessTradeFees: "Less that month's trade fees",
  markupPercent: 'percent',
  markupPerDollar: 'per dollar',
  countedAs0: 'counted as 0',
  notPublished: 'not published',
  conversionByStandingOrder: (label: string) => `Conversion ${label}`,
  backTo: (original: string) => `Back to ${original}'s fee`,
  was: (original: string, what?: string) => (what ? `${original}: ${what}` : `${original}:`),
  // The two calculators

  // The short term's inputs
  monthsCount: (months: number) =>
    months === 1 ? 'a month' : months === 2 ? '2 months' : `${months} months`,
  boiRate: "Bank of Israel's rate, on average",
  boiRateTip:
    "The rate the Bank of Israel sets, which savings rates across the economy follow. It decides it eight times a year. Type the rate you expect, on average over the months: a money market fund will earn about that, less its fee. A fixed-rate deposit isn't affected by it once opened, so the lower the rate you expect, the better the deposits do against the funds.",
  boiRateNote: (today: string) =>
    `Today it's ${today}. A fixed-rate deposit locks in its rate on the day it's opened; a money market fund changes with the Bank of Israel's rate.`,

  // The places to compare
  caveats: 'Caveats',

  // The short term's results
  forMonths: (months: number) => (months === 1 ? 'for a month' : `for ${months} months`),
  aboveTheRate: (rate: string) => `${rate} above the rate`,
  yoursPart: 'Yours',
  keptPart: 'Kept by the fund or the bank',
  taxPart: 'Tax',
  monthsAxis: 'Months',
  afterMonths: (months: number) => (months === 1 ? 'After a month' : `After ${months} months`),

  // The guide's frame (the sentences themselves are the core's)
  glossary: 'Glossary',
  aboutLink: 'How the numbers are made',
  allOptions: 'All options',
  allOptionsTip:
    "Shows everything that can be changed and compared: what to buy, in which ways, on which exchange and with which assumptions. Turning it on doesn't change the answer, only what is shown.",
  whatForTitle: 'What is the money for?',
  whatForHint: 'Pick a situation like yours, or skip it and answer below',
  whatForTip:
    'Each situation fills the questions below with an example and shows what it teaches. Every number can be changed.',
  iKnowWhatToBuy: 'I know what I want to buy and compare ›',
  iHaveNow: 'I have now',
  iAddMonthly: "And I'll add every month",
  neededIn: "I'll need the money in",
  timeUnit: 'Unit of time',
  monthsUnit: 'months',
  yearsUnit: 'years',
  dontKnowYet: "Don't know yet",
  iAmAge: 'I am',
  retireAtAge: "and I'll retire at",
  answerUpdatesBelow: '↓ The answer updates below',
  whichBank: 'Which bank holds the portfolio?',
  otherBank: 'Another bank',
  answerTitle: 'What fits money like this',
  whatIsIt: 'What is it? ›',
  changeKind: 'Change ›',
  backToRecommendation: 'Back to the recommendation ›',
  alternativesTitle: 'Compared with the other options',
  withinTitle: "Where it's cheapest to buy",
  allPlans: (count: number) => `All ${count} plans ›`,
  stepsTitle: 'How to do it',
  donutTitle: 'Where every shekel goes at the end',
  ladderTitle: 'And if you need the money sooner or later?',
  learnMore: 'More about this ›',
  back: 'Back',
  glossaryHint: "Every term the calculator uses, as it's called in your bank's app and in ads.",
  whatToBuy: 'What do you want to buy?',
  thingSp500: 'The S&P 500 index',
  thingOther: 'Another index or a stock',
  thingBonds: 'Bonds',
  thingShort: 'Short-term money',
  waysTitle: 'In which ways to compare?',
  waysBest: 'The ways, best first',
  // Package L: the learn card, the glossary, the best bar
  cardWhat: 'What it is',
  cardGoodFor: 'What it is good for',
  cardDownside: 'The downside',
  cardCost: 'What it costs',
  cardTax: 'Tax',
  cardWhereMet: 'Where else you meet it',
  cardInTheBank: "In the bank's app:",
  cardInAds: 'In ads:',
  cardVersus: (name: string) => `Versus ${name} ›`,
  allTerms: 'All terms ›',
  cardBack: '‹ Back',
  bestBarToday: "in today's money",
  bestBarJump: 'To the answer',
}

export type Text = typeof en
