# The answer, redesigned: design rules and plan of record

**Status:** the plan of record, written on 2026-10-19 from a design session with the site's owner. That session followed a beginner‑usability audit of the live site, and is the project's source of truth for design. `CLAUDE.md`, `README.md` and `web/tests/CLAUDE.md` were brought in line with it the same day.

The previous plan of record (the guide and the workbench) was carried out on 2026‑10‑19, and is in git: `git show d14e955:REDESIGN.md`. Its table of the guide's rules is kept here as Appendix A, because the code and the tests implement it.

**Where the code stands:** as of 2026-10-20 Part 2 is built, with the owner's later decisions:

- principles 17 and 18, and the §2.8 amendment on "מדד";
- no assumptions sheet and no override in the guide (the yearly return is a field on the stock row);
- the visitor's bank an optional fact among the questions ("הבנק שלי · לא חובה"), since asking it in where to buy failed the review;
- above the horizon lines, the data decides between states A and C.

Part 4, the owner's review of what was built (2026-10-21), is designed and not built.

The review against Part 1's checklist is `docs/redesign/review-2026-10-20.md`. The worked examples' structure holds, except for the deviations the review and the final report list.

**Audience:** whoever changes the page, and in particular an agent that will plan this work and hand it to subagents. Subagents start without the session's context, so everything they need is in this file, in `docs/redesign/mockup.dc.html`, and in the sources below.

**Order of precedence:** Part 1 → Part 4 → Part 2 → the mockups → the audit report's Part 3. The report's Part 3 is older: its row of assumption switches ("הנחנו ש…", §2) was **rejected** by the owner, and so were the interest‑rate buttons in the first mockup. Where code, a test or a comment contradicts Part 1, Part 1 wins. When it's unclear, ask the owner.

**Sources:**

