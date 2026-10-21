// Every text the web side shows, in Hebrew: the same keys as `en.ts`, whose
// type this is, so a text missing here fails svelte-check.

import type { Text } from './en'

// One and two have words of their own: שנה, שנתיים, then 3 שנים.
// "שנים" up to ten, "שנה" above, as Hebrew counts them: 5 שנים, 20 שנה.
const yearsText = (years: number) =>
  years === 1 ? 'שנה' : years === 2 ? 'שנתיים' : years <= 10 ? `${years} שנים` : `${years} שנה`
const monthsText = (months: number) => (months === 1 ? 'חודש' : months === 2 ? 'חודשיים' : `${months} חודשים`)

export const he: Text = {
  // The page
  title: 'כמה יישאר לכם בסוף',
  share: 'שיתוף',
  linkCopied: 'הקישור הועתק',
  linkToComparison: 'קישור להשוואה הזו',
  shareTip:
    'מעתיק קישור להשוואה הזו, עם כל מה שהזנתם: מה קונים, ההפקדות, הציפיות והמסלולים שסימנתם. גם מסלולים שלכם שסימנתם (ששיניתם או שיצרתם) נכללים בקישור, כך שמי שיפתח אותו יראה גם אותם.',

  // The summary
  overYears: (years, todaysMoney) => `לאורך ${yearsText(years)}${todaysMoney ? ', בכסף של היום' : ''}`,
  cheapestThroughout: (from, to, swept) =>
    swept === 'Monthly'
      ? `הזול ביותר בעמלות בכל הפקדה חודשית מ\u2011${from} עד ${to}`
      : `הזול ביותר בעמלות בכל הפקדה חד\u2011פעמית מ\u2011${from} עד ${to}`,
  cheaperBelow: (amount, plan, swept) =>
    swept === 'Monthly'
      ? `בהפקדה חודשית של פחות מ\u2011${amount}, זול יותר: ${plan}`
      : `בהפקדה חד\u2011פעמית של פחות מ\u2011${amount}, זול יותר: ${plan}`,
  cheaperAbove: (amount, plan, swept) =>
    swept === 'Monthly'
      ? `בהפקדה חודשית של יותר מ\u2011${amount}, זול יותר: ${plan}`
      : `בהפקדה חד\u2011פעמית של יותר מ\u2011${amount}, זול יותר: ${plan}`,
  cheaperBothWays: (below, belowPlan, above, abovePlan, swept) =>
    swept === 'Monthly'
      ? `בהפקדה חודשית של פחות מ\u2011${below}, זול יותר: ${belowPlan}; ביותר מ\u2011${above}: ${abovePlan}`
      : `בהפקדה חד\u2011פעמית של פחות מ\u2011${below}, זול יותר: ${belowPlan}; ביותר מ\u2011${above}: ${abovePlan}`,
  checkInputs: (error) => `בדקו את הנתונים: ${error}.`,
  notANumber: 'באחד השדות יש טקסט שאינו מספר',

  // The table
  rank: 'דירוג',
  plan: 'מסלול',
  leftAfterTax: 'נשאר אחרי מס',
  leftAfterTaxTip:
    'הסכום שנשאר לכם בסוף, ולפיו הטבלה מדורגת: כמה שקלים תקבלו כשתמכרו הכול, אחרי עמלת המכירה, ההמרה חזרה לשקלים ומס רווחי הון.',
  underAHundredth: 'פחות מ\u20110.01%',
  yearlyCost: 'עמלות, כאחוז בשנה',
  spreadBetween: (low, lowName, high, highName) => `בין ${low} (${lowName}) ל\u2011${high} (${highName})`,
  spreadAtMost: (name, amount) => `ב${name}: ${amount}`,
  ratesOf: (date) => `שערי החליפין מ\u2011${date}`,
  reportAMistake: 'דיווח על טעות (GitHub)',
  accessibilityStatement: 'הצהרת נגישות',
  needBiggerFirstDeposit: 'דורשים הפקדה חד\u2011פעמית גדולה יותר',
  tax: 'מס',
  feesShort: 'עמלות',
  taxTip: (inflation, surtax, threshold) =>
    `מס רווחי הון: 25% מהרווח. בנייר ערך בשקלים, ובקופה או בפוליסה, את מה ששילמתם בקנייה מתרגמים קודם לערך הכסף ביום המכירה, לפי האינפלציה (${inflation} בשנה), כך שרווח שרק שומר על ערך הכסף לא ממוסה. בחשבון מסחר, בנייר ערך בדולרים או באירו מתרגמים לפי שער החליפין, שבמחשבון נשאר כמו היום, ולכן כל הרווח ממוסה. רווח גדול בשנה אחת מחייב גם מס יסף: ${surtax} על מה שמעבר ל\u2011${threshold} (הסכום עולה עם האינפלציה מ\u20112028); המחשבון מניח שאין לכם הכנסה אחרת באותה שנה, ומשכורת מוסיפה עליו. מכירה על פני כמה שנים יכולה לחסוך אותו. באג״ח שקלית לא צמודה המס הוא 15% מכל הרווח, והמחשבון לא מבחין בה. עמלות הקנייה והמכירה נחשבות חלק מהעלות, ולכן מקטינות את המס; מדמי המשמרת והטיפול מקזזים רק את מה ששולם בשנת המכירה. לכן, בין שני מסלולים שמשאירים בערך אותו סכום, זה שגובה על קנייה ומכירה משלם פחות מס. קופת גמל להשקעה שנמשכת כקצבה מגיל 60 פטורה ממס.`,
  noTax: 'אין',
  feesPaidTip: (selling, buying) => {
    const all = `סך כל העמלות ששילמתם: על קניות, על המרות, דמי משמרת וטיפול או דמי הניהול של קופה${selling ? ', ועל המכירה בסוף' : ''}.`
    if (!buying) return all
    const { years, everyMonths, abroad } = buying
    const count = Math.floor((years * 12) / everyMonths)
    const often = everyMonths === 1 ? 'כל חודש' : `כל ${monthsText(everyMonths)}`
    return `${all} בקנייה ${often} במשך ${yearsText(years)} אלה עד ${count} קניות${abroad ? `, ובחו״ל גם עד ${count} המרות` : ''}.`
  },
  inTodaysMoneyNote: ' בכסף של היום: מתורגם לערך הכסף היום, לפי האינפלציה.',
  feesLink: (label, amount) => `העמלות של ${label}: ${amount}, ועל מה שולמו`,
  notOfferedFor: (purchase) => `לא מוצע ל${purchase}`,
  notOffered: 'לא מוצע',
  overTheCeiling: 'ההפקדות שלכם עוברות את תקרת ההפקדה השנתית',
  theCeiling: 'התקרה',
  stillLocked: 'הכסף עדיין נעול בסוף התקופה',
  theLock: 'הנעילה',
  priceUnclear: 'לא מחושב: המחיר בתעריפון לא ברור',
  notWorkedOut: 'לא מחושב',

  // The charts
  chart: 'גרף',
  valueView: 'שווי התיק',
  valueViewTip: 'שווי התיק בכל מסלול, שנה אחר שנה, לפני מכירה.',
  lostView: 'אבד לעמלות',
  lostViewTip:
    'כמה פחות יש בכל מסלול, שנה אחר שנה, לעומת השקעה בלי עמלות ובקנייה כל חודש. כך רואים מתי מסלול אחד עוקף אחר. הנקודה האחרונה היא אחרי מכירה, כמו בטבלה.',
  byDepositView: 'לפי הפקדה',
  byDepositViewTip:
    'העמלות בשנה של כל מסלול (כאחוז מהתיק) אילו הפקדתם סכום אחר. במקום שבו שני קווים נחתכים, מתחלף הזול מביניהם. מסלול עם עמלת מינימום יקר יחסית בהפקדות קטנות, ומסלול עם עמלה קבועה לכל פקודה נעשה זול יותר ככל שההפקדה גדלה. הקו של קופה נעצר בהפקדה שבה ההפקדות של שנה עוברות את התקרה שלה. הקו המקווקו הוא ההפקדה שלכם. הגרף לא כולל מס.',
  breakdownView: 'פירוט עמלות',
  breakdownViewTip:
    'כמה כל מסלול משלם בעמלות, לפי סוג, ואיך הן מצטברות לאורך השנים. המס לא מוצג כאן, כי הוא לא עמלה.',
  pinHintMouse: (bars) => `לחצו על שורה או על ${bars ? 'עמודה' : 'קו'} כדי לנעוץ מסלול`,
  pinHintTouch: (bars) => `הקישו על שורה או על ${bars ? 'עמודה' : 'קו'} כדי לנעוץ מסלול`,
  zoomHintMouse: 'גלגלת: התקרבות\u00a0· גרירה: הזזה\u00a0· R: איפוס',
  zoomHintTouch: 'גררו את קצות המחוון כדי להתקרב',
  unpinAll: 'שחרור הכול',
  years: 'שנים',
  noFees: 'ללא עמלות',
  yourDeposit: 'ההפקדה שלכם',
  you: (deposit) => `אתם: ${deposit}`,
  depositAMonth: 'הפקדה בחודש',
  oneTimeDeposit: 'הפקדה חד\u2011פעמית',
  aMonth: 'בחודש',
  perMonth: 'לחודש',
  aYear: 'בשנה',
  atOnce: 'בבת אחת',
  after: (years, months) => {
    // A hyphen joins ו to a number only: ו-5 חודשים, but וחודש.
    const and = months > 2 ? 'ו\u2011' : 'ו'
    // Under a year it is months alone: "אחרי 4 חודשים", not "0 שנים ו‑4 חודשים".
    if (years === 0 && months > 0) return `אחרי ${monthsText(months)}`
    return months === 0 ? `אחרי ${yearsText(years)}` : `אחרי ${yearsText(years)} ${and}${monthsText(months)}`
  },

  // The fee breakdown
  purchases: 'קניות',
  purchasesTip: 'עמלת המסחר על כל קנייה.',
  conversions: 'המרות',
  conversionsTip: 'המרת השקלים למטבע של נייר הערך: עמלת ההמרה ומרווח ההמרה.',
  holding: 'דמי ניהול ומשמרת',
  holdingTip:
    'מה שגובים רק על החזקת הכסף. בבנק או בבית השקעות: דמי משמרת (אחוז מהתיק), דמי טיפול (סכום קבוע לחודש) או שניהם. בקופה או בפוליסה: דמי ניהול, אחוז מהצבירה ולפעמים גם מכל הפקדה.',
  selling: 'מכירה',
  sellingTip: 'מכירת הכול בסוף, וההמרה חזרה לשקלים.',
  clickAFee: 'לחצו על',
  tapAFee: 'הקישו על',
  aFeeToCompare: 'עמלה כדי להשוות את המסלולים לפיה',
  compareBy: 'השוואת המסלולים לפי',
  barsAria: 'העמלות ששילם כל מסלול לאורך כל התקופה, לפי סוג',
  yearByYear: 'שנה אחר שנה:',
  hoverOrPin: '. לצפייה במסלול אחר, העבירו עליו את העכבר או נעצו אותו.',
  tapAnotherBar: '\u00a0· הקישו על עמודה של מסלול אחר כדי לראות אותו',
  overTimeAria: (label) => `העמלות של ${label} נערמות שנה אחר שנה, לפי סוג`,

  // The inputs
  whatYouBuy: 'מה קונים',
  whatYouBuyTip:
    'קרנות סל, קרנות מחקות, אג״ח ומניות הן כולן ניירות ערך, והבנקים ובתי ההשקעות גובים על כל סוג עמלות שונות. קרן סל וקרן מחקה יכולות להחזיק בדיוק את אותן חברות: ההבדל הוא איך קונים אותן, ולכן גם כמה משלמים בעמלות.',
  security: 'נייר ערך',
  tradedOn: 'נסחר ב',
  tradedOnTip: (security, example) =>
    `${security} אפשר לקנות בתל אביב, או בחו״ל בדולרים או באירו. ${example} הבנקים ובתי ההשקעות גובים עמלות שונות בכל בורסה, ובחו״ל חלקם גובים גם על המרת השקלים.`,
  tradedOnBonds: 'אג״ח ממשלת ישראל נסחרות בתל אביב בשקלים, ואג״ח ממשלת ארה״ב בניו יורק בדולרים.',
  tradedOnStocks: 'טבע, למשל, נסחרת בתל אביב בשקלים ובניו יורק בדולרים.',
  tradedOnFunds:
    'על אותו מדד S&P\u00a0500 יש קרן סל אחרת בכל בורסה: בתל אביב קרן ישראלית בשקלים, בניו יורק קרן אמריקאית כמו VOO, ובאירופה בדרך כלל קרן אירית. לכל קרן דמי ניהול משלה, ומהדיבידנדים שלה מנוכה מס אחר: בקרן אמריקאית 25% מכל דיבידנד, ובקרן אירית 15% בתוך הקרן.',
  tradedOnIndexFunds:
    'על אותו מדד S&P\u00a0500 יש קרן מחקה ישראלית בשקלים, וקרנות מחקות זרות בדולרים, שקונים דרך הבנק או בית ההשקעות. לכל קרן דמי ניהול משלה, ומס אחר על הדיבידנדים.',
  exchange: 'בורסה',
  downloadingRates: '\u00a0· מוריד את שערי היום…',
  couldntDownloadRates: '\u00a0· לא הצלחנו להוריד את שערי היום',
  couldntDownloadCheck: (error) => `לא הצלחנו להוריד את שערי היום (${error}). בדקו את השערים שלמטה.`,
  ratesSource:
    'שערי הבנק המרכזי האירופי, ששונים מהשער היציג של בנק ישראל בשבריר אחוז לכל היותר. אפשר לשנות אותם כדי לנסות שערים אחרים.',
  deposits: 'הפקדות',
  buyEvery: 'קנייה כל',
  buyEveryTip:
    'ההפקדות מחכות כמזומן עד הקנייה הבאה. קנייה פחות תכופה חוסכת עמלות מינימום, אבל הכסף שמחכה לא מרוויח בינתיים. למשל, בהפקדה של ₪2,000 בחודש ועמלת מינימום של ₪5: קנייה כל חודש עולה ₪60 בשנה, וקנייה כל 3 חודשים עולה ₪20, אבל אז עד ₪4,000 מחכים בלי תשואה, עד חודשיים בכל פעם.',
  intervals: { 1: 'חודש', 2: 'חודשיים', 3: '3 חודשים', 6: '6 חודשים', 12: 'שנה' },
  growingBy: 'ההפקדה החודשית גדלה ב',
  growingByTip:
    'בכמה אחוזים ההפקדה החודשית גדלה כל שנה, כמו משכורת: ב\u20113%, הפקדה של ₪2,000 בחודש גדלה ל\u2011₪2,060 בשנה השנייה ולכ\u2011₪3,500 בשנה העשרים. 0 משאיר את ההפקדות קבועות.',
  percentAYear: '% בשנה',
  expectations: 'ציפיות',
  yearlyReturn: 'תשואה שנתית',
  yearlyReturnTip:
    'כמה נייר הערך מרוויח בשנה בממוצע, במטבע שלו, כולל דיבידנדים או ריבית. מדד S&P 500 הניב מאז 1926 כ\u201110% בשנה בדולרים, לפני אינפלציה; זו לא הבטחה לעתיד, ובקרן בשקלים על מדד S&P\u00a0500 גם שער הדולר משנה את התשואה. אג״ח שמחזיקים עד הפירעון מניבה את התשואה לפדיון שלה, שתלויה גם במחיר הקנייה ולא רק בריבית שכתובה עליה, נניח 4%.',
  sharePrice: (stock) => (stock ? 'מחיר מניה' : 'מחיר יחידה'),
  sharePriceTip: (stock) =>
    stock
      ? 'המחיר של מניה אחת היום. רוב הבנקים ובתי ההשקעות מוכרים רק מניות שלמות, ולכן מה שלא מספיק למניה נוספת מחכה לקנייה הבאה: במחיר של $500 למניה, הפקדה של ₪2,000 קונה מניה אחת, והשאר מחכה. במחשבון המחיר עולה עם כל התשואה, כאילו הדיבידנדים נשארים במניה.'
      : 'המחיר של יחידה אחת של הקרן או של האג״ח היום. רוב הבנקים ובתי ההשקעות מוכרים רק יחידות שלמות, ולכן מה שלא מספיק ליחידה נוספת מחכה לקנייה הבאה: במחיר של $500 ליחידה, הפקדה של ₪2,000 קונה יחידה אחת, והשאר מחכה. במחשבון המחיר עולה עם כל התשואה, כאילו הדיבידנדים והריבית נשארים בתוך נייר הערך.',
  inflation: 'אינפלציה',
  inflationTip:
    'בכמה הכסף מאבד מערכו בשנה, כי מחירי המוצרים והשירותים עולים. מס רווחי הון על נייר ערך בשקלים, ובקופה או בפוליסה, משולם רק על הרווח שמעבר לאינפלציה, ולכן שם אינפלציה גבוהה יותר פירושה פחות מס. יעד האינפלציה של בנק ישראל הוא 1%–3%, והמחשבון מתחיל מאמצע הטווח.',
  todaysMoney: 'הצגת הסכומים בכסף של היום',
  todaysMoneyTip:
    'עם הזמן הכסף מאבד מערכו: שקל בעוד 20 שנה יקנה פחות משקל היום. כשהאפשרות מסומנת, כל סכום מתורגם לערך הכסף היום, לפי האינפלציה שלמעלה. הדירוג לא משתנה, רק אופן ההצגה של הסכומים.',
  atTheEnd: 'בסוף',
  atTheEndTip:
    'האם מוכרים הכול בסוף התקופה (בתשלום עוד עמלת מכירה, ובחו״ל גם המרה חזרה לשקלים), או ממשיכים להחזיק. מכירה היא ההנחה המקובלת בהשוואות.',
  sell: 'מכירה',
  sellTip:
    'הכול נמכר בסוף, ובחו״ל מומר חזרה לשקלים: עוד עמלת מכירה ועוד המרה. הטבלה מדרגת לפי מה שנשאר אחרי מס.',
  keep: 'החזקה',
  keepTip:
    'שום דבר לא נמכר: הטבלה מדרגת לפי שווי התיק בסוף, בלי עמלת מכירה ובלי מס. מתאים לכסף שתמשכו בהדרגה, או שתורישו.',
  whatsCompared: 'מה משווים',
  whatsComparedTip:
    'את אותו נייר ערך, באותה בורסה, אפשר לקנות דרך בנקים ובתי השקעות רבים. לכל אחד יש כמה מסלולים, ולכל מסלול עמלות משלו. לצידם יש קופות ופוליסות, שמשקיעות את הכסף בשבילכם. ב״הוספה והסרה״ בוחרים מה להשוות, ושם אפשר גם להוסיף מסלול משלכם.',
  tickedAtFirst: 'בהתחלה מסומן, בכל בנק ובית השקעות, המסלול שלקוח חדש מקבל, וקופת גמל להשקעה.',
  addOrRemove: 'הוספה והסרה',
  comparedOf: (count, total) => `${count} מתוך ${total} מסלולים`,
  doneComparing: (count) => `סיום\u00a0· ${count} בהשוואה`,
  countOf: (count, total) => `${count} מתוך ${total}`,
  banksAndHouses: 'בנקים ובתי השקעות',
  banksAndHousesTip:
    'את אותו נייר ערך, באותה בורסה, אפשר לקנות דרך בנקים ובתי השקעות רבים. לכל אחד יש כמה מסלולים, ולכל מסלול עמלות משלו: ללאומי, למשל, יש מסלול אונליין, מסלול ״לאומי 18+״ ומסלול פפר. לחצו על שם כדי לראות את המסלולים שלו, וסמנו את אלה שתרצו להשוות.',
  fundsAndPolicies: 'קופות ופוליסות',
  fundsAndPoliciesTip:
    'במקום לקנות ניירות ערך בעצמכם, אפשר להפקיד את הכסף בקופה או בפוליסה: חברה מנהלת משקיעה אותו בשבילכם וגובה דמי ניהול. יש להן כללי מס משלהן, והטבלה מביאה אותם בחשבון. דמי הניהול נקבעים לכל חוסך בנפרד, ולכן כל סוג מוצג לפי מה שהחוסכים בו משלמים בפועל, כמו הממוצע, ולצידו המקסימום.',
  takingTheMoneyOut: 'משיכת הכסף',
  takingTheMoneyOutTip: (age) =>
    `מקופת גמל להשקעה אפשר למשוך את הכסף בבת אחת, כמו מכירה בחשבון מסחר, או מגיל ${age} לקבל אותו כקצבה חודשית פטורה ממס. חשבון מסחר, קרן השתלמות ופוליסת חיסכון לא משלמים קצבה, ולכן הבחירה כאן לא משנה אותם.`,
  allAtOnce: 'בבת אחת',
  allAtOnceTip: 'הכול נמשך בסוף התקופה, ומשלמים מס רווחי הון על הרווח.',
  asAPension: 'כקצבה',
  asAPensionTip: (age) =>
    `מגיל ${age} אפשר להעביר את הכסף מקופת גמל להשקעה לקופה שמשלמת קצבה חודשית. הרווח פטור ממס, וגם הקצבה. הטבלה מציגה את הסכום שמועבר.`,
  yourAge: 'הגיל שלכם היום',
  yourAgeTip: 'קצבה אפשר לקבל רק מגיל מסוים, ולכן המחשבון צריך לדעת בני כמה תהיו בסוף התקופה.',
  oldEnough: (then, from) => `בסוף התקופה תהיו בני ${then}, כך שתוכלו לקבל קצבה (אפשר מגיל ${from}).`,
  tooYoung: (then, from) =>
    `בסוף התקופה תהיו בני ${then}, וקצבה אפשר לקבל רק מגיל ${from}. לכן הקופה ממוסה כאילו נמשכה בבת אחת.`,
  usual: 'ללקוח חדש',
  mayCostMoreAt: (broker) => `ב${broker} זה עשוי לעלות יותר. למה?`,
  about: (name) => `על ${name}`,
  changeACopy: (label) => `שינוי עותק של העמלות של ${label}`,
  yourPlans: 'המסלולים שלכם',
  yourPlansTip:
    'בנקים ובתי השקעות נותנים לא פעם הנחה בעמלות למי שמתמקח: למשל 0.06% במקום 0.07%, או מינימום נמוך יותר. לחצו על ✎ ליד מסלול כדי להעתיק אותו ולשנות את העמלות למה שאתם חושבים שתוכלו לקבל, או הוסיפו מסלול שלא ברשימה עם ״+ מסלול חדש״.',
  thinkLowerFees:
    'חושבים שתוכלו לקבל עמלות נמוכות יותר, או משתמשים בבנק או בבית השקעות שלא ברשימה? לחצו על ✎ ליד מסלול כדי לשנות עותק שלו.',
  newPlan: '+ מסלול חדש',
  change: (name) => `שינוי ${name}`,
  copyOf: (name, subtitle) => `עותק של ${name}\u00a0· ${subtitle}`,
  copyOfWord: 'עותק של',
  yourDeal: (subtitle) => `העסקה שלכם\u00a0· ${subtitle}`,
  yourOwn: 'משלכם',
  brokerYourOwn: (broker) => `${broker}\u00a0· משלכם`,
  yourPlan: 'המסלול שלכם',
  yourPlanNumbered: (count) => `המסלול שלכם ${count}`,
  couldntLoad: 'המחשבון לא נטען. בדקו את החיבור לאינטרנט ורעננו את הדף.',
  ratesUnreachable: 'אין חיבור לשירות שערי החליפין',
  ratesServiceAnswered: (status) => `שירות שערי החליפין החזיר שגיאה ${status}`,

  // Names in the other language
  inEnglish: 'באנגלית',
  alsoCalled: (count) => (count === 1 ? 'שם נוסף' : 'שמות נוספים'),
  whatMeans: (about) => `מה פירוש ״${about}״`,
  why: 'למה:',
  sources: 'מקורות',

  // A plan's details
  aboutTheNumbers: 'על המספרים',
  aboutContents: 'בעמוד הזה',
  backToTop: 'חזרה לראש העמוד',
  close: 'סגירה',
  tariffPdf: 'תעריפון (PDF)',
  forPurchase: (purchase) => `ל${purchase}:`,
  changeTheseFees: '✎ שינוי בעותק',
  caveatsFor: (purchase) => `הסתייגויות ל${purchase}`,
  allPrices: 'כל המחירים',
  andCaveatsAboutOthers: (count) =>
    count === 1
      ? ', והסתייגות אחת על בחירות או סכומים אחרים'
      : `, ו\u2011${count} הסתייגויות על בחירות או סכומים אחרים`,
  buyingAndSelling: 'קנייה ומכירה',
  fractionsOfAShare: 'שברי מניה',
  soldOn: (exchanges) => `נמכרים ב${exchanges}`,
  tracksYouChoose: (covers) => `שיטות חיוב${covers ? ` ל${covers}` : ''}: בוחרים אחת`,
  buyingByStandingOrder: 'קנייה בהוראת קבע',
  everything: 'הכול',
  none: 'אין',
  ofTheBalance: 'מהצבירה',
  ofEachDeposit: 'מכל הפקדה',
  account: 'דמי משמרת וטיפול',
  shareOfHoldings: 'כאחוז מהתיק (דמי משמרת)',
  fixedAmount: 'כסכום קבוע (דמי טיפול)',
  conversion: 'המרה',
  fee: 'עמלה',
  orIfLess: 'או, אם נמוך יותר',
  byStandingOrder: 'בהוראת קבע',
  markup: 'מרווח',
  caveatsAboutOthers: 'הסתייגויות על בחירות או סכומים אחרים',
  howTheNumbersAreMade: 'איך המספרים מחושבים, ומה לא נכלל ↗',
  plans: 'מסלולים',
  previewHintYours: '✎ שינוי העמלות',
  previewHint: 'ℹ כל הפרטים וההסתייגויות\u00a0· ✎ שינוי עותק של העמלות',

  // The editor
  simple: 'פשוט',
  simpleTip: 'העמלות על מה שאתם קונים: המחיר והמינימום של כל אחת.',
  fullPriceList: 'תעריפון מלא',
  fullPriceListTip:
    'כל השורות בתעריפון, לכל נייר ערך ולכל בורסה, כולל מקסימום לכל עמלה וכל כמה זמן נגבים דמי המשמרת.',
  view: 'תצוגה',
  planName: 'שם המסלול',
  broker: 'בנק / בית השקעות',
  optional: 'לא חובה',
  cancel: 'ביטול',
  addPlan: 'הוספת מסלול',
  saveTheCopy: 'שמירת העותק',
  deleteQuestion: (name) => `למחוק את ״${name}״?`,
  delete: 'מחיקה',
  keepPlan: 'השארה',
  deletePlan: 'מחיקת המסלול',
  done: 'סיום',
  notOfferedHere: 'לא מוצע',
  addAFee: '+ הוספת עמלה',
  sellsFractions: 'מוכר שברי מניה',
  sellsFractionsTip:
    'כל ההפקדה מושקעת מיד, גם כשהיא לא מספיקה למניה שלמה: במחיר של $500 למניה, הפקדה של ₪2,000 קונה מניה ועוד חלק ממניה, במקום מניה אחת ויתרה שמחכה. עמלה שנגבית לפי מניה סופרת חלק ממניה כמניה שלמה.',
  fractionsTipFull:
    'מניות וקרנות סל בחו״ל נקנות במניות שלמות, אלא אם הבנק או בית ההשקעות מוכר שברי מניה: אז כל ההפקדה מושקעת מיד, גם כשהיא לא מספיקה למניה שלמה. במחיר של $500 למניה, הפקדה של ₪2,000 קונה מניה ועוד חלק ממניה, במקום מניה אחת ויתרה שמחכה. עמלה שנגבית לפי מניה סופרת חלק ממניה כמניה שלמה.',
  addRow: '+ שורה',
  removeRow: (covers) => `הסרת השורה של ${covers}`,
  removeShareRow: (covers) => `הסרת שורת דמי המשמרת של ${covers}`,
  neverUsed: '⚠ לא בשימוש: שורות מפורטות יותר מכסות את כל מה שהיא חלה עליו.',
  nothingCanBeBought: 'אי אפשר לקנות דבר: הוסיפו שורה.',
  noShareOfHoldings: 'אין דמי משמרת.',
  leastFirstDeposit: 'הפקדה ראשונית מינימלית',
  leastFirstDepositTip:
    'הסכום הנמוך ביותר שאפשר לפתוח בו חשבון. הטבלה מזהירה כשההפקדה החד\u2011פעמית שלכם נמוכה ממנו.',
  securities: 'ניירות ערך',
  exchanges: 'בורסות',
  noneTickedMeansAll: 'אם לא מסומן דבר, השורה חלה על הכול.',
  whatItCovers: 'על מה זה חל',
  price: 'מחיר',
  plus: 'ועוד',
  perShare: 'למניה',
  min: 'מינימום',
  max: 'מקסימום',
  unit: 'יחידה',
  rate: 'שיעור',
  period: 'תקופה',
  charged: 'נגבים',
  freeFor: 'חינם למשך',
  months: 'חודשים',
  afterOpening: 'מהפתיחה',
  freeMonths: 'חודשים חינם',
  // A field's name for screen readers, where its visible label ("מינימום")
  // repeats under every fee: the fee, then the field.
  custody: 'דמי משמרת',
  handling: 'דמי טיפול',
  fieldOf: (fee, field) => `${fee}: ${field}`,
  lessTradeFees: 'בקיזוז עמלות המסחר של אותו חודש',
  markupPercent: 'אחוז',
  markupPerDollar: '₪ לדולר',
  countedAs0: 'נספר כ\u20110',
  notPublished: 'לא פורסם',
  conversionByStandingOrder: (label) => `המרה ${label}`,
  backTo: (original) => `חזרה לעמלה של ${original}`,
  was: (original, what) => (what ? `${original}: ${what}` : `${original}:`),

  // The two calculators

  // The short term's inputs
  boiRate: 'ריבית בנק ישראל, בממוצע',
  boiRateTip:
    'הריבית שבנק ישראל קובע, ושלפיה זזות הריביות על חיסכון בכל המשק. הוא מחליט עליה שמונה פעמים בשנה. כאן מקלידים את הריבית שאתם מצפים לה, בממוצע על פני התקופה: קרן כספית תרוויח בערך אותה, פחות דמי הניהול. פיקדון בריבית קבועה לא מושפע ממנה אחרי שנפתח, ולכן ככל שתצפו לריבית נמוכה יותר, הפיקדונות ייצאו טובים יותר מול הקרנות.',

  // The places to compare
  caveats: 'הסתייגויות',

  // The short term's results
  aboveTheRate: (rate) => `${rate} מעל הריבית`,
  monthsAxis: 'חודשים',
  afterMonths: (months) =>
    months === 1 ? 'אחרי חודש' : months === 2 ? 'אחרי חודשיים' : `אחרי ${months} חודשים`,

  // The guide's frame (the sentences themselves are the core's)
  glossary: 'מילון המונחים',
  aboutLink: 'איך המספרים מחושבים',
  allOptions: 'כל האפשרויות',
  allOptionsTip:
    'מראה את כל מה שאפשר לשנות ולהשוות: מה לקנות, באילו דרכים, באיזו בורסה ובאילו הנחות. התשובה לא משתנה כשמדליקים אותו, רק מה שמוצג.',
  whatForTitle: 'בשביל מה הכסף?',
  whatForHint: 'בחרו מצב שדומה לשלכם, או דלגו וענו למטה',
  whatForTip: 'כל מצב ממלא את השאלות למטה בדוגמה, ומראה מה אפשר ללמוד ממנה. אפשר לשנות כל מספר.',
  iKnowWhatToBuy: 'אני יודע / יודעת מה אני רוצה לקנות ולהשוות ›',
  iHaveNow: 'יש לי עכשיו',
  iAddMonthly: 'ואוסיף כל חודש',
  neededIn: 'אצטרך את הכסף בעוד',
  timeUnit: 'יחידת זמן',
  monthsUnit: 'חודשים',
  yearsUnit: 'שנים',
  iAmAge: 'אני בן / בת',
  retireAtAge: 'ואפרוש בגיל',
  otherBank: 'בנק אחר',
  myBank: 'הבנק שלי',
  noBankChosen: '—',
  withinTitle: 'איפה הכי זול לקנות אותה',
  // The workbench's charts and its funds
  afterSaleAndTax: 'אחרי מכירה ומס',
  whichFund: 'איזו קרן מחזיקה את זה?',
  whichFundTip:
    'את אותו מדד S&P\u00a0500 אפשר לקנות בתל אביב דרך קרן סל ישראלית או דרך קרן זרה שנסחרת גם שם. העלות השנתית שלהן שונה, והמחשבון מחשב כל אחת לחוד.',
  learnMore: 'עוד על זה ›',
  back: 'חזרה',
  glossaryHint: 'כל המונחים שהמחשבון משתמש בהם, כמו שהם נקראים באפליקציה של הבנק ובפרסומות.',
  whatToBuy: 'מה אתם רוצים לקנות?',
  thingSp500: 'מדד S&P\u00a0500',
  thingOther: 'מדד אחר או מניה',
  thingBonds: 'אג״ח',
  thingShort: 'כסף לטווח קצר',
  waysTitle: 'באילו דרכים להשוות?',
  // The learn card and the glossary
  cardWhat: 'מה זה',
  cardGoodFor: 'למה זה טוב',
  cardDownside: 'החיסרון',
  cardCost: 'מה זה עולה',
  cardTax: 'מס',
  cardWhereMet: 'איפה עוד פוגשים את זה',
  cardInTheBank: 'באפליקציה של הבנק:',
  cardInAds: 'בפרסומות:',
  cardVersus: (name: string) => `לעומת ${name} ›`,
  allTerms: 'כל המונחים ›',
  cardBack: '‹ חזרה',

  // The questions
  horizonUnknownPlaceholder: '?',

  // ── Package K: the workbench, the full list, the charts ──
  workbenchSummary: (returnPercent, inflationPercent, selling, plans) =>
    `תשואה ${returnPercent}\u00a0· אינפלציה ${inflationPercent}\u00a0· ${selling ? 'מכירה בסוף' : 'בלי למכור'}\u00a0· ${plans}`,
  plansTicked: (count) => (count === 1 ? 'מסלול אחד מסומן' : `${count} מסלולים מסומנים`),
  changeAssumptions: 'לשנות ›',
  hideAssumptions: 'לסגור ›',
  exampleTag: 'דוגמה',
  thingTip: {
    sp500:
      'מדד S&P\u00a0500: 500 החברות הגדולות בארה״ב. אפשר לקנות אותו בכמה דרכים, וההשוואה מראה כמה יישאר בכל אחת.',
    other: 'מדד אחר, מניה או אג״ח בבורסה מסוימת: ההשוואה בין הבנקים ובתי ההשקעות שמוכרים אותם.',
    bonds: 'אג״ח של המדינה, שמחזיקים עד שהיא פודה אותן בתאריך קבוע.',
    short: 'קרן כספית או פיקדון: לכסף שצריכים בקרוב, או שלא רוצים שירד.',
  },
  waysHint: 'סמנו כמה דרכים כדי לראות אותן זו לצד זו.',
  waysTip:
    'דרך היא מה שמחזיק את מדד S&P\u00a0500 בשבילכם: קרן סל בבורסה בתל אביב או בחו״ל, קרן מחקה, או מסלול של קופת גמל. לכל דרך עלות משלה, ומס משלה.',
  levers: 'הנחות והגדרות',
  waysTable: 'הדרכים, מהטובה לפחות טובה',
  waysTableHint: 'בכל דרך: המסלול הזול ביותר בשבילכם',
  wayAndWho: 'דרך · דרך מי',
  productCostNote: (amount) => `הקרן עצמה: ${amount}`,
  costAYearTip: 'כל מה שהכסף מפסיד בשנה, כאחוז מהתיק, כמו דמי ניהול של קרן.',
  costAYearProduct: ' כולל את הקרן עצמה: דמי הניהול שלה, המס שהיא מאבדת על דיבידנדים, והפער ממנו.',
  productItself: 'הקרן עצמה',
  productItselfTip:
    'מה שהקרן שמחזיקה את מדד S&P\u00a0500 לוקחת לעצמה, בלי קשר לבנק: דמי הניהול שלה, המס על הדיבידנדים שהיא מקבלת והפער בינה ובינו.',
  showWayPlans: (count) => `כל ${count} המסלולים של הדרך ›`,
  hideWayPlans: 'להסתיר ›',
  noPlansTicked: 'לא סומן מסלול להשוואה: הוסיפו אחד ב״מה משווים״.',
  inTodaysMoneyAxis: 'בכסף של היום',
  waysChartsNote: 'כל דרך מצוירת במסלול הזול ביותר שלה.',
  // ── The redesign (REDESIGN.md Part 2): each stream adds its page words
  // here, in both files. Words about money come from the core. ──
  // W2: the answer card (§2.2–§2.4): pills, "בחרתם", the waiting line, the
  //     plan's heading, "נשארה החלטה אחת / נשארו N החלטות".
  answerCard: 'התשובה',
  confidenceChip: { A: 'כאן קשה לטעות', B: 'ההבדל קטן', C: 'תלוי במה שחשוב לכם' },
  linePill: { Clear: 'ברור', Tie: 'תיקו', Rules: 'לפי הכללים' },
  pillYourCall: 'החלטה שלכם',
  pillChosen: 'בחרתם',
  pillWaiting: 'אחרי הבחירה',
  lineWaiting: 'יתברר אחרי ההחלטה למעלה',
  thePlan: 'התוכנית שלכם',
  decisionsLeft: (count: number) =>
    count === 1
      ? 'נשארה החלטה אחת. כשתבחרו, התוכנית שלכם תופיע כאן.'
      : `נשארו ${count} החלטות. כשתבחרו, התוכנית שלכם תופיע כאן.`,
  yourMonthlyExpenses: 'ההוצאות שלכם בחודש',
  // P3: the plan and the path to it (§4.5). The steps' words are the core's.
  pathHeading: 'איך הגעתם לזה',
  stepYours: 'בחירה שלכם',
  stepChange: 'שינוי',
  stepClose: 'סגירה',
  whyClear: 'למה זה ברור?',
  whyClearHide: 'להסתיר את ההסבר',
  toHow: 'איך עושים את זה, צעד אחר צעד',
  beforeChange: (amount: string) => `לפני השינוי: ${amount}`,
  planUpdated: (amount: string) => `התוכנית עודכנה: ${amount}`,
  // W3: the outcome chart (§2.6): the reference lines' legend, the axis.
  // (The chart's words about money are the core's: `outcomeWords`.)
  outcomeReturnLabel: 'התשואה השנתית שהחישוב מניח למדד המניות הרחב',
  // W4: how to do it and where to buy (§2.8, §2.9): the empty state, the
  //     reference row's label.
  howHeading: 'איך עושים את זה',
  howNothingYet: 'עוד לא בחרתם',
  howEmpty: 'בחרו למעלה, או הקישו על שורה בגרף, ותראו כאן מה קונים, איפה ואיך.',
  searchLabel: 'מה מחפשים באפליקציה:',
  placeholderNote: '____ ו\u2011[—] מסמנים שם שעוד לא בדקנו: האתר לא ממציא אותו.',
  compareAnyway: 'להשוות בכל זאת ›',
  referenceRow: 'נקודת השוואה',
  sameAmount: 'אותו סכום',
  tie: 'תיקו',
  yourBankRow: (name: string) => `הבנק שלכם: ${name}`,
  // W5: the gain split and the horizons (§2.7, §2.9): headings, columns.
  gainTitle: 'כמה מהרווח נשאר אצלכם',
  gainBarLabel: (name, parts) => `${name}: ${parts}`,
  horizonsTitle: 'ואם התאריך יזוז?',
  horizonsLead: 'אותן הפקדות, לתאריך אחר: מה הייתה התשובה אז. הכול בכסף של היום, אחרי עמלות ומס.',
  horizonsIn: (years) => `בעוד ${yearsText(years)}`,
  horizonsOpen: (when) => `${when}: להציג את התשובה לתאריך הזה`,
  horizonsWhen: 'מתי',
  horizonsWhat: 'מה מתאים אז',
  horizonsAmount: 'כמה יהיה',
  horizonsDeposited: 'הפקדתם',
  horizonsStockRange: 'טווח של מניות:',
  // P2: the questions card (REDESIGN.md §4.2): "מתי", the examples, the status line.
  moneyTitle: 'הכסף שלכם',
  examplesButton: 'דוגמאות',
  examplesTitle: 'לראות דוגמה',
  examplesWait: 'המספרים שלכם יחכו לכם',
  exampleShownNow: 'מוצגת עכשיו',
  exampleOf: (name) => `דוגמה: ${name}`,
  yourNumbers: 'המספרים שלכם',
  makeThemYours: 'שנו אותם לשלכם',
  backToYours: 'חזרה למספרים שלכם',
  alreadyInvested: 'הכסף הזה כבר מושקע, בתיק בבנק שלי',
  whenTitle: 'מתי תצטרכו את הכסף?',
  whenDate: 'בעוד זמן ידוע',
  whenRetirement: 'בפרישה',
  whenAnyDay: 'לשעת חירום',
  whenUnknown: 'עוד אין תאריך',
  whenIn: 'בעוד',
  whenAgeToday: 'הגיל שלי היום',
  whenRetireAt: 'גיל הפרישה',
  whenAnyDaySentence: 'כמו כרית ביטחון: כסף שצריך להיות זמין כבר למחרת.',
  whenUnknownSentence: 'בלי תאריך. התשובה תראה מה מתאים לטווחים שונים.',
  fillAges: 'מלאו את הגיל שלכם היום ואת גיל הפרישה.',
  // "₪100,000 ועוד ₪2,000 בחודש, 3 שנים", "₪200,000 מושקעים, 20 שנה".
  exampleMoney: (now, monthly, invested) =>
    now && monthly
      ? `${now} ועוד ${monthly} בחודש`
      : now
        ? `${now}${invested ? ' מושקעים' : ''}`
        : `${monthly ?? '₪0'} בחודש`,
  exampleIn: (months) => (months % 12 === 0 ? yearsText(months / 12) : monthsText(months)),
  exampleRetirement: (age, retireAt) => `מגיל ${age} עד ${retireAt}`,
  exampleNoDate: 'בלי תאריך',
  // P4: the phone (REDESIGN.md §4.3): the collapsed facts, the answer's bar,
  //     the new decision's pill, a decision as a table.
  factsChange: 'לשנות',
  factsDone: 'סיום',
  toAnswer: 'לתשובה',
  answerAfterChange: 'התשובה אחרי השינוי',
  answerNow: (amount) => `התשובה עכשיו: ${amount}`,
  amountRange: (low, high) => `${low} עד ${high}`,
  newDecision: (question) => `נפתחה החלטה: ${question}`,
  tableEnd: 'מה יהיה בסוף',
  tableAsks: 'מה זה דורש',
  tableFits: 'מתאים אם',
  moreAboutOptions: (count) =>
    count === 2
      ? 'עוד על שתי האפשרויות'
      : count === 3
        ? 'עוד על שלוש האפשרויות'
        : `עוד על ${count} האפשרויות`,
}
