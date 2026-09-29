// Every text the web side shows, in Hebrew: the same keys as `en.ts`, whose
// type this is, so a text missing here fails svelte-check.

import type { Text } from './en'

export const he: Text = {
  // The page
  title: 'עמלות המסחר, בריבית דריבית',
  subtitle:
    'כמה העמלות של הבנקים ובתי ההשקעות בישראל עולות לכם לאורך השנים, על קרנות סל, קרנות מחקות, אג״ח ומניות.',
  otherLanguage: 'English',
  switchLanguage: 'החלפה לאנגלית',
  share: 'שיתוף',
  linkCopied: 'הקישור הועתק',
  linkToComparison: 'קישור להשוואה הזו',
  shareTip:
    'מעתיק קישור להשוואה הזו: מה קונים, ההפקדות והציפיות, והמסלולים שסימנתם. המסלולים שלכם שביניהם נוסעים עם הקישור, כך שמי שפותח אותו רואה גם אותם.',

  // The summary
  youDeposit: 'ההפקדות שלכם',
  overYears: (years, todaysMoney) => `לאורך ${years} שנים${todaysMoney ? ', בכסף של היום' : ''}`,
  withNoFees: 'ללא עמלות',
  ifSoldBeforeTax: 'במכירה בסוף, לפני מס',
  heldAtEnd: 'מוחזק בסוף',
  best: (label) => `הזול ביותר: ${label}`,
  lostAndYearly: (lost, yearly) => `${lost} אבדו לעמלות · ${yearly} בשנה`,
  yearlyAndLost: (yearly, lost) => `${yearly} בשנה · ${lost} אבדו לעמלות`,
  cheapestThroughout: (from, to, swept) =>
    swept === 'Monthly'
      ? `הזול ביותר בכל הפקדה חודשית מ-${from} עד ${to}`
      : `הזול ביותר בכל הפקדה חד-פעמית מ-${from} עד ${to}`,
  cheaperBelow: (amount, plan, swept) =>
    swept === 'Monthly'
      ? `בהפקדה של פחות מ-${amount} בחודש, ${plan} זול יותר`
      : `בהפקדה חד-פעמית של פחות מ-${amount}, ${plan} זול יותר`,
  cheaperAbove: (amount, plan, swept) =>
    swept === 'Monthly'
      ? `בהפקדה של יותר מ-${amount} בחודש, ${plan} זול יותר`
      : `בהפקדה חד-פעמית של יותר מ-${amount}, ${plan} זול יותר`,
  cheaperBothWays: (below, belowPlan, above, abovePlan, swept) =>
    swept === 'Monthly'
      ? `בהפקדה של פחות מ-${below} בחודש ${belowPlan} זול יותר; של יותר מ-${above}, ${abovePlan}`
      : `בהפקדה חד-פעמית של פחות מ-${below} ${belowPlan} זול יותר; של יותר מ-${above}, ${abovePlan}`,
  checkInputs: (error) => `בדקו את הנתונים: ${error}.`,
  tickABroker: 'סמנו בנק או בית השקעות כדי להשוות.',

  // The table
  rank: 'דירוג',
  plan: 'מסלול',
  yearlyCost: 'עלות שנתית',
  yearlyCostTip:
    'כמה העמלות שוות כחיוב שנתי על ההחזקות שלכם, כפי שקרן מציגה את דמי הניהול שלה: תשלום של החלק הזה מההחזקות כל שנה, ולא יותר, היה משאיר אתכם עם אותו הדבר. השוו אותה לדמי הניהול של קרן, או לאותו מסלול בהפקדה אחרת. 100% כשלא נשאר דבר.',
  lostToFees: 'אבד לעמלות',
  lostToFeesTip: (selling) =>
    `כמה פחות נשאר לכם${selling ? ', אחרי מכירה,' : ''} לעומת ללא עמלות ובקנייה כל חודש: העמלות, בתוספת הצמיחה שהן וכסף שחיכה לקנייה היו מרוויחים. ₪10 בחודש בעמלות במשך 20 שנה הם ₪2,400 ששולמו, אבל כ-₪7,000 שאבדו ב-10% בשנה.`,
  valueIfSold: 'שווי במכירה',
  valueIfSoldTip:
    'מה תקבלו בשקלים ממכירת הכול בסוף, אחרי עמלת המכירה וההמרה בחזרה. לפני מס. בחו״ל זו עוד עמלת מסחר אחת ועוד המרה אחת.',
  feesPaid: 'עמלות ששולמו',
  feesPaidTip: (selling) =>
    `כל עמלה שנגבתה: קניות, המרות, דמי משמרת, דמי טיפול${selling ? ', ומכירה בסוף' : ''}. ב-20 שנה של קנייה כל חודש אלה 240 קניות, ובחו״ל 240 המרות.`,
  valueHeld: 'שווי ההחזקות',
  valueHeldTip: 'כמה ההשקעה שווה בסוף, בלי למכור.',
  inTodaysMoneyNote: ' בכסף של היום: מחולק בכמה שהמחירים יעלו עד אז.',
  feesLink: (label, amount) => `העמלות של ${label}: ${amount}, לאן הן הלכו`,
  notOfferedFor: (purchase) => `לא מוצע עבור ${purchase}`,
  notOffered: 'לא מוצע',

  // The charts
  chart: 'גרף',
  valueView: 'שווי תיק',
  valueViewTip: 'כמה כל מסלול שווה, שנה אחר שנה, לפני מכירה.',
  lostView: 'אבד לעמלות',
  lostViewTip:
    'כמה פחות יש בכל מסלול לעומת ללא עמלות ובקנייה כל חודש, שנה אחר שנה. מראה היכן מסלולים עוקפים זה את זה. הסוף הוא אחרי מכירה, כמו בטבלה.',
  byDepositView: 'לפי הפקדה',
  byDepositViewTip:
    'העלות השנתית של כל מסלול אילו הפקדתם יותר או פחות ממה שאתם מפקידים: היכן ששני קווים נחתכים, הדירוג ביניהם מתהפך. מסלולים עם עמלות מינימום עולים הרבה בהפקדות קטנות ומעט בגדולות. הקו המקווקו הוא ההפקדה שלכם.',
  breakdownView: 'פירוט',
  breakdownViewTip: 'מה כל מסלול משלם בעמלות, לפי סוג, ואיך הן נערמות לאורך השנים.',
  pinHintMouse: (bars) => `לחצו על שורה או על ${bars ? 'עמודה' : 'קו'} כדי לנעוץ`,
  pinHintTouch: (bars) => `הקישו על שורה או על ${bars ? 'עמודה' : 'קו'} כדי לנעוץ`,
  zoomHintMouse: 'גלגלת: זום בשנים · גרירה: הזזה · R: איפוס',
  zoomHintTouch: 'גררו את קצות המחוון כדי להתקרב',
  unpinAll: 'שחרור הכול',
  years: 'שנים',
  noFees: 'ללא עמלות',
  yourDeposit: 'ההפקדה שלכם',
  you: (deposit) => `אתם: ${deposit}`,
  depositAMonth: 'הפקדה בחודש',
  oneTimeDeposit: 'הפקדה חד-פעמית',
  aMonth: 'בחודש',
  atOnce: 'בבת אחת',
  after: (years, months) => {
    const yearsText = years === 1 ? 'שנה' : years === 2 ? 'שנתיים' : `${years} שנים`
    const monthsText = months === 1 ? 'חודש' : months === 2 ? 'חודשיים' : `${months} חודשים`
    return months === 0 ? `אחרי ${yearsText}` : `אחרי ${yearsText} ו-${monthsText}`
  },

  // The fee breakdown
  purchases: 'קניות',
  purchasesTip: 'עמלת המסחר על כל קנייה.',
  conversions: 'המרות',
  conversionsTip: 'המרת שקלים למטבע של נייר הערך: העמלה והמרווח.',
  custody: 'דמי משמרת',
  custodyTip: 'נגבים על החזקת ניירות הערך.',
  handling: 'דמי טיפול',
  handlingTip: 'העמלה החודשית של החשבון, יהיה בו מה שיהיה.',
  selling: 'מכירה',
  sellingTip: 'מכירת הכול בסוף, וההמרה בחזרה לשקלים.',
  clickAFee: 'לחצו על',
  tapAFee: 'הקישו על',
  aFeeToCompare: 'עמלה כדי להשוות את המסלולים לפיה',
  compareBy: 'השוואת המסלולים לפי',
  barsAria: 'העמלות ששילם כל מסלול לאורך כל התקופה, לפי סוג',
  yearByYear: 'שנה אחר שנה:',
  hoverOrPin: '· רחפו או נעצו מסלול אחר כדי לראות אותו',
  tapAnotherBar: '· הקישו על עמודה של מסלול אחר כדי לראות אותו',
  overTimeAria: (label) => `העמלות של ${label} נערמות שנה אחר שנה, לפי סוג`,

  // The inputs
  tryAnExample: 'נסו דוגמה',
  moreOptions: 'אפשרויות נוספות',
  moreOptionsTip:
    'שלושה נתונים נוספים, לתמונה מדויקת יותר: הפקדות שגדלות כל שנה כמו משכורת, אינפלציה, כדי לראות כל סכום בכסף של היום, והאם הכול נמכר בסוף או נשמר. כשהן כבויות, האפליקציה מניחה הפקדות קבועות, ללא אינפלציה, ומכירה בסוף.',
  whatYouBuy: 'מה קונים',
  security: 'נייר ערך',
  tradedOn: 'נסחר ב',
  exchange: 'בורסה',
  downloadingRates: '· מוריד את שערי היום…',
  couldntDownloadRates: '· לא ניתן היה להוריד את שערי היום',
  couldntDownloadCheck: (error) => `לא ניתן היה להוריד את שערי היום (${error}). בדקו את ברירות המחדל.`,
  deposits: 'הפקדות',
  everyMonth: 'כל חודש',
  buyEvery: 'קנייה כל',
  buyEveryTip:
    'ההפקדות מחכות כמזומן עד הקנייה הבאה. קנייה לעיתים רחוקות יותר פירושה פחות עמלות מינימום, אבל המזומן לא צומח בזמן ההמתנה. בהפקדה של ₪2,000 בחודש עם עמלת מינימום של ₪5, קנייה כל חודש עולה ₪60 בשנה בעמלות; כל 3 חודשים עולה ₪20, אבל עד ₪4,000 שוכבים ללא תשואה עד חודשיים בכל פעם.',
  intervals: { 1: 'חודש', 2: 'חודשיים', 3: '3 חודשים', 6: '6 חודשים', 12: 'שנה' },
  growingBy: 'גדלות ב',
  growingByTip:
    'בכמה יותר אתם מפקידים כל חודש לעומת שנה קודם, כמו משכורת שגדלה: ב-3%, ₪2,000 בחודש הופכים ל-₪2,060 בשנה השנייה ולכ-₪3,500 בשנה העשרים. 0 משאיר את ההפקדות קבועות.',
  percentAYear: '% בשנה',
  expectations: 'ציפיות',
  yearlyReturn: 'תשואה שנתית',
  yearlyReturnTip:
    'בכמה נייר הערך צומח בשנה, במטבע שלו. S&P 500 עשה בממוצע כ-10%; אג״ח צומחת לפי הריבית שלה, נניח 4%.',
  sharePrice: 'מחיר מניה',
  sharePriceTip:
    'מחיר מניה אחת היום. הבנקים ובתי ההשקעות כאן מוכרים מניות שלמות בלבד, ולכן הפקדה קטנה מדי למניה מחכה לקנייה הבאה: ב-$500 למניה, הפקדה של ₪2,000 קונה מניה אחת והשאר מחכה. הוא צומח עם התשואה השנתית.',
  inflation: 'אינפלציה',
  inflationTip:
    'המחירים עולים, ולכן שקל בעוד 20 שנה קונה פחות משקל היום. הזינו את האינפלציה השנתית הצפויה (יעד בנק ישראל הוא 1–3%) וכל סכום יוצג בשקלים של היום: מחולק בכמה שהמחירים יעלו עד אז. הדירוג לא משתנה, רק איך המספרים נקראים. 0 מציג את הסכומים כפי שיהיו.',
  atTheEnd: 'בסוף',
  atTheEndTip:
    'האם הכול נמכר בתום השנים, בתשלום עמלת מסחר אחרונה ובחו״ל המרה אחרונה, או נשמר. מכירה היא ההנחה המקובלת, ומה שרוב העמלות מובילות אליו.',
  sell: 'מכירה',
  sellTip:
    'הכול נמכר בסוף ובחו״ל מומר בחזרה לשקלים: עוד עמלת מסחר אחת ועוד המרה אחת. הטבלה מדרגת לפי מה שנשאר.',
  keep: 'שמירה',
  keepTip: 'דבר לא נמכר: הטבלה מדרגת לפי שווי ההחזקות, ולא משולם דבר על מכירה. לכסף שתמשכו לאט, או תורישו.',
  brokersAndPlans: 'בנקים, בתי השקעות ומסלולים להשוואה',
  brokersAndPlansShort: 'בנקים, בתי השקעות ומסלולים',
  tickedAtFirst: 'מסומן בהתחלה: המסלול הרגיל של כל בנק ובית השקעות, זה שלקוח חדש מקבל.',
  usual: 'רגיל',
  mayCostMoreAt: (broker) => `עשוי לעלות יותר ב${broker}: למה`,
  about: (name) => `על ${name}`,
  changeACopy: (label) => `שינוי עותק של העמלות של ${label}`,
  yourPlans: 'המסלולים שלכם',
  thinkLowerFees:
    'חושבים שתוכלו לקבל עמלות נמוכות יותר, או משתמשים בבנק או בית השקעות שלא ברשימה? ✎ על מסלול משנה עותק שלו.',
  newPlan: '+ מסלול חדש',
  change: (name) => `שינוי ${name}`,
  copyOf: (name, subtitle) => `עותק של ${name} · ${subtitle}`,
  copyOfWord: 'עותק של',
  yourDeal: (subtitle) => `העסקה שלכם · ${subtitle}`,
  yourOwn: 'משלכם',
  brokerYourOwn: (broker) => `${broker} · משלכם`,
  yourPlan: 'המסלול שלכם',
  yourPlanNumbered: (count) => `המסלול שלכם ${count}`,
  ratesServiceAnswered: (status) => `שירות השערים ענה ${status}`,

  // Names in the other language
  inHebrew: 'בעברית',
  inEnglish: 'באנגלית',
  alsoCalled: 'נקרא גם',
  whatMeans: (about) => `מה פירוש ״${about}״`,
  why: 'למה:',
  sources: 'מקורות',

  // A plan's details
  aboutTheNumbers: 'על המספרים',
  close: 'סגירה',
  tariffPdf: 'תעריפון (PDF)',
  forPurchase: (purchase) => `עבור ${purchase}:`,
  changeTheseFees: '✎ שינוי העמלות האלה',
  caveatsFor: (purchase) => `הסתייגויות עבור ${purchase}`,
  allPrices: 'כל המחירים',
  andCaveatsAboutOthers: (count) =>
    count === 1
      ? ', והסתייגות אחת על בחירות או סכומים אחרים'
      : `, ו-${count} הסתייגויות על בחירות או סכומים אחרים`,
  buyingAndSelling: 'קנייה ומכירה',
  fractionsOfAShare: 'שברי מניה',
  soldOn: (exchanges) => `נמכרים ב${exchanges}`,
  tracksYouChoose: (covers) => `שיטות חיוב${covers ? ` ל${covers}` : ''}: בוחרים אחת`,
  buyingByStandingOrder: 'קנייה בהוראת קבע',
  everything: 'הכול',
  none: 'אין',
  handlingFee: 'דמי טיפול',
  theAccount: 'החשבון',
  conversion: 'המרה',
  fee: 'עמלה',
  orIfLess: 'או, אם נמוך יותר',
  byStandingOrder: 'בהוראת קבע',
  markup: 'מרווח',
  caveatsAboutOthers: 'הסתייגויות על בחירות או סכומים אחרים',
  howTheNumbersAreMade: 'איך המספרים מחושבים, ומה לא נכלל ↗',
  plans: 'מסלולים',
  previewHintYours: '✎ משנה את העמלות שלו',
  previewHint: 'ℹ מציג את כל הפרטים וההסתייגויות · ✎ משנה עותק של העמלות שלו',

  // The editor
  simple: 'פשוט',
  simpleTip: 'העמלות על מה שאתם קונים: המחיר והמינימום של כל אחת.',
  fullPriceList: 'תעריפון מלא',
  fullPriceListTip: 'כל שורה בתעריפון, לכל נייר ערך ובורסה, עם מקסימום ותדירות גביית דמי המשמרת.',
  view: 'תצוגה',
  planName: 'שם המסלול',
  broker: 'בנק / בית השקעות',
  optional: 'לא חובה',
  cancel: 'ביטול',
  addPlan: 'הוספת מסלול',
  deleteQuestion: (name) => `למחוק את ״${name}״?`,
  delete: 'מחיקה',
  keepPlan: 'השארה',
  deletePlan: 'מחיקת המסלול',
  done: 'סיום',
  notOfferedHere: 'לא מוצע',
  addAFee: '+ הוספת עמלה',
  sellsFractions: 'מוכר שברי מניה',
  sellsFractionsTip:
    'כל ההפקדה מושקעת, גם כשהיא קטנה ממחיר מניה: ב-$500 למניה, הפקדה של ₪2,000 קונה מניה וקצת במקום מניה אחת והשאר מחכה. מחיר למניה מחשב שבר כמניה שלמה.',
  fractionsTipFull:
    'מניות וקרנות סל בחו״ל נקנות במניות שלמות, אלא אם הברוקר מוכר שברים: אז כל ההפקדה מושקעת, גם כשהיא קטנה ממחיר מניה. ב-$500 למניה, הפקדה של ₪2,000 קונה מניה וקצת במקום מניה אחת והשאר מחכה. מחיר למניה מחשב שבר כמניה שלמה.',
  addRow: '+ שורה',
  removeRow: (covers) => `הסרת השורה של ${covers}`,
  removeCustodyRow: (covers) => `הסרת שורת דמי המשמרת של ${covers}`,
  neverUsed: '⚠ לא בשימוש: שורות ספציפיות יותר מכסות את כולה.',
  nothingCanBeBought: 'אי אפשר לקנות דבר: הוסיפו שורה.',
  noCustodyFee: 'אין דמי משמרת.',
  leastFirstDeposit: 'הפקדה ראשונית מינימלית',
  leastFirstDepositTip:
    'הסכום הנמוך ביותר שאפשר לפתוח איתו חשבון. הטבלה מזהירה כשההפקדה החד-פעמית שלכם נמוכה ממנו.',
  securities: 'ניירות ערך',
  exchanges: 'בורסות',
  noneTickedMeansAll: 'כלום לא מסומן פירושו הכול.',
  whatItCovers: 'על מה זה חל',
  price: 'מחיר',
  plus: 'ועוד',
  perShare: 'למניה',
  min: 'מינימום',
  max: 'מקסימום',
  unit: 'יחידה',
  rate: 'שיעור',
  period: 'תקופה',
  charged: 'נגבה',
  freeFor: 'חינם למשך',
  months: 'חודשים',
  afterOpening: 'מהפתיחה',
  freeMonths: 'חודשים חינם',
  lessTradeFees: 'בקיזוז עמלות המסחר של אותו חודש',
  markupPercent: 'אחוז',
  markupPerDollar: 'לדולר',
  countedAs0: 'נספר כ-0',
  notPublished: 'לא פורסם',
  conversionByStandingOrder: (label) => `המרה ${label}`,
  backTo: (original) => `חזרה לעמלה של ${original}`,
  was: (original, what) => (what ? `${original}: ${what}` : `${original}:`),
}
