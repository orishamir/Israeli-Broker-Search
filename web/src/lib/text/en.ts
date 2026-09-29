// Every text the web side shows, in English. `he.ts` has the same keys in
// Hebrew: its type is this object's, so a text missing there fails
// svelte-check. The core's own texts (fees, caveats, the about page) come
// from the core in the language it was set to; these are the page's frame.
// Texts with a number or a name in them are functions.

export const en = {
  // The page
  title: 'Broker fees, compounded',
  subtitle: "What Israeli brokers' fees cost you over the years, for ETFs, index funds, bonds and stocks.",
  /** The other language, to switch to: shown in that language. */
  otherLanguage: 'עברית',
  switchLanguage: 'Switch to Hebrew',
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
  checkInputs: (error: string) => `Check your inputs: ${error}.`,
  tickABroker: 'Tick a broker on the left to compare.',

  // The table
  rank: 'Rank',
  plan: 'Plan',
  yearlyCost: 'Yearly cost',
  yearlyCostTip:
    'What the fees come to as a yearly charge on your holdings, the way a fund states its management fee: paying this share of your holdings every year, and nothing else, would leave you the same. Compare it with a fund’s fee, or with the same plan at another deposit. 100% when nothing is left.',
  lostToFees: 'Lost to fees',
  lostToFeesTip: (selling: boolean) =>
    `How much less you end up with${selling ? ', after selling,' : ''} than with no fees and buying every month: the fees, plus the growth they and money waiting for a purchase would have earned. ₪10 a month in fees over 20 years is ₪2,400 paid, but about ₪7,000 lost at 10% a year.`,
  valueIfSold: 'Value if sold',
  valueIfSoldTip:
    "What you'd get in shekels by selling everything at the end, after the sell fee and converting back. Before tax. Abroad, that's one more trade fee and one more conversion.",
  feesPaid: 'Fees paid',
  feesPaidTip: (selling: boolean) =>
    `Every fee charged: purchases, conversions, custody, handling${selling ? ', and selling at the end' : ''}. Over 20 years of buying every month, that is 240 purchases and, abroad, 240 conversions.`,
  valueHeld: 'Value held',
  valueHeldTip: 'What the investment is worth at the end, without selling.',
  inTodaysMoneyNote: ' In today’s money: divided by how much prices will have risen by then.',
  feesLink: (label: string, amount: string) => `${label} fees: ${amount}, see what they went to`,
  notOfferedFor: (purchase: string) => `Not offered for ${purchase}`,
  notOffered: 'Not offered',

  // The charts
  chart: 'Chart',
  valueView: 'Value',
  valueViewTip: 'What each plan is worth, year by year, before selling.',
  lostView: 'Lost to fees',
  lostViewTip:
    'How much less each plan has than with no fees and buying every month, year by year. Shows where plans overtake each other. The end is after selling, as in the table.',
  byDepositView: 'By deposit',
  byDepositViewTip:
    "Each plan's yearly cost if you deposited more or less than you do: where two lines cross, their ranking flips. Plans with minimum fees cost a lot for small deposits and little for large ones. The dashed line is your deposit.",
  breakdownView: 'Breakdown',
  breakdownViewTip: 'What each plan pays in fees, by kind, and how they pile up over the years.',
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
  custody: 'Custody',
  custodyTip: 'Charged for holding the securities.',
  handling: 'Handling',
  handlingTip: "The account's monthly fee, whatever it holds.",
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
  moreOptionsTip:
    "Three more inputs, for a closer picture: deposits that grow every year as a salary does, inflation, to see every amount in today's money, and whether everything is sold at the end or kept. Off, the app takes deposits that stay the same, no inflation, and selling at the end.",
  whatYouBuy: 'What you buy',
  security: 'Security',
  tradedOn: 'Traded on',
  exchange: 'Exchange',
  downloadingRates: "· downloading today's…",
  couldntDownloadRates: "· couldn't download today's",
  couldntDownloadCheck: (error: string) =>
    `Couldn't download today's rates (${error}). Check these defaults.`,
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
    "Today's price of one share. Brokers here sell whole shares only, so a deposit too small for a share waits for the next purchase: at $500 a share, a ₪2,000 deposit buys one share and the rest waits. It grows with the yearly return.",
  inflation: 'Inflation',
  inflationTip:
    "Prices rise, so a shekel in 20 years buys less than one today. Enter the yearly inflation you expect (Israel's target is 1–3%) and every amount is shown in today's shekels: divided by how much prices will have risen by then. The ranking doesn't change, only how the numbers read. 0 shows the amounts as they will be.",
  atTheEnd: 'At the end',
  atTheEndTip:
    'Whether everything is sold when the years are up, paying a last trade fee and, abroad, a last conversion, or kept. Selling is the usual assumption, and what most of the fees lead up to.',
  sell: 'Sell',
  sellTip:
    'Everything is sold at the end and, abroad, converted back to shekels: one more trade fee and one more conversion. The table ranks by what that leaves.',
  keep: 'Keep',
  keepTip:
    "Nothing is sold: the table ranks by what the holdings are worth, and nothing is paid for selling. For money you'll draw on slowly, or pass on.",
  brokersAndPlans: 'Brokers and plans to compare',
  brokersAndPlansShort: 'Brokers and plans',
  tickedAtFirst: "Ticked at first: each broker's usual plan, the one a new customer gets.",
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
  inHebrew: 'In Hebrew',
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
  handlingFee: 'Handling fee',
  theAccount: 'The account',
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
    'Every row of the price list, for every security and exchange, with maximums and how often custody is charged.',
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
  removeCustodyRow: (covers: string) => `Remove the custody row for ${covers}`,
  neverUsed: '⚠ Never used: more specific rows cover all of it.',
  nothingCanBeBought: 'Nothing can be bought: add a row.',
  noCustodyFee: 'No custody fee.',
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
}

export type Text = typeof en
