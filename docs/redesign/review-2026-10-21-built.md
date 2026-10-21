# Review of the built Part 4, 2026-10-21 (P6)

**What was reviewed:** `main` after P1–P5 (`eb4bc3b`), built with `npm run wasm` and served by Vite on port 5186, with the exchange rates mocked as in `tests/fixtures.ts`.

- **The page:** Playwright scripts with the project's Chromium, at desktop 1280×800, iPhone SE (3rd gen) and Galaxy S24. They covered:
  - every example in "דוגמאות";
  - every answer of "מתי" (from "לדירה או למטרה בתאריך");
  - §2.5's three worked examples (the apartment also with Hapoalim);
  - each example decided to its plan twice, by the first option everywhere and by the last.
- **The core's words:** every example, with and without Hapoalim, along every decision path (76 paths), dumped through `AppState`. Every Hebrew string was scanned for:
  - a bare "מדד";
  - the S&P 500 named without "כדוגמה";
  - "לא מניות", placeholders, and "מה כדאי לשקול".
- **Stability (21):** each field and control of the questions card was used, and every box in the card measured before and after, together with the scroll. Each decision's pressed button was measured right after the press and once the answer settled.

**Key:** the numbers are Part 1's principles. **Fixed** means fixed in this round, with a test that fails without the fix (Rust) or a changed spec. **Logged** means written in `implementation-log-2026-10-21.md` under P6 for the owner.

## What failed, by example and width