| What                                                                        | Where                                                                                                 |
| --------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Live site                                                                   | https://orishamir.github.io/Israeli-Broker-Search/                                                    |
| Audit report: 27 findings, transcripts, the first redesign proposal         | `/mnt/c/Users/ori/development/audit-broker-search/reports/06-beginner-usability-tester-2026-10-19.md` |
| Screenshots cited in the report                                             | `/mnt/c/Users/ori/development/audit-broker-search/reports/06-beginner-usability-tester-2026-10-19/`   |
| Mockup source (reference for structure, copy and numbers; not code to port) | `docs/redesign/mockup.dc.html`                                                                        |
| Mockup, viewable (the owner's account only)                                 | https://claude.ai/artifact/TZTmXqM2oEmjzQk6F9dKmZ                                                     |

The mockup file is in a design‑canvas format: HTML with `{{holes}}`, `<sc-for>` and `<sc-if>`, plus a `class Component` script that holds all the data and copy for the three examples. Read it for the behaviour and the Hebrew copy; build the real thing in the site's own stack, style and dark theme. The mockup itself is light.

---

## Part 1: Design principles

These rules apply to every screen of the site, not only to the answer. Each one says what to do, which case taught it, and how to check it. Principles 1–9 came from the owner's feedback and matter most; 10–16 came from the audit's findings.

### From the owner's feedback

The owner's own words, which the first principles follow from:

> "We shouldn't project a false narrative of a 'perfect solution' for the customer. We can and should display our suggestion. If it's a 'you can't go wrong in your case' suggestion then maybe we can allow ourselves to display it more confidently. But we have to remember that the choice isn't that simple. (The way to remember that isn't to simply add more tooltips. It should be a UI/UX way.)"

> "You only answer the questions when they are relevant, and the user can clearly see the established connection between what the site asks him to choose, why it's up to him, and what difference it would make."

**1. Don't present a perfect answer.**

- **The rule:** the site can't know a visitor's whole situation. An answer that pretends to fit perfectly is either distrusted ("yeah sure bro") or, worse, believed. Show how sure the site is, and what is still the visitor's call. Do it **through structure and layout, never by adding disclaimers or tooltips.**
- **The case:**
  - On the default apartment example, the answer flips from an Israeli CPI‑linked bond (via a new account at Excellence) to a money‑market fund (in the bank app). The flip happens between an assumed average Bank of Israel rate of 3.5% and 3.75%, with the two options ₪300–450 apart.
  - A "what fits" badge went to the lower of two near‑equal options (deposit ₪15,960 vs bond ₪16,030).
  - The answer named Excellence as the cheapest broker, with brokers ₪86 apart, and once in a tie.
- **The check:** for every answer, can a visitor tell whether it's a clear call, a near‑tie, or theirs to make? Would the answer flip under plausible versions of what the site can't know? If so, it mustn't look like a verdict.

**2. Ask only when the answer changes the outcome.**

- **The rule:** if every answer to a question leads to the same result in this situation, don't ask it. State the conclusion instead, and say you checked when that reassures.
- **The case:**
  - In the 5‑year bonus example, "do you have a securities account?" changed one tag. The owner called it clutter.
  - In the 3‑year apartment example, "can the date move?" changed nothing. It became a sentence: "גם אם התאריך יכול לזוז קצת, 3 שנים קצר מדי למניות לפי הכללים של האתר".
- **The check:** for each question, list its answers and what each produces _in this scenario_. If all outputs are the same, delete the question.

**3. Ask in context, not up front.**

- **The rule:** a question belongs at the point of the answer it decides, as a decision whose consequences are visible. It doesn't belong in a questionnaire before the answer, or in a row of settings or assumptions.
- **What the visitor sees together:** what's being asked, why only they can answer it, and what each answer leads to.
- **The case:** the first mockup had a row of switches at the top, "הנחנו ש…" (we assumed that…). The owner rejected it in favour of the bonus example's in‑place decision.
- **The check:** no settings or assumption rows; every question to the visitor sits next to the options it chooses between.

**4. A decision is options side by side, not a form field.**

- **Each option shows:**
  - what you get, in ₪, and how certain it is;
  - what it asks of you: effort, a locked period, waiting out a fall, a new account;
  - when it fits ("מתאים אם…");
  - a choose button worded in the visitor's own terms, often as the answer to the question ("לא, אצטרך את הכסף בדיוק אז").
- **Under the options:** the difference in ₪, and what could reverse it.
- **After choosing:** the chosen option is marked; the others stay **visible but dimmed**; the choice can be changed at any time.
- **The case:** the bonus example's "safe or growth" decision, which the owner singled out as the model to follow.

**5. Don't infer preferences from facts.**

- **The rule:** "has no securities account" doesn't mean "won't open one". Making that assumption hides a decision from the visitor.
- **Don't** fix it by adding an "I don't care" option. **Do** expose the trade‑off, so the visitor weighs the effort against the money:
  - Apartment: opening an account buys ₪1,206 over 3 years, and that edge reverses above a ~3.6% average rate.
  - 30‑year saving: it buys ₪90,186.
- Word each option so it holds either way: "צריך חשבון מסחר. אם כבר יש לכם אחד, זו עוד פקודה."
- **The check:** for every statement the page makes or implies about the visitor, ask "did they tell us this?". If not, it's a guess. Either remove what depends on it, or turn it into a visible trade‑off.

**6. Ask the visitor only what only the visitor can know.**

- **Ask about:** their flexibility, their priorities, their situation (an emergency cushion, a study fund through work).
- **Don't ask for forecasts.** Turn uncertainty about the world into information: a range ("₪176,045 אם הריבית תהיה בממוצע 4%") and a switch point ("אם הריבית תהיה בממוצע מעל כ‑3.6%, הקרן הכספית משאירה יותר").
- **The case:** the first mockup asked the visitor to pick an average Bank of Israel rate. It was removed.

**7. The same kind of question takes the same form everywhere.**

- **The rule:** one answer structure (§2.2), one decision component (§2.4), one confidence vocabulary (§2.3).
- **The case:** date flexibility was a top‑of‑page switch in one example and the deciding question in another. The owner flagged it, and preferred the deciding‑question form.

**8. A decision that depends on another appears only after it.**

- Until then, its line reads "יתברר אחרי ההחלטה למעלה".
- **Examples:** the choice between safe options appears only after "safe" is chosen; "open an account or buy through your bank?" appears only after "ETF" is chosen.

**9. Fix causes, not symptoms, and think them through.**

- When a problem appears, look for its structural cause before adding an option, a tooltip or a note. The owner rejected the quick fix twice: an "I don't care" option, and more tooltips.
- Redesign from the visitor's questions, not from the current screen.

### From the audit findings

**10. Every visual answers one named question, and can be read without a "?".**

- If a sentence answers the question better, use the sentence. The chart by deposit, unclear even to its author, became one sentence under the best plan for that reason.
- Label every mark that matters directly. Never identify anything by colour alone. No unexplained shading. Spend the axis's height where the options differ.
- **The cases:**
  - The comparison chart labelled 3 of its 5 lines.
  - Its shaded band changed colour with the product it belonged to.
  - Its axis ran from ₪0 to beyond ₪250k, while the safe options ended ₪1,206 apart (about 1 pixel).
  - The ring said "הפקדה 35%", which reads as a place money goes to.

**11. A visual must never contradict the answer.**

- Plot what the answer is based on: after‑tax value, in today's money.
- **The case:** in the retirement example, the US‑ETF line sat above the recommended line for 20 years and dropped below it only at the last point, where tax was applied.

**12. Compare like with like, against what the visitor put in and against doing nothing.**

- Use one unit everywhere (today's money).
- Show the total deposited. Always show the current‑account (עו״ש) comparison; the owner likes it.
- **The cases:**
  - The ring's percentages fit nominal shekels, while the rest of the page uses today's money.
  - The horizon table compared totals with very different deposits (₪124k vs ₪580k) and showed none of them.

**13. Precision matches certainty.**

- Round headline amounts, and give ranges for uncertain outcomes.
- Treat ties as ties: no rank numbers and no "הזול" when options are within noise.
- Show differences rather than ranks.
- Each number appears in one place. The audit found the same final amount five times on one page.
- Something every place of a kind shares is said once, for the kind, not under each row.

**14. Make the last step concrete.**

- A visitor must be able to type what to buy into their app. Use the identity line in §2.8.
- Never write "מדד" alone. Lead with what the product is, and give its trade name second.
- **The cases:**
  - "קרן מחקה מדד מניות רחב" came with no name or security number.
  - "אג״ח ממשלתי צמוד מדד" never said it's an Israeli government bond; "מדינת ישראל" appeared nowhere on the page.

**15. Compare from the visitor's own situation.**

- Take the visitor's own bank as the baseline: "₪2,900 יותר מאשר בבנק שלכם".
- Show that rather than a ranking of providers. When staying at their bank changes which product is best, say so.

**16. Be honest with data.**

- Never invent numbers. Where data is missing, use a clearly marked placeholder (`[₪—]`, `____`).
- Every number shown comes from the calculation engine, not from copy.
- Words about the past say they're the past ("ב‑N מתוך M תקופות…"), and are never rounded to "never".

### From the owner, after the plan (2026-10-19)

**17. Rank only what earns the same; the rest is drawn beside, and what to hold is the visitor's call.**

- **Ranked against each other:** different products that hold the same thing, by what each leaves after fees and tax. For example, a fund in Tel Aviv, a US ETF and a study fund's track on the S&P 500; or a money‑market fund, a deposit and a bond held to maturity.
- **Drawn beside, not ranked:** what earns differently (the index against the safe kinds), with its risk.
- **Not compared at all:** which indexes to hold and how to spread money between them. That's the visitor's call, and the site doesn't model it.
- **A fund's name** is an example to search for ("דוגמה, לא המלצה"), never a pick between funds.
- **The index itself is an example too.** The page calls what to buy "מדד מניות רחב", never the S&P 500. It names the S&P 500 only where it is what was measured (the history, and the basis of the numbers), and says so: "החישוב לפי מדד S&P 500, כדוגמה למדד רחב". Which index to hold, and whether to spread between several, is shown as the visitor's call: its own step in the plan, marked "בחירה שלכם" as calmly as a step the site is sure of. What to consider when choosing comes later, once that explanation is written; until then the page doesn't link to it.
- **The case:** the owner, on letting the visitor set the yearly return: one would presumably want to diversify, so the site mustn't recommend one ETF over another.
- **The case (2026‑10‑21):** the plan read "קרן סל על מדד S&P 500, בארה״ב", and the answer's title "מדד S&P 500, לטווח של 30 שנים", as if the site had chosen the index. The owner: the S&P 500 should only be an example; choosing an index is more complicated, out of the site's scope, and the page should say it isn't obvious.

**18. When a principle starts to strain, question the principle.**

- **The rule:** follow the principles and the design. But if keeping a principle starts to need a stretch, stop and ask whether the principle itself is wrong, incomplete or missing a case. A stretch looks like:
  - a reading of its words that the visitor wouldn't recognise;
  - an awkward screen or an empty question;
  - a workaround that exists only to satisfy the letter of a rule.

  Then propose fixing, changing or adding a principle, rather than bending the page to fit.

- **Who decides:** a change to a principle is the owner's call. Propose it with the case that strained it, keep following the current principle meanwhile, and record the change here with its case, as principle 17 was.
- **The case:** principle 17 came from exactly this. Keeping "ask only what only the visitor can know" (6) would have removed the visitor's control over the yearly return. The fix was to say what the site does and doesn't compare, not to argue the control away.
- **The check:** for every choice justified by a principle, would the visitor, reading the result, agree it serves them? If the honest answer is "only by a technicality", raise it.

### From the owner, after the redesign (2026-10-20 and 21)

**19. An example is a starting point, not a fact about the visitor.**

- **The rule:** an example fills only fields the visitor can see and change. It never answers a decision, and never sets anything that isn't a field on screen. The page calls the numbers an example exactly while the fields still hold the example's numbers. Once the visitor changes one, the example is no longer marked; if they put it back, it's marked again.
- **The case:** the chips under "בשביל מה הכסף?" read as the visitor's purpose, and stayed lit after the numbers changed. Four of them set things no field showed:
  - כרית ביטחון set "needed any day": with 36 months typed in, the answer still said "כסף שאולי תצטרכו בכל יום";
  - עוד לא יודעים answered the cushion decision with "not yet" and ₪17,000 of expenses (BUT‑13);
  - לפרישה turned the date into ages, and the only way back was another chip;
  - כבר יש לי תיק בבנק marked the money as already invested.

  A lit chip also claimed what the engine didn't know: לדירה promised a date that can't move, while the answer under it asked whether it could.

- **The check:** for each example, list everything it sets. Is each item a field on screen? Does it answer any decision in advance? Change one field: does the page stop calling it an example? Change it back: is it marked again?

**20. A control looks like what it does, and does only that.**

- **The rule:**
  - It looks like what it does: a link takes you somewhere, a button does something here, and a switch turns a setting on and off.
  - It changes only what its words name, never another control's state.
  - Each setting has one control, under one name, that turns it both on and off.
  - What it changes appears where the visitor is looking, or the page takes them there.
- **The case:** "אני יודע / יודעת מה אני רוצה לקנות ולהשוות ›" looked like a link, and turned on כל האפשרויות, a switch across the page with a different name. Clicking it again did nothing. The panel it opened already had the guide's pick chosen, against its own words. "להשוות בכל זאת ›" does the same, and so did "כל 10 המסלולים ›" (BUT‑20).
- **Known exceptions:** those two controls stay until the workbench is redesigned. The owner deferred that redesign on 2026‑10‑20.
- **The check:** for each control: does it look like what it does? Does its label name everything it changes? Does any other control change the same setting? Can the visitor see the result from where they clicked?

**21. What the visitor is using never moves.**

- **The rule:** a field the visitor is typing in, or a control they just pressed, stays exactly where it is: not up, down or sideways, not by a pixel, and not by a smooth glide either. Nothing above it or beside it may appear, disappear or change size because of what they did. Content that comes and goes uses space kept for it from the start, or floats over the page. This rule is never traded off against anything else.
- **The case:** in the 2026‑10‑20 mockups, typing a new monthly amount over an example's amount removed the "אלה מספרים לדוגמה" line above the fields, and the field being typed in jumped up under the cursor. The line was right to exist; its place was wrong. `holdChosen` in `scroll.ts` already does this for decisions.
- **The check:** at desktop and phone width, use every field and control and watch it. Does it stay exactly where it was? Does anything above it or beside it appear, vanish or resize?

**22. Trying an answer shows what changed, in place, and can be taken back.**

- **The rule:** a visitor who doesn't know why a question matters learns it by trying one answer, then another, and watching what changes. So:
  - **The layout holds still between tries.** Only content changes: numbers, words, which option is marked.
  - **What changed stays findable until the next try:**
    - a number changes where it is, and the visitor can see what it was;
    - a state (a line becomes a decision, a different option is marked) changes where it is, and stays marked as changed;
    - the structure (a decision appears or goes, or the answer becomes a different kind) changes only below what the visitor touched, and opens gradually rather than appearing at once;
    - a screen reader hears one short sentence saying what changed.
  - **A change out of sight leaves a sign in sight.** Where the answer is off screen, the visitor can see that it changed, and how, without being taken away from what they're using.
  - **Every try can be taken back.** Trying an example or an option never loses what the visitor entered. The way back sits where they're looking, as ↺ does for a changed fee.
- **The case:**
  - in the 2026‑10‑20 mockups, each answer to "when will you need the money?" showed different fields, so the bank field and the examples jumped with every try;
  - on today's site, tapping an example on a phone scrolls the page to the answer, so comparing two examples means scrolling back each time;
  - in both, loading an example erased the numbers the visitor had typed.
- **The check:** at desktop and phone width, try every answer of every question in turn. Did anything move except content? After each try, can you point to what changed without searching, and hear it with a screen reader? Can you get back to what you had?

### Review checklist: run it on every screen and every example

- [ ] **Questions:** list every question the page asks the visitor. For each one:
  - does its answer change the outcome _here_ (2)?
  - is it asked where it decides (3)?
  - is it something only the visitor can know (6)?
  - is it shown as options with consequences (4)?
  - does it take the same form as similar questions elsewhere (7)?
  - is it hidden until what it depends on is answered (8)?
- [ ] **Assumptions:** list every statement the page makes or implies about the visitor. Did they tell us (5)?
- [ ] **Confidence:** would the answer flip under the plausible versions in §2.3? If so, is it presented as a near‑tie or as the visitor's decision, not as a verdict (1)?
- [ ] **Visuals:** for each one:
  - which question does it answer (10)?
  - is every mark labelled?
  - does it agree with the answer (11)?
  - can it be read with no "?"?
- [ ] **Numbers:** for each number:
  - is it in today's money (12)?
  - is it rounded to match its certainty (13)?
  - does it appear only once (13)?
  - is it comparable with the numbers next to it (12)?
  - does it come from the engine (16)?
- [ ] **Last step:** can a beginner type what to buy, and where (14)?
- [ ] **Ranking:** is anything ranked against what earns differently, or a fund named as a pick rather than an example (17)?
- [ ] **Quick fixes:** was any problem solved with a tooltip, note or extra option where a structural fix exists (9)?
- [ ] **Examples:** does each example set only fields on screen and answer no decision? Is it marked exactly while the fields hold its numbers (19)?
- [ ] **Controls:** does every control look like what it does, change only what it names, and be the only control for its setting (20)?
- [ ] **Stability:** does any field or control move while the visitor uses it, or anything above or beside it appear, vanish or resize (21)?
- [ ] **Trying:** after each answer tried, does only content change, can the visitor find what changed (and hear it), and can they get back what they had (22)?

---

## Part 2: The design

### 2.1 Page order

1. **Inputs:** the visitor's facts only: what they have now, the monthly amount, when they'll need the money, and their bank. No row of assumptions.
2. **The answer card:**
   - the confidence chip (§2.3) with a one‑line note;
   - a one‑line title and a short body stating what's clear;
   - the answer lines (§2.2);
   - then either **"התוכנית שלכם"**, a one‑sentence plan shown once every decision is made, or "נשארה החלטה אחת / נשארו N החלטות" while some are open.
3. **"מה יהיה לכם ב‑<month year>"** (§2.6): the outcome chart, which is also the list of options.
4. **"איך עושים את זה":** the chosen option, or a row clicked in the chart, placed next to the answer rather than in the side column. Until something is chosen it shows an empty state: "בחרו למעלה, או הקישו על שורה בגרף, ותראו כאן מה קונים, איפה ואיך."
5. **"כמה מהרווח נשאר אצלכם"** (§2.7): shown only when fees or tax drive a decision on the page.
6. **"ואם התאריך יזוז?"** (§2.9): the horizon section. It isn't in the mockup.
7. **Details on demand**, as in‑place expanders, not tooltips: an optional over‑time chart (after‑tax), the fee details, and the method and sources.

The workbench behind **כל האפשרויות** (the full table, the growth, lost‑to‑fees, breakdown and by‑deposit charts, the editor) is out of scope unless the owner says otherwise. Part 1 still applies to it.

### 2.2 The answer lines

Every answer is built from the same lines, in this order:

- **קודם:** only when a step must come first, such as an emergency cushion.
- **סוג המקום:** safe or stock index.
- **הדרך:** deposit, money‑market fund, Israeli bond, ETF, study fund, provident fund…
- **איפה:** which provider.

Each line is in exactly one of three states:

| State                          | Shows                                            | Pill                     |
| ------------------------------ | ------------------------------------------------ | ------------------------ |
| Stated by the site             | one sentence                                     | ברור / תיקו / לפי הכללים |
| Decision                       | the question, then the decision component (§2.4) | החלטה שלכם → בחרתם       |
| Waiting on an earlier decision | "יתברר אחרי ההחלטה למעלה"                        | אחרי הבחירה              |

### 2.3 Confidence states (the chip)

The site works out, for the visitor's inputs, whether its pick holds up. It reruns the comparison under plausible versions of what it can't know:

- the average Bank of Israel rate ±0.5 point;
- inflation at 1% and at 3%;
- a stock return of 5% and of 10%, plus the historical worst and median for the horizon;
- provident‑fund fees at the average and at a negotiated rate.

| State | Chip               | When                                                                                                                              | Look                                                                                                                           |
| ----- | ------------------ | --------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| **A** | כאן קשה לטעות      | The same direction wins in every version, by far more than the noise margin.                                                      | The only state with the accent colour, the accent border and "מה שמתאים".                                                      |
| **B** | ההבדל קטן          | Options fall within the noise margin, or their order flips between versions.                                                      | The choice between them is presented as a decision about convenience or certainty, with the ₪ difference and the switch point. |
| **C** | תלוי במה שחשוב לכם | The option with the higher expected outcome has a downside the safe one doesn't, and only the visitor can say which matters more. | The deciding question is the first decision.                                                                                   |

The noise margin starts at the larger of ₪1,000 and 1% of the deposits, until the owner decides otherwise (§2.12).

### 2.4 The decision component

- **Content:**
  - the question;
  - one sentence on why it's the visitor's to answer;
  - 2–3 options, each with:
    - a title;
    - an amount, with its certainty in words ("ידוע כבר היום", "בריבית של היום", "אם המדד יעלה 10% בשנה");
    - up to 3 facts;
    - an optional "מתאים אם…";
    - a button label;
  - the difference: ₪, and what could reverse it.
- **Behaviour:**
  - The chosen option gets a strong border; the others dim but stay visible.
  - Choices can be changed. Changing one re‑evaluates which dependent decisions appear.
  - Chart rows dim to match the choices, and "איך עושים את זה" follows them.
- **Accessibility:**
  - real `<button>`s with `aria-pressed`;
  - the group labelled by the question;
  - targets at least 44 px;
  - text contrast at least 4.5:1 in the site's dark theme;
  - works with a keyboard, in right‑to‑left layout, and at phone width (options stack).
- **When not to use it:** if one option wins in every version, by a lot, and nothing about the visitor could rule it out, it isn't a decision. State it. If something about the visitor _could_ rule it out (an employer's study fund, for example), it is a decision, and that condition goes in "מתאים אם".

### 2.5 The three worked examples: the acceptance reference

The amounts are from the live site on 19/10/2026, in today's money, after fees and tax. They'll change as the data changes, so **compare structure in tests, not numbers**. A winner fixed in a test breaks with every monthly data update, so write each expectation as a rule the core must keep. The Hebrew copy is in `docs/redesign/mockup.dc.html`.

**Apartment in 3 years** (₪100,000 now, ₪2,000 a month, the visitor's bank Hapoalim). State **B**.

| Line      | State                | Content                                                                                                                                                                                                  |
| --------- | -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| סוג המקום | stated, ברור         | Safe, not stocks: 3 years and a fixed date. Even if the date can move a little, 3 years is too short for stocks under the site's rules. (This replaces a date question, which wouldn't change anything.) |
| הדרך      | decision             | "מה חשוב לכם יותר: הכי פשוט, או לדעת כבר היום כמה יהיה?"                                                                                                                                                 |
| איפה      | waiting; then stated | Money‑market fund → "in your bank's app". Bond → "Excellence, Altshuler or Meitav, at most ₪149 apart; about ₪2,900 more than at your bank" (תיקו).                                                      |

The **הדרך** decision has two options:

- **Simplest: money‑market fund.** ₪173,728 at today's rate, ₪176,045 if the rate averages 4%. In the bank app, can be sold any day.
- **Known in advance: Israeli CPI‑linked bond.** ₪174,934, known today. Needs a trading account ("if you already have one, it's one more order"). Held until it matures in 2029. The shekel bond is mentioned as a variant: ₪174,010.

The difference shown under them: the bond leaves ₪1,206 more at today's rate; the order flips above an average rate of about 3.6%; and a bond bought through the visitor's own bank would leave ₪171,940, less than the money‑market fund.

There is **no** question about having an account (principle 5) and **no** question about the date (principle 2).

**Bonus, 5 years** (₪15,000 now, nothing monthly). State **C**.

| Line      | State                              | Content                                                                                                                                                      |
| --------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| סוג המקום | decision                           | "אם ב‑2031 הבורסה תהיה בירידה, תוכלו לחכות עוד שנה‑שנתיים?"                                                                                                  |
| הדרך      | waiting; then a decision or stated | After "safe": a decision, "איזו דרך בטוחה?". After "growth": stated, a fund on a broad stock index (לפי הכללים; the numbers use the S&P 500 as the example). |
| איפה      | waiting; then stated               | Follows the choice: Bank Jerusalem for the deposit; the bank app for the money‑market fund; "Excellence or Altshuler (tie), Meitav ₪597 less" for growth.    |

The **סוג המקום** decision has two options:

- **Safe**, about ₪15,700–₪16,030. Button: "לא, אצטרך את הכסף בדיוק אז".
- **Growth**, ₪19,925 if the index rises 10% a year; it ended a 5‑year period in a loss in 135 of 1,110 periods since 1926 (in dollars, before fees and tax). Button: "כן, אפשר לחכות".
- The difference shown under them: about ₪3,900 more if the index rises 10% a year, against the risk of ending below ₪15,000 in the year the money is needed.

The **"איזו דרך בטוחה?"** decision, after "safe", has three options within ₪330 of each other:

- a 5‑year fixed deposit, ₪15,960, locked until 2031;
- an Israeli CPI‑linked bond, ₪16,030, needs a trading account, can be sold before maturity;
- a money‑market fund, ₪15,700, in the bank app.

There is **no** question about having an account (principle 2).

**₪500 a month for 30 years** (nothing now). State **A** for the type of place; the remaining decisions are about how.

| Line      | State                              | Content                                                                                                                                     |
| --------- | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| קודם      | decision                           | "יש לכם כבר כסף בצד ל‑3 חודשי הוצאות?"                                                                                                      |
| סוג המקום | stated, ברור                       | Stock index, 30 years. The index didn't end a single one of 810 thirty‑year periods since 1926 in a loss (in dollars, before fees and tax). |
| הדרך      | decision                           | "איפה ייצבר הכסף: קרן סל, או קרן השתלמות שלכם?"                                                                                             |
| איפה      | waiting; then a decision or stated | After "ETF": a decision, "לפתוח חשבון חדש, או לקנות דרך הבנק שלכם?". After "study fund": stated, "in your study fund".                      |

The decisions and their options:

- **קודם (the cushion).**
  - "I have one": all ₪500 goes to the index from this month.
  - "Not yet": first ₪24,000 into a money‑market fund over about 4 years, assuming ₪8,000 of monthly expenses (the visitor types theirs).
- **הדרך (where the money builds up).**
  - **ETF on a broad stock index** (calculated with the S&P 500 as the example): ₪427,357. Anyone can open one, through a trading account.
  - **Study fund, stock track** (calculated the same way): ₪468,537. "מתאים אם" the visitor has one and can add ₪500 a month. The figure is for a self‑employed person; the employee figure is a placeholder.
  - Difference: about ₪41,000, mostly from the tax exemption.
- **איפה (after choosing the ETF).**
  - **A new account at Interactive:** ₪427,357, about 0.46% a year.
  - **Through the visitor's bank, Hapoalim:** ₪337,171, about 1.57% a year.
  - Difference: ₪90,186, mostly fees. This is where principle 5 matters most.

**"כמה מהרווח נשאר אצלכם"** compares Interactive with Hapoalim:

|             | Kept     | Tax     | Fees    |
| ----------- | -------- | ------- | ------- |
| Interactive | ₪247,357 | ₪92,637 | ₪19,673 |
| Hapoalim    | ₪157,171 | ₪74,614 | ₪53,948 |

The site still has seven more situations (לילד, כרית ביטחון, ירושה, לפרישה, עוד לא יודעים, כבר יש לי תיק בבנק, אני יודע מה אני רוצה לקנות). Apply the same lines and principles to them, and run the review checklist on each. Two known traps:

- **לפרישה:** show a monthly‑annuity estimate, and prompt the visitor to count the pension they already have (audit finding BUT-5).
- **כבר יש לי תיק בבנק:** don't assume what the portfolio holds. That's a question only the visitor can answer, so ask it as a decision (BUT-6).

### 2.6 "מה יהיה לכם ב‑<month year>": the outcome chart

- **Rows.** One row per option, at most 5, also serving as the list of options. Each row has its name and short tags on the right (right‑to‑left), a mark on a shared horizontal ₪ axis (in today's money, after tax), and the amount.
- **Reference lines.** Two lines run through every row: "מה שהפקדתם" (solid) and "אם הכסף יישאר בעו״ש" (dashed). The legend gives their amounts.
- **Marks.**
  - **A safe option with a known amount:** a dot.
  - **Money‑market fund:** a dot at today's rate, on a short bar spanning the rate versions (3.25%–4% in the example).
  - **Stock option:** a bar from the historical worst to the historical best for this horizon, with ticks at the median and at the 10% assumption, each labelled. Under the bar, a note gives the count of past periods that ended in a loss.
- **Axis.** Spans the data: a little below the lowest bad case to a little above the highest typical value. A tail beyond that gets an arrow and a number, not a clipped band.
- **Excluded options.** An option the rules exclude (stocks for a fixed date in 3 years) stays dimmed, with a one‑line reason.
- **Interaction.** Rows dim to match the visitor's choices. Clicking a row shows its "איך עושים את זה".
- **Not in the mockup.** The ends of the historical range aren't in the mockup (`[₪—]`). The engine must provide them, in today's ₪, after fees and tax.
- **Optional over‑time view.** Behind an expander: after‑tax value at every point, a label on every line, shading with a fixed meaning that's labelled on the chart, and an axis fitted to the lines.

### 2.7 "כמה מהרווח נשאר אצלכם"

- One sentence gives the deposits ("תפקידו ₪180,000…"); the bars don't draw them.
- One horizontal bar per compared option, all on the same ₪ scale, showing **only the gain**. Each bar is split into kept / tax / fees, with every segment labelled in ₪.
- One takeaway sentence.
- Everything in today's money.
- It replaces the donut ("לאן הולך כל שקל בסוף"). Show it only where fees or tax drive a decision on the page.

### 2.8 The identity line and naming rules

Under every product name, one line of facts, always in this order:

who issues it / what it is · where it trades · currency · what it's linked to · when you can take the money out · the name or number to search for

Examples:

- "הלוואה למדינת ישראל · נסחרת בבורסה בתל אביב · בשקלים, צמודה למדד המחירים לצרכן · נפדית ב‑2029 · לחיפוש: ממשלתי צמוד ____ (הסדרה שנפדית הכי קרוב לתאריך שלכם)"
- "קרן שמחזיקה את 500 החברות הגדולות בארה״ב · נסחרת בבורסה בארה״ב, בדולרים · נמכרת בכל יום מסחר · למשל VOO (דוגמה, לא המלצה)"

Naming rules:

- **"מדד" never stands alone.** Write "מדד המחירים לצרכן" or "מדד S&P 500".
  - **Amended by the owner (2026-10-20):** "מדד" may stand alone only to refer back to an index already named in the same unit, and only if no other index is named in that unit. A unit is an option card, a stated line, a chart row with its label, or an identity line. A tag or fragment shown on its own isn't a unit, so it names the index in full.
  - **Why:** the rule exists for ambiguity (BUT‑25: "צמוד מדד" and "מדד מניות רחב" a few lines apart). Read literally, it repeated "מדד S&P 500" eight to ten times on one card ("₪427,000 אם המדד יעלה 10% בשנה", under a title that names the S&P 500), or forced less precise rephrasings.
  - **Allowed:** "המדד" in an option titled "קרן סל על מדד מניות רחב". **Not allowed:** "המדד" in a bond option that also says "צמודה למדד המחירים לצרכן", or a "המדד" whose index is named only elsewhere on the page.
- **Lead with what the product is.** "אג״ח צמודה של מדינת ישראל", not "אג״ח ממשלתי צמוד מדד". The trade name goes in the search slot, because that's what the app shows.
- **Separate the wrapper from its content.** "קרן השתלמות, מסלול מניות".
- **Colour:** one colour per type of product, the same in every view, and no two options in near‑identical hues.

### 2.9 Where to buy, and the horizon

**Where to buy:**

- Use the visitor's own bank as the reference row ("נקודת השוואה"), and show the others as ₪ differences from it.
- Options within noise are a tie, and the answer names none of them as cheapest. The core's tie‑break (`compare_with_broker_caveats`) may still decide the order; the page shows a tie.
- If staying at their bank changes which product is best, say so where the decision is made, as the apartment example does.

**The horizon section, "ואם התאריך יזוז?"** (it replaces the ladder):

- **What each row shows:**
  - what will have been deposited by then;
  - the pick for that horizon under the _visitor's_ answer about date flexibility (fixes BUT-3);
  - the outcome as a range and as "על כל ₪1 שתפקידו".
- **The check:** a row's click opens exactly what the row showed.

### 2.10 Removed

- The row of assumption switches ("הנחנו ש…"), which only the mockup had.
- The interest‑rate buttons.
- The donut.
- The alternatives line chart as the main chart.
- The paragraph explaining the band, and the rotated "אחרי מכירה ומס" label.
- "בהנחה של 10% בשנה" under answers it doesn't drive.
- Repeats of the same final amount.
- The accent "approved" styling outside state A.
- Charts that need a "?" to be understood.
- "למה דווקא זה" as a heading. It asserts there's a single right answer.

### 2.11 What the engine must provide

Check what already exists before building anything; don't duplicate the engine's logic in the UI. Keep as much logic as possible in Rust (`CLAUDE.md`).

| Need                                                                                                                          | Evidence on the live site                                                                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Each option's outcome at the date, in today's money, after tax                                                                | The alternatives list shows these values.                                                                                                 |
| The same comparison under versions of the assumptions (§2.3)                                                                  | The answer recomputes when **ההנחות שמאחורי המספרים** changes.                                                                            |
| The switch point between two options (for example, the rate at which the order flips)                                         | None. Solve for it, or search across versions.                                                                                            |
| The historical worst, median and best at the horizon for stock options, in today's ₪, after fees and tax                      | `history.rs`'s `Band` has worst, p10, median, p90 and best, in dollars and before fees and tax.                                           |
| The gain split in today's ₪: kept, tax, fees, fund costs                                                                      | The donut shows percentages, apparently in nominal shekels. The table shows ₪ fees and tax in some scenarios.                             |
| Each product's result at the visitor's own bank                                                                               | The full ranking table shows per‑provider results, including the branch and online plans.                                                 |
| The total deposited                                                                                                           | Easy to derive.                                                                                                                           |
| The bond series that matures nearest the visitor's date (name and number), and example funds with security numbers or tickers | Probably new data. Only VOO and CSPX appear, under **כל האפשרויות**. Until real data exists, show a marked placeholder; never invent one. |
| An annuity estimate for retirement                                                                                            | Probably new.                                                                                                                             |
| The decisions themselves (question, options, dependencies, whether each changes the outcome)                                  | Belongs in the core, as `follow_ups` does today, so a Rust test can check principle 2 for every situation.                                |

### 2.12 Decisions still with the owner

- **The noise margin:** a fixed ₪ amount, a share of the deposits, or the spread between the versions. Start with the larger of ₪1,000 and 1%.
- **Which versions to rerun, and how far apart.** This sets how often state A appears.
- **Whether the over‑time view stays at all.**
- **When to ask about the cushion.** Today's rule asks only when there's a monthly amount and no date (Appendix A). The 30‑year example asks it with a date. Should it be asked whenever money would go somewhere a forced sale could lose?

Ask the owner when you reach any of these. Don't settle them yourself.

### 2.13 What this reverses, in the previous plan and in the code

1. **"No named index"** (the previous plan's §12, the morning of 2026‑10‑19) is **reversed**.
   - **Was (2026‑10‑19):** name the S&P 500 wherever it is what's measured and bought.
   - **Now (narrowed 2026‑10‑21):** name the S&P 500 only where it is what's measured, always as an example. Call what to buy "מדד מניות רחב". Give example funds or tickers marked "דוגמה, לא המלצה" (principle 14), and show that choosing the index is the visitor's call (principle 17).
   - **Where, place by place:**
     - the answer's title: "מדד מניות רחב, לטווח של 30 שנים", not "מדד S&P 500, …";
     - the plan: "קרן סל על מדד מניות רחב", and under the amount "החישוב לפי מדד S&P 500, כדוגמה למדד רחב";
     - the path of choices under the plan (§4.5): a step "איזה מדד", marked "בחירה שלכם" with an empty circle, no colour of its own and no "?";
     - the outcome chart and "where to buy": the row is "קרן סל על מדד מניות רחב", and the chart's caption says "המניות לפי מדד S&P 500, לדוגמה";
     - "איך עושים את זה": "קרן סל על מדד מניות רחב"; what to search for gives examples, such as VOO on the S&P 500 or a fund on a world index [____], marked "דוגמה, לא המלצה";
     - the history ("ב‑N מתוך M תקופות… מאז 1926") stays as it is, and adds "הנתונים של מדד S&P 500": that is what was measured, so the name is right there.
   - **To change:** `Way::kind_name`, `basis_sentence`, the check in `guide_words.rs` (it should now fail on the S&P 500 named as what to buy), and the texts in `describe.rs`, `describe/card.rs` and `web/src/lib/text/he.ts`.
2. **The register.**
   - **Before:** "מה שמתאים לכסף כזה" headed every answer, and "למה דווקא זה" headed the reasons.
   - **Now:** the confidence chips (§2.3), with "מה שמתאים" in state A only. "Never 'ההמלצה שלנו', no 'you should'" still holds.
3. **The follow‑up cards** (`follow_ups`: flexibility asked at 36–84 months, the cushion when there's no date).
   - **Now:** decisions in context (§2.2, §2.4), shown only when the answer changes the outcome (principle 2).
   - **What stays:** what is asked, and when, stays the rules' (Appendix A), except where §2.5 differs and §2.12 is settled.
4. **The alternatives chart** (lines per kind, the shaded band, "only three end labels") → the outcome chart (§2.6).
5. **The donut** → "כמה מהרווח נשאר אצלכם" (§2.7).
6. **The ladder** → the horizon section, which follows the visitor's date flexibility (§2.9).
7. **Within (the top three, with the cheapest named in the answer)** → where to buy, relative to the visitor's bank, with ties shown as ties (§2.9).
8. **"כבר יש לי תיק בבנק"** → what the portfolio holds is asked as a decision (BUT-6).
9. **Retirement** → a monthly‑annuity estimate, and a prompt about the existing pension (BUT-5).
10. **Steps in the side column** → "איך עושים את זה" next to the chosen option, with the identity line (§2.1, §2.8).

---

## Part 3: Running the implementation

### Before planning

- Read Part 1 and Part 2, `docs/redesign/mockup.dc.html`, and the audit report: Part 2 (BUT-21 to BUT-27) and the High findings of Part 1 (BUT-1 to BUT-6).
- Read `CLAUDE.md` (stack, tests, traps, Hebrew, git workflow) and `web/tests/CLAUDE.md`.
- The previous plan (`git show d14e955:REDESIGN.md`) is context only.
- Look at the live site, or the dev server, on the three worked examples, to see today's behaviour.

### How to run it

- **Speed over a working `main`** (the owner's choice, 2026-10-19).
  - Between stages, the build, the tests and the commits may all be broken.
  - Commit at milestones. Run `CLAUDE.md`'s "Done means" in full once, at the end, and fix what it finds then.
- **Each subagent that runs in parallel gets its own git worktree** (`isolation: worktree`), so the Rust builds, `npm run wasm` and half‑finished edits don't collide.
  - Merge each into `main` as soon as it's done.
  - Then delete the worktree and its branch (`git worktree remove`, `git branch -d`). A worktree with its build takes over a gigabyte, so none are left lying around.
  - Work that runs alone can stay in the main tree.
- **Give parallel workstreams different files where possible**, so their merges don't conflict. If two need the same file, run them one after the other.
- **Never push** until the owner says so. A push to `main` deploys, and the deploy's tests would fail on a broken tree.

### Suggested workstreams

Adjust them to the code's real structure.

| #   | Workstream                                                                                                                                                                                                                                                                                                                                                                           | Depends on                       |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------- |
| W1  | **Core and data contract:** the per‑scenario answer structure (lines, line states, decisions, options, dependencies, plan), robustness versions and confidence state, switch points, historical ranges at the date, the gain split in today's ₪, results at the visitor's bank, deposits. Add the bindings. Define the data shape first, so the UI streams can build against a stub. | —                                |
| W2  | **Answer card:** chip, title and body; the answer lines; the decision component; plan and "remaining decisions"; all copy, in the core where it's about money (`CLAUDE.md`).                                                                                                                                                                                                         | W1's contract                    |
| W3  | **Outcome chart and option list** (§2.6), linked to the decisions and to "איך עושים את זה". Replaces `Alternatives` and `alternatives-chart.ts`.                                                                                                                                                                                                                                     | W1's contract                    |
| W4  | **"איך עושים את זה"**, the identity line and where‑to‑buy (§2.8, §2.9), plus a naming pass over all user‑facing copy (§2.13, item 1).                                                                                                                                                                                                                                                | W1's contract                    |
| W5  | **"כמה מהרווח נשאר אצלכם"** (§2.7, replaces `Donut`) and the horizon section (§2.9, replaces `Ladder`).                                                                                                                                                                                                                                                                              | W1's contract                    |
| W6  | **Cleanup:** the removals in §2.10 and §2.13; regression checks on **כל האפשרויות**; right‑to‑left layout, phone width, the dark theme, accessibility; the specs and `web/tests/CLAUDE.md`'s table.                                                                                                                                                                                  | W2–W5                            |
| W7  | **Review:** run the Part 1 checklist on every situation, plus the beginner tests in the audit report's Part 3 §10, and report what fails.                                                                                                                                                                                                                                            | each workstream, then at the end |

### Every subagent prompt must include

- this file's path, with the instruction to **read Part 1 before anything else and apply it to every choice**;
- its workstream's sections of Part 2, and the acceptance checks below;
- "If a design question isn't answered here, apply Part 1. If it's still open, return the question to the orchestrator instead of guessing." The orchestrator takes open questions to the owner.
- "Don't fix a design problem with a tooltip, a note or an extra option until you've looked for the structural fix (principle 9)."
- "If applying Part 1 needs a stretch, return the strain and a proposed principle instead of stretching (principle 18)."
- the files it owns, and what to check before handing back: that its own part compiles and its own new tests pass, as far as the rest of the tree allows. The whole tree may be broken between stages.

### Acceptance

- **Structure.** The three worked examples (§2.5) reproduce the structure exactly:
  - which questions appear, and which don't;
  - on which line each one sits;
  - the options;
  - the order of dependent decisions;
  - the confidence state.
- **Numbers.** Every number comes from the core. Nothing from the mockup is hard‑coded.
- **No empty questions.** In every situation, no question appears whose answers lead to the same outcome. Test this in Rust, by evaluating each question's answers.
- **No assumptions.** No statement about the visitor that they didn't give, and no row of assumptions.
- **Consistency.** Every decision uses the one decision component.
- **Charts.** They need no "?" to be read. Every mark that matters is labelled. Every value is after tax, in today's money.
- **Where it runs.** It works by keyboard, at phone width (the Samsung and the narrowest iPhone), in the dark theme, and in right‑to‑left layout.
- **Verification.** Check on the running dev server in a real browser, not only in unit tests. Run `CLAUDE.md`'s "Done means" in full, at the end.

---

## Part 4: The owner's review of the built redesign (2026‑10‑21)

**Status:** designed with the owner on 2026‑10‑20 and 21, after reviewing the built Part 2 on desktop and phone. Not built. Where Part 4 and Part 2 disagree, Part 4 wins. Principles 19–22 and the narrowed S&P 500 rule (principle 17, §2.13 item 1) came from the same review and are already in Part 1 and Part 2.

**References:** `docs/redesign/review-2026-10-21/` holds the accepted mockups, one `.dc.html` file per frame, in the same design‑canvas format as `docs/redesign/mockup.dc.html`. Read them for structure, behaviour and copy; they aren't runnable here. Several were drawn before later wording decisions (for example §4.4), so **where a frame's copy and this text differ, this text wins.** The canvas itself is the owner's (https://claude.ai/artifact/7xeysuBmA57wUhy7y744yC), with each settled page in its version history.

### 4.1 The answer lines' names

Everywhere: the answer card, the path (§4.5) and the phone.

| Was       | Now           |
| --------- | ------------- |
| קודם      | כסף לחירום    |
| סוג המקום | מניות או בטוח |
| הדרך      | המוצר         |
| איפה      | איפה קונים    |

### 4.2 The questions card

**"מתי תצטרכו את הכסף?"** Four buttons in a 2×2 radio group: "בעוד זמן ידוע", "בפרישה", "לשעת חירום", "עוד אין תאריך". Under them, one box whose height never changes (principle 21), sized for the tallest of the four at the narrowest phone. It holds the chosen answer's fields or sentence:

- בעוד זמן ידוע: "בעוד [n]", and a years/months switch;
- בפרישה: "הגיל שלי היום [ ]" and "גיל הפרישה [ ]";
- לשעת חירום: "כמו כרית ביטחון: כסף שצריך להיות זמין כבר למחרת.";
- עוד אין תאריך: "בלי תאריך. התשובה תראה מה מתאים לטווחים שונים."

Frames: `WhenB.dc.html`, and the same inside `InputsCard.dc.html`.

**The examples** replace the row of example chips (principles 19 and 22):

- A "דוגמאות" button beside the card's title, "הכסף שלכם", opens a list over the fields: each example's name and its numbers, with the one on screen ticked. The list takes no space on the page.
- A status line under the title, always the same height: "דוגמה: <name>" in a quiet pill while the fields hold that example's numbers, and "המספרים שלכם" otherwise. While an example is on screen, its fields are tinted.
- The first visit loads one example, as today, with "שנו אותם לשלכם" beside the pill.
- Once the visitor has typed numbers of their own, picking an example keeps them, and the status line offers "↺ חזרה למספרים שלכם", which puts them back exactly. Trying several examples in a row still returns to the visitor's numbers, not to the previous example's. The bank stays as chosen.
- On phone, "דוגמאות" sits beside "לשנות" in the collapsed facts row (§4.3), and opens the list as a sheet from the bottom, titled "לראות דוגמה", with "המספרים שלכם יחכו לכם".
- The examples and their numbers stay today's (`examples.rs`). Each sets only fields on screen (principle 19).

Frames: `InputsCard.dc.html` (its `examplesMode` "menu" is the chosen design; "toggle" and "firstvisit" are the rejected alternatives) and `PhoneExA.dc.html`.

**The bank field** stays the platform's own `<select>` on phone (the owner, 2026‑10‑21).

### 4.3 The phone

- **The facts collapse to one line** once the answer is shown, such as "₪100,000 עכשיו, ₪2,000 בחודש, בעוד 3 שנים", with "לשנות", which opens the fields in place ("סיום" closes them), and "דוגמאות" (§4.2). The answer starts on the first screen.
- **The page never scrolls by itself** after a change (principles 21 and 22).
- **While the fields are open,** a slim bar at the bottom of the screen shows the answer's chip and amount after the change, and "לפני השינוי: …" before it, with "לתשובה". The visitor sees what changed without leaving the field.
- **A decision that opens below the screen** is announced by a pill at the bottom ("נפתחה החלטה: איזו דרך בטוחה?") that takes the visitor there when tapped. The button they pressed stays where it was.
- **A decision is a comparison table:**
  - two options are two columns, with the rows "מה יהיה בסוף", "מה זה דורש" and "מתאים אם", the difference right under them, and the rest behind "עוד על שתי האפשרויות";
  - three or more options are one row each, with the amounts in one column at the end, to compare at a glance.

Settled details:

- "Phone" is the site's existing phone breakpoint. On desktop the layout stays as it is, and decisions keep their side‑by‑side cards.
- "לפני השינוי" is the answer as it was when the visitor tapped "לשנות". It stays until the fields close.
- "סיום" closes the fields, and nothing scrolls. "לתשובה" closes them and takes the visitor to the top of the answer: a scroll the visitor asked for.
- The new‑decision pill goes away once that decision is on screen, or at the visitor's next change.

Frames: `PhoneFirst`, `PhoneEditing`, `PhoneChose`, `DecisionTable` and `ThreeOptions` (`.dc.html`).

### 4.4 Say what to do, and why: "בטוח", not "לא מניות"

"לא מניות" says what not to do, and not why. Everywhere it appears:

| Where                                   | Now                                                                                               |
| --------------------------------------- | ------------------------------------------------------------------------------------------------- |
| The chip's line (state B, safe)         | "מה שחשוב ברור: דרך בטוחה. איזו מהן, תלוי במה שנוח לכם."                                          |
| The title, when the safe ways are close | "דרך בטוחה, ואתם בוחרים איזו."                                                                    |
| Under the title, one safe way           | "זו הדרך הבטוחה היחידה כאן." (only where the route really has one: see §4.9)                      |
| Under the title, one way leads          | "בין הדרכים הבטוחות, זו משאירה יותר מהאחרות בכל גרסה שבדקנו (ריבית בנק ישראל, אינפלציה)."         |
| "מניות או בטוח", a short or known date  | "בטוח. במניות, ב‑N מתוך M תקופות של <the visitor's span> מאז 1926 היה נשאר בסוף פחות ממה שהופקד." |
| "מניות או בטוח", the 5‑year rule        | The same sentence, for the visitor's span, in place of "לפי הכללים של האתר".                      |
| "מניות או בטוח", money needed any day   | Unchanged: "בטוח: כסף שאולי תצטרכו בכל יום לא יכול לחכות שהבורסה תתאושש."                         |

N and M are the history the engine already computes for the span, and today shows only inside the chart: the S&P 500 as the example, after fees and tax, in today's money. Frame: `NotStocksProposal.dc.html`.

### 4.5 After choosing: the plan, and the path to it

When every decision is answered, or there were none, the card becomes the plan:

- the chip, "התוכנית שלכם", and the plan as one large sentence: "₪500 בחודש לקרן סל על מדד מניות רחב, בחשבון חדש באינטראקטיב";
- the amount, larger: "בערך ₪427,000", with "ב‑2056, אם המדד יעלה 10% בשנה", and under it "בכסף של היום, אחרי מס. תפקידו בערך ₪136,000. החישוב לפי מדד S&P 500, כדוגמה למדד רחב.";
- on phone, these fill the first screen.

Under it, "איך הגעתם לזה": a step for each line and decision, in the answer's order. Each step has its label (§4.1), the choice in bold, and one line saying what it hinged on. Three kinds of step look different:

- **Decided by the visitor:** a filled circle with a tick, and "שינוי", which opens that decision again in place of the step (the one decision component). Choosing an option closes it back into the step, and the plan above updates.
- **Decided by the site:** a filled circle with a tick, the tag "ברור", and "למה זה ברור?", which opens its explanation in place.
- **Still the visitor's, outside the site:** "איזה מדד" (principle 17). An empty circle, and the tag "בחירה שלכם" in the same neutral style as "ברור": no colour of its own, no "?". Its choice is "מדד מניות רחב", and its line: "המספרים כאן לפי מדד S&P 500, כדוגמה, כי יש לו ההיסטוריה הארוכה ביותר. איזה מדד לקנות, ואם לפזר בין כמה, אתם בוחרים. האתר לא משווה בין מדדים." Later, "מה כדאי לשקול?" will open an explanation in place, as "למה זה ברור?" does. Until the owner approves that explanation, nothing appears in its place, not even a greyed‑out link.

**Each line says what the decision hinged on, not only money.** The owner: a summary that says only "more" or "less" money doesn't summarise the decision. The line comes from the decision's own `why`, facts and "מתאים אם", and names what the choice gives, and what the other option would have given besides money. The lines, with the visitor's own amounts, years and names filled in:

| Step          | The choice                                       | Its line                                                                                                                       |
| ------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ |
| כסף לחירום    | יש לכם כבר כרית ביטחון                           | "לכן כל ה‑₪500 הולכים ל<the plan's product>. הכרית שומרת שהוצאה פתאומית לא תכריח אתכם למכור דווקא בירידה."                     |
| כסף לחירום    | קודם כרית ביטחון                                 | "קודם ₪<sum> לכרית, שאפשר למשוך ממנה כבר למחרת. אחר כך ה‑₪500 עוברים ל<the plan's product>."                                   |
| מניות או בטוח | מניות (stated by the site)                       | "הכסף מחכה <span>." with "למה זה ברור?"                                                                                        |
| מניות או בטוח | בטוח (stated by the site)                        | §4.4's sentence for the span.                                                                                                  |
| מניות או בטוח | בטוח (the visitor: "לא, אצטרך את הכסף בדיוק אז") | "תצטרכו את הכסף בדיוק ב‑<year>, אז אי אפשר לחכות שהבורסה תתאושש."                                                              |
| מניות או בטוח | מניות (the visitor: "כן, אפשר לחכות")            | "אם ב‑<year> הבורסה תהיה בירידה, תוכלו לחכות עוד שנה‑שנתיים."                                                                  |
| המוצר         | קרן סל (the study‑fund decision)                 | "מתאים כשאין לכם קרן השתלמות שאפשר להוסיף לה. בקרן השתלמות הרווח פטור ממס אחרי 6 שנים, אבל היא פתוחה רק לעצמאים או דרך מעסיק." |
| המוצר         | קרן השתלמות, במסלול מניות                        | "בקרן ההשתלמות שלכם הרווח פטור ממס אחרי 6 שנים. קרן סל הייתה פתוחה לכל אחד, בלי מעסיק."                                        |
| המוצר         | הכי פשוט: קרן כספית                              | "קרן כספית: הכי פשוט, ונמכרת בכל יום מסחר. <the other option> הייתה נותנת לדעת כבר היום כמה יהיה."                             |
| המוצר         | לדעת מראש: <bond or deposit>                     | "יודעים כבר היום כמה יהיה. קרן כספית הייתה פשוטה יותר, ונמכרת בכל יום מסחר."                                                   |
| המוצר         | איזו דרך בטוחה?: פיקדון                          | "ריבית ידועה מראש, אבל הכסף נעול עד <year>."                                                                                   |
| המוצר         | איזו דרך בטוחה?: אג״ח                            | "יודעים כבר היום כמה יהיה. צריך חשבון ניירות ערך, ואפשר למכור גם לפני הפדיון."                                                 |
| המוצר         | איזו דרך בטוחה?: קרן כספית                       | "הכי פשוט, ונמכרת בכל יום מסחר. הסכום בסוף זז עם ריבית בנק ישראל."                                                             |
| איפה קונים    | חשבון חדש ב<broker>                              | "פותחים חשבון פעם אחת. בבנק לא צריך לפתוח כלום, אבל העמלות שם גבוהות יותר, ומשולמות כל שנה."                                   |
| איפה קונים    | דרך הבנק שלכם                                    | "לא צריך לפתוח כלום. בחשבון חדש העמלות היו נמוכות יותר, אבל צריך לפתוח אותו פעם אחת."                                          |
| איפה קונים    | stated by the site (no decision)                 | The stated line's own reason, as today.                                                                                        |

For a decision not in this table (such as "מה יש בתיק?"), write the line the same way: the chosen option's "מתאים אם" or main fact, then what the other option gives that isn't money. The amounts stay in the decision, one tap away under "שינוי".

**"למה זה ברור?"**, for stocks over 30 years (the same shape for any span where stocks are stated):

- a lead on how unlikely a loss is, said as the past: no 30‑year period since 1926 ended below what was deposited;
- the count: all 810 thirty‑year periods since 1926, depositing every month as the visitor does. None ended below what was deposited, and even the worst left more than a money‑market fund;
- the crises those periods went through: the crash of 1929, the Second World War, 2000 and 2008. Drop the frame's "איבדה יותר משלושה רבעים מערכה": it isn't from the engine;
- four amounts: what's deposited (₪136,000), a money‑market fund (₪159,000), stocks in the worst period (₪281,000), stocks at 10% a year (₪427,000);
- "זה מה שקרה בעבר, לא הבטחה", and that the data is the S&P 500's, after fees and tax, in today's money.

Under the path, the button "איך עושים את זה, צעד אחר צעד" takes the visitor to that section. Frames: `AfterProposal.dc.html` and `AfterPhone.dc.html`.

### 4.6 The outcome chart (amends §2.6)

- Each row's amount sits next to its name.
- Every mark on a stock row's band is labelled with its amount: "הגרועה בעבר", "10% בשנה", "חציון העבר", "הטובה".
- The yearly return field sits in the stock row: "אם המדד יעלה [10]% בשנה".
- "הפקדתם" and "בעו״ש" are drawn on the chart as labelled lines.
- The facts under a row's name are plain text ("הריבית משתנה, בבנק או בבית השקעות"), not pills.
- The caption: "בכסף של היום, אחרי עמלות ומס. המניות לפי מדד S&P 500, לדוגמה. הקישו על שורה כדי לראות איך עושים את זה."
- Only options the visitor could choose get a row: no stock row where stocks aren't offered (§4.9).

Frame: `ChartProposal.dc.html`.

### 4.7 Where to buy, and "כמה מהרווח נשאר אצלכם" (amends §2.7 and §2.9)

- Every amount is rounded the same way.
- "דרך בנק" is one row, as a reference point: the range from the most expensive bank to the cheapest (Mizrahi online, in the example).
- The comparison is a difference: "באינטראקטיב: בין ₪88,500 ל‑₪105,000 יותר מאשר דרך בנק, לאורך 30 שנים."
- In the bars showing where the gain goes, every part has its name and amount on it or beside it, and "הקרן עצמה" has a colour of its own. A sentence under them says where the difference mostly comes from ("ההבדל בא בעיקר מהעמלות…").

Frame: `BuyProposal.dc.html`.

### 4.8 "איך עושים את זה": the facts (amends §2.8's identity line)

The identity line's pills look like buttons and say nothing about what each one is. They become labelled rows: one fact per row, beside the question it answers, the same questions in the same order for every product.

| Question       | The money‑market fund                         | The CPI‑linked government bond                        |
| -------------- | --------------------------------------------- | ----------------------------------------------------- |
| מה זה          | קרן נאמנות שמלווה כסף לזמן קצר למדינה ולבנקים | הלוואה למדינת ישראל, שנסחרת בבורסה בתל אביב           |
| כמה מרוויחה    | בערך את ריבית בנק ישראל, פחות דמי הניהול      | תשואה שנקבעת ביום הקנייה, על גבי האינפלציה            |
| מתי אפשר למשוך | בכל יום מסחר                                  | נפדית ב‑<year>. אפשר למכור גם לפני, במחיר של אותו יום |
| מטבע           | שקלים                                         | שקלים, צמודה למדד המחירים לצרכן                       |

- §2.8's parts map onto these: where it trades joins "מה זה", and the linkage joins "מטבע". The name to search for keeps its own box. Keep today's words for "מה זה", "מתי אפשר למשוך" and "מטבע" for every product.
- "כמה מרוויחה" is new. For each kind:
  - a money‑market fund: "בערך את ריבית בנק ישראל, פחות דמי הניהול" (today's "הריבית זהה עם ריבית בנק ישראל" leaves out the fee);
  - a fixed‑rate deposit: "ריבית קבועה, שהבנק קובע ביום ההפקדה";
  - a deposit that can be withdrawn any day (§4.9): "ריבית משתנה, שהבנק קובע";
  - מק״מ, and an unlinked government bond: "תשואה שנקבעת ביום הקנייה";
  - a CPI‑linked government bond: "תשואה שנקבעת ביום הקנייה, על גבי האינפלציה";
  - an index fund or ETF on a broad stock index: "כמו המדד: עולה ויורד";
  - a study fund's, provident fund's or savings policy's stock track: "כמו המסלול: עולה ויורד".
- Where to buy it isn't a fact here: "איפה לקנות", below, says it.
- The search box ("מה מחפשים באפליקציה") and the steps stay as they are in this round.

Frame: `HowFacts.dc.html`.

### 4.9 Money needed any day (changes the any‑day route)

Today the any‑day route offers only the money‑market fund (`safe_pool` in `guide/route.rs`), because the only deposits the engine knows are locked until the end of their term. The answer then says it's "the only safe way here", and the chart sets it against stocks, which aren't offered. Instead:

- Compare every place the money can be taken from the next day, like any other safe answer: the money‑market fund; a deposit that can be withdrawn any day; and מק״מ (the Bank of Israel's short‑term bill), sold on any trading day at that day's price, in a securities account. Rank them by what they leave, with the usual confidence states. The title and chip follow the numbers.
- The chart shows only those rows, with the "הפקדתם" and "בעו״ש" lines, and no stock row.
- Their facts say what else differs: the deposit stays at the visitor's bank; the fund is bought in an app; מק״מ needs a securities account, and its price moves a little before it's repaid.
- **The deposit's data.** The Bank of Israel's Excel behind קו המשווה (`g060a.xls`), which `policies/deposit-rates.py` already reads, has daily and weekly terms per bank, in sheet L7.6.1a (fixed rate); the script skips them today. Most daily deposits are at a variable rate (press reports citing the Bank of Israel put the system's average at about 1.3%–1.5%), so first look for a variable‑rate sheet in the same file. Use, in this order: the variable‑rate sheet's daily term; else L7.6.1a's daily term. A bank without a published rate isn't compared for this option, exactly as for fixed deposits today. The bank is chosen the way the site chooses it for fixed deposits today. Record what's used in `policies/sources.md`, and add the term to the staleness test, as for the other terms.
- **מק״מ** is the unlinked government bond kind, at the one‑year end of the yield curve (`policies/yield-curve.py`), held for the year, with the costs and tax the site already applies to an unlinked government bond. Its name: "מק״מ, מלווה קצר מועד של בנק ישראל".
- **The fallback, without waiting for the owner:** if neither sheet has a daily term per bank, build this instead, and say so in the final report. Keep the fund alone, drop the chart for this route, and say in the "המוצר" line why not a deposit: "פיקדון רגיל נעול עד סוף התקופה. פיקדון שאפשר למשוך בכל יום משלם בדרך כלל ריבית נמוכה יותר." מק״מ is still added, if the yield curve supports it.

Frame: `AnyCompare.dc.html`, the "כרית ביטחון" example (₪30,000); [₪—] marks what the engine doesn't compute yet.

### 4.10 "ואם התאריך יזוז?" (amends §2.9, and Appendix A's fixed list of dates)

- The rows are built around the visitor's date, and the visitor's row is marked "התאריך שלכם". The dates, from the list 1, 3, 5, 10 and 20 years:
  - **earlier:** the list's closest date before the visitor's, and also 5 years (or 1 year, if 5 is that closest date or after it) when it's earlier still;
  - **the visitor's own date**, exactly as they gave it, months included;
  - **later:** the list's next date after the visitor's; from 20 years on, the visitor's date plus 5 years.

  So 30 years shows 5, 20, 30 and 35; 10 years shows 1, 5, 10 and 20; 3 years shows 1, 3 and 5. The engine must compute dates it doesn't today, such as 35 years.

- Each row: "מתי", "מה מתאים אז" (with "אם תצטרכו בדיוק אז:" and "אם תוכלו לחכות:" where both apply), "כמה יהיה" and "הפקדתם".
- No "על כל ₪1" column: "כמה יהיה" next to "הפקדתם" already says it, and the ratio was one more thing to decode.

Frame: `HorizonProposal.dc.html`.

### 4.11 Not in this round

Nothing in Part 4 is open. These are left as they are, on purpose:

- **"מה כדאי לשקול כשבוחרים מדד"** (§4.5): a draft is in `IndexProposal.dc.html`. It ships only once the owner approves it. Until then, nothing in its place.
- **"מה מחפשים באפליקציה"**: dense (14 funds, fund numbers, "כלומר תיקו"), and not yet reviewed. Unchanged.
- **The "אני יודע / יודעת מה אני רוצה לקנות" link**, and "להשוות בכל זאת ›": unchanged until the workbench is redesigned (principle 20's known exceptions). The owner postponed it.
- **The workbench** ("כל האפשרויות"): unchanged, except where it shares a component changed here; then it must keep working.

### 4.12 Running it

Part 3's way of working holds: speed over a working `main`, a worktree for each parallel stream, and never push. Suggested workstreams, to adjust to the code:

| #   | Workstream                                                                                                                                                                                                                                 | Depends on            |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------- |
| P1  | **Engine, data and copy in the core:** the any‑day options (§4.9), the horizon's dates (§4.10), the history count for every span (§4.4), the path's steps and lines (§4.5), the line names (§4.1), the S&P 500 naming pass (§2.13 item 1). | —                     |
| P2  | **The questions card:** "מתי", the examples, the status line and "חזרה למספרים שלכם", on desktop and phone (§4.2).                                                                                                                         | —                     |
| P3  | **The answer card and the path** (§4.4, §4.5).                                                                                                                                                                                             | P1                    |
| P4  | **The phone flow** (§4.3): the collapsed facts, the answer bar, the new‑decision pill, no scrolling, decisions as tables.                                                                                                                  | P2, P3                |
| P5  | **The chart, where to buy, the gain split and the horizon** (§4.6, §4.7, §4.10), and the facts in "איך עושים את זה" (§4.8).                                                                                                                | P1                    |
| P6  | **Review:** Part 1's checklist, including 19–22, on every example at desktop and phone width; the Hebrew copy; every number from the core.                                                                                                 | each, then at the end |

Every subagent prompt includes Part 3's list, and: "Part 4 amends Part 2, and wins where they disagree. Read the frame named in your section, but its copy may be out of date: the text here wins."

**This round runs unattended.** The owner isn't available to answer, so Part 3's "return the question to the orchestrator, who takes it to the owner" changes for this round:

- Nobody waits for an answer. A question Part 4 doesn't answer is settled by Part 1, then by the closest decision already in Part 4, then by keeping today's behaviour.
- Every such call is written in `docs/redesign/implementation-log-2026-10-21.md` (what, why, where), so the owner can review it.
- A step that fails three times, each with a different fix, is logged there as not done, and the work moves on.
- The round ends with `docs/redesign/implementation-report-2026-10-21.md`: what was built, per section of Part 4; what wasn't, and why; the log's calls; and the result of `CLAUDE.md`'s "Done means". Everything is committed to `main`, and **not pushed**: the owner reviews it and pushes.

---

## Appendix A: The guide's rules

These rules are unchanged by this redesign, except where §2.5 and §2.13 change how a question is asked or presented. `crates/core/src/guide/route.rs` implements this table, and `crates/core/tests/guide.rs` tests it row by row. Copied from the previous plan's §3.

Inputs the guide knows (`Answers`):

- the situation (optional);
- the money now, and the money each month;
- the horizon: months, or Unknown, or Retirement with the saver's age and retirement age;
- flexibility: Fixed or CanWait, today asked at 36–84 months;
- the cushion: Has, or HasNot with monthly expenses, today asked when the horizon is Unknown and the monthly amount is above 0;
- needs‑any‑day, set by the cushion situation.

The expected index return, inflation and the Bank of Israel's rate are the app's defaults (10%, 2%, the published rate) unless the workbench changes them.

| The money                             | Arena and what fits                                                                                                                                                                                                                                                              | Reasons the user reads (shape; exact texts in `describe.rs`, both languages)                                                                       | Drawn beside it                                                                   |
| ------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Needs-any-day, or horizon ≤ 12 months | Safe. Rank the safe kinds that fit; a money market fund wins unless a deposit leaves more _and_ needs-any-day is off.                                                                                                                                                            | "כסף שאולי תצטרכו בכל יום…": reachable next day; earns about the Bank of Israel's rate. A deposit: locked, said so.                                | Deposits by term, the money fund, the current account.                            |
| 13–36 months, a known date            | Safe. Rank: fixed deposit for the term, money market fund, government bond maturing at the date.                                                                                                                                                                                 | The horizon; one sum or monthly (deposits take one sum); what's known in advance.                                                                  | The others; a stock index fund with its band.                                     |
| 37–84 months, date Fixed              | Safe. Rank among the kinds whose end sum is **known in advance** (bond to maturity, fixed deposit ≤ 60 months); the money fund is an alternative ("הריבית משתנה").                                                                                                               | Fixed date → stocks could be down exactly then; so long → the money fund's rate isn't promised; one sum → the deposit locks, the bond can be sold. | A stock index fund with its band and "ב־N מתוך M תקופות של X שנים נגמרה בהפסד".   |
| 37–59 months, CanWait                 | Safe, as Fixed, but the reason says that from 5 years with flexibility stocks become reasonable.                                                                                                                                                                                 |                                                                                                                                                    | Both arenas.                                                                      |
| 60–84 months, CanWait                 | **Index.** Rank the index kinds (the ways) by what's left after tax.                                                                                                                                                                                                             | "אם הבורסה תהיה בירידה אז, תחכו": the rule; the history sentence for this horizon; what safety would have cost.                                    | The safe kinds, the current account.                                              |
| ≥ 85 months (7 years and more)        | Index. Rank the ways.                                                                                                                                                                                                                                                            | The horizon against the index's history for it; the cheapest way and why (fees after tax; the product's own cost).                                 | The safe kinds ("מה עולה הביטחון לאורך זמן כזה"), the current account.            |
| Retirement (age, retire-at)           | Index, with the provident fund for investment **as a pension** when the saver is ≥ 60 at the end; the study fund shown with who can open one.                                                                                                                                    | The tax-free pension against 25% in a brokerage account; the ceiling; withdrawable any time.                                                       | The brokerage account after tax, the study fund, the policy, the current account. |
| Horizon Unknown, monthly > 0          | Two-part answer: the cushion first if HasNot (money market fund until 3 months of expenses; months to fill = target ÷ monthly); then an index kind that can be opened any time (provident fund for investment or a broker's index fund, whichever leaves more). Then the ladder. | No cushion → sell in a fall; no date → a place you can open any time that grows.                                                                   | The ladder.                                                                       |
| Horizon Unknown, one sum              | The ladder, and its lesson.                                                                                                                                                                                                                                                      |                                                                                                                                                    |                                                                                   |
| Situation "כבר יש לי תיק בבנק"        | The bank's branch price list ticked; the guide dialog (where to find your fees; transfer, don't sell; managed funds' fees aren't counted); the editor in simple mode.                                                                                                            |                                                                                                                                                    | The same comparison.                                                              |
| "אני יודע מה אני רוצה לקנות"          | The workbench, with the ways to compare.                                                                                                                                                                                                                                         |                                                                                                                                                    | Everything.                                                                       |

**Fixed numbers:**

- the arena boundary: 7 years for a fixed date, 5 years with flexibility;
- flexibility is asked from 3 to 7 years;
- the cushion target is 3 months of expenses (the words say "3–6");
- deposits only up to 60 months (the Bank of Israel publishes none longer);
- the ladder's horizons are 1, 3, 5, 10 and 20 years, plus "any day" when there's a sum and no monthly amount. The horizon section (§2.9) replaces the ladder's presentation.

**Within an arena,** "what's left after tax" decides. Results equal to the shekel count as a tie, and a tie goes to the plan without a may‑cost‑more caveat. A plan the first deposit can't open is never the answer; the unlock line says what opens it.

Three engine facts behind the table:

- **A CPI‑linked bond's end sum isn't known in advance**, only its real value is (`Safety::RealValueKnown`). So it isn't ranked in the fixed‑date rows of 37 to 84 months, where "known in advance" decides, but it is drawn, and ranked in the shorter rows.
- **The study fund isn't the saver's to open:** it needs an employer or self‑employment (`Circumstance::StudyFundNeedsEligibility`). It's never the answer for someone who didn't say they can open one. In this redesign, that's what the study fund's "מתאים אם" says.
- **A bond held to maturity pays no sale fee and no conversion** (`Scenario.redeems_at_maturity`).