| Example | Width | Failed | What was done |
|---|---|---|---|
| כבר יש לי תיק בבנק | all | 13: the horizon's "מה מתאים אז" said one fund three times ("קרן מחקה…, קרן מחקה… או קרן מחקה…") | Fixed (`horizon_fit`) |
| כרית ביטחון, and "לשעת חירום" | all | 14, 16: the difference called makam "אג״ח" ("דרך בנק, אג״ח הייתה משאירה…") | Fixed (`safe_way_decision`) |
| חיסכון מהמשכורת, לילד, בונוס, ירושה, לפרישה, עוד לא יודעים, כבר יש לי תיק | all | 17: the history's basis beside a figure, "(… לפי מדד S&P 500)", didn't say it's an example | Fixed (`on_basis`: "… כדוגמה") |
| לדירה, בונוס, כרית ביטחון (makam), עוד לא יודעים (bond) | all | 5: the plan said "בחשבון חדש ב…" for a place the site states, which may hold an account the visitor already has | Fixed: "בחשבון ניירות ערך ב…"; "חשבון חדש" only after "לפתוח חשבון" |
| כרית ביטחון with a bank named (and any tie that holds the visitor's bank) | all | 15: the visitor's bank was listed in the tie as one more bank ("לאומי, וואן זירו, מזרחי, הפועלים…") | Fixed: named first, as theirs ("הבנק שלכם (הפועלים), לאומי…") |
| עוד לא יודעים, the bond | all | 14: where to buy had only "דרך בנק" as a reference row, with nothing beside it (P5's point) | Fixed: where no house holds it, the banks are listed |
| בונוס, ירושה, עוד לא יודעים, לילד | desktop | 21, 22: a choice that opened a decision scrolled the page 420–490 px, carrying the pressed button off its place | Fixed: the pill announces it at every width, as §4.3 does on a phone |
| חיסכון מהמשכורת, "בפרישה" | desktop | 16: "לשכירים: [₪—]" under the study fund | Left: a marked placeholder is what principle 16 asks for where the engine has no figure |
| every dated example | all | 20, 22: a horizon row's tap moves the date and scrolls to the answer, with no visible sign that the row is a control and no way back to the visitor's date | Logged, with a proposal (P4's point) |
| every stock row | all | 13: one amount three times (the row, the "10% בשנה" mark, the plan) | Logged, with a proposal (P1/P5's point) |
| every example whose last choice makes the plan | all | 22: the card above the pressed button becomes the plan | Logged, with a proposal (P3's point) |

## What passed, everywhere

- **4.1:** the four line names, in the card, the path and on the phone.
- **4.2:** "מתי"'s 2×2 group and its one box, the "דוגמאות" list and the status line, and "↺ חזרה למספרים שלכם". Every field, the unit switch, "מתי", the bank, the box and an example left every box in the card where it was, at all three widths. Only the status line's words changed, in its reserved height. Each example sets only fields on screen (now a unit test on the core's own examples), and is marked exactly while they hold its numbers.
- **4.3:** on a phone the facts collapse to one line and the answer starts on the first screen. Picking an example doesn't scroll. The three‑option decision is a row each.
- **4.4:** "בטוח" with the visitor's span and the history's count, never "לא מניות".
- **4.5:** the plan and "איך הגעתם לזה", with the three kinds of step. "איזה מדד" is an empty circle marked "בחירה שלכם", and nothing stands in for "מה כדאי לשקול?".
- **4.6:** the amounts by the names, every band mark labelled, the return field in the stock row, and the two labelled lines. The any‑day route (כרית ביטחון, לשעת חירום) shows only its three rows: the fund, makam and the deposit that can be withdrawn any day. There is no stock row.
- **4.7:** "דרך בנק" as one row with a range, and the difference in words. The gain's parts are named on or beside their bars.
- **4.8:** the four labelled facts, the same questions in the same order.
- **4.10:** the dates around the visitor's: 3 years gives 1/3/5, 5 gives 1/3/5/10, 15 gives 5/10/15/20, 18 gives 5/10/18/20, 20 gives 5/10/20/25, 30 gives 5/20/30/35, and retirement at 35→67 gives 5/20/32/37. No date gives 1/3/5/10/20, none marked.
- **Copy:**
  - no bare "מדד" (only "איזה מדד", which the rule lets stand);
  - the S&P 500 is named only as an example or as what was measured, apart from the search box's fund names (§4.11 keeps the box);
  - the workbench's own texts in `he.ts` name it where the visitor chose it.
- **Numbers:** every figure on the guide comes from the core. The web builds only the examples' numbers line, from the fields.

## The streams' points

- **P4, "מתאים אם" behind "עוד" with three options on a phone:** kept folded. "מה זה דורש" already says what tells the any‑day options apart, so principle 4 holds on the row.
- **P5, the bond's where‑to‑buy with no places:** fixed (above).
- **P5, `outcome.spec.ts` re‑checking the core's words:** trimmed to structure.
- **P2, the filter in `exampleAnswers`:** a no‑op since P1, so it's removed. The unit test now checks the core's examples.
- **P4, the horizon row's scroll:** a scroll the visitor asks for. The strains are logged, as above.
- **P1/P5, the stock row's amount three times:** logged with a proposal.
- **P3, the card becoming the plan:** logged with a proposal.

## Measured, and not a failure

- On a phone, a choice that changes the card's head above it (the bonus's "בטוח" turns state C into B, a different title) is held still by `holdChosen`. The pressed button ends within a pixel of where it was (304.2 → 303.7 px on the iPhone SE).
- The last choice, which makes the plan, scrolls the page so that the new step stands where the option was: 62 to 1,456 px on the iPhone SE, 24 to 362 px on desktop. That's P3's mitigation, logged above.
- Where only banks hold a bond (no first deposit), where to buy doesn't say why no investment house is listed. Every house's plan needs a first deposit, and the index's unlock line exists only for the index. Left: the bank rows answer the visitor's question, which is where to buy.

## Tests run

- `cargo test`: all pass. The guide's tests were run again after the last core change.
- `cargo clippy --all-targets`: 0 warnings. `cargo fmt --check`: clean.
- `npm run wasm`.
- `npx vitest run`: 149 pass. Under load (the review's own browsers running), two answer‑card tests timed out at 5 s once, and passed alone and in the next full run.
- Playwright `answer`, `outcome`, `phone`, `questions`, `acceptance`: 116 pass. `layout`, `address`, `app`: 126 pass, 2 failed under load and passed on rerun.
- `npm run lint`: clean.
- `npm run test:perf` was not run here, though the scroll's behaviour changed. It's for "Done means".
