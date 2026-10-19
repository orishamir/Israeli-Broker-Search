"""Prints the names and numbers in `crates/core/src/examples.rs`: what a
visitor types into the bank's app to find what the site describes (REDESIGN.md,
principle 14). Every name and number is fetched, none written by hand:

- Israeli government bonds (ממשל צמודה, ממשל שקלית) and the Bank of Israel's
  bills (מק״מ): every series the Tel Aviv Stock Exchange lists, with its
  number and the day it's repaid in full.
- Israeli index funds (קרנות מחקות) on the S&P 500, unhedged, in shekels:
  every one on Maya, with its fees.
- Israeli ETFs (קרנות סל) on the S&P 500, unhedged: the exchange's numbers
  in `TEL_AVIV_ETFS` below, with their fees from Funder; the script warns of
  any other plain S&P 500 ETF it finds, so a new one is noticed.
- The Irish ETFs on the S&P 500 listed in Tel Aviv, and the US and European
  ones by ticker, with the fees and sizes their issuers publish.
- The shekel money market funds (כספית שקלית) on Maya, with their numbers
  and fees.

Each list is in order of size, as its source reports it, largest first: an
order that says nothing about which is better (REDESIGN.md, principle 17).

    python3 -I policies/examples.py [--write]

With `--write`, the block of the Rust file between its two `examples.py`
marker lines is replaced; without it the block is printed. `policies/sources.md`
says where each part comes from.
"""

import datetime
from decimal import Decimal
import html
import json
import re
import sys
import time
import urllib.request
from pathlib import Path

RUST = Path(__file__).resolve().parent.parent / "crates/core/src/examples.rs"
BEGIN = "// BEGIN examples.py"
END = "// END examples.py"
BROWSER = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/129.0 Safari/537.36"
)
# The exchange answers its API only with these (CLAUDE.md, "Researching a tariff").
TASE_HEADERS = {
    "Content-Type": "application/json",
    "Referer": "https://market.tase.co.il/",
    "User-Agent": BROWSER,
}
MAYA = "https://maya.tase.co.il/api/v1/funds/mutual"
MAYA_HEADERS = {
    "content-type": "application/json",
    "accept": "application/json, text/plain, */*",
    "origin": "https://maya.tase.co.il",
    "referer": "https://maya.tase.co.il/he/funds/mutual-funds",
    "user-agent": BROWSER,
}

# The exchange's own codes for the issuers of government debt (its
# `SubId`): the CPI-linked bonds, the fixed-rate shekel ones, and the Bank of
# Israel's bills. The variable-rate (ממשל משתנה) and the dollar ones
# (ממשל גלובל) are left out: the app doesn't model them.
CPI_LINKED, SHEKEL, MAKAM = "954", "926", "891"

# The plain, unhedged S&P 500 ETFs of Israeli managers ((4D), no "מנוטרלת
# מט״ח"), by the exchange's number.
TEL_AVIV_ETFS = [1146471, 1144385, 1149020, 1150333, 1165810, 1148162, 1201656, 1218916]
# The Irish ones listed in Tel Aviv: iShares', which holds the shares, and
# Invesco's, which swaps for the index.
FOREIGN_IN_TEL_AVIV = [1159250, 1183441]

# Direction marks the exchange's names carry, which only confuse a search.
MARKS = dict.fromkeys(map(ord, "‎‏‪‫‬"))


def get(url: str, headers: dict | None = None, data: bytes | None = None) -> bytes:
    request = urllib.request.Request(url, data=data, headers={"User-Agent": BROWSER, **(headers or {})})
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


def clean(name: str) -> str:
    return re.sub(r"\s+", " ", name.translate(MARKS)).strip()


# ─────────────────────────── The exchange ───────────────────────────


def entities() -> list[dict]:
    return json.loads(get("https://api.tase.co.il/api/content/searchentities?lang=0", TASE_HEADERS))


def security(number: int | str) -> dict:
    time.sleep(0.2)
    url = f"https://api.tase.co.il/api/company/securitydata?securityId={number}&lang=0"
    return json.loads(get(url, TASE_HEADERS))


def day(text: str) -> datetime.date:
    return datetime.datetime.strptime(text, "%d/%m/%Y").date()


def bonds(listed: list[dict]) -> list[tuple[str, int, str, datetime.date]]:
    """(name, number, linkage, repaid on), by the day each is repaid."""
    found = []
    for entity in listed:
        if entity["Type"] != 1 or entity["SubId"] not in (CPI_LINKED, SHEKEL, MAKAM):
            continue
        data = security(entity["Id"])
        repaid = day(data["RedemptionDate"])
        name = clean(entity["Name"])
        # A series' name ends in the month and year it's repaid in full
        # (0529: May 2029). The exchange's date is the last payment's, so the
        # two must agree: a date before it would be a part payment. A bill's
        # name is numbered otherwise, and it's repaid at once.
        if entity["SubId"] != MAKAM and (match := re.search(r"(\d\d)(\d\d)$", name)):
            month, year = int(match[1]), 2000 + int(match[2])
            if (repaid.year, repaid.month) != (year, month):
                sys.exit(f"{name}: repaid {repaid}, which its name doesn't say")
        linkage = {CPI_LINKED: "CpiLinked", SHEKEL: "Unlinked", MAKAM: "Unlinked"}[entity["SubId"]]
        if entity["SubId"] == CPI_LINKED and data["Linkage"] != "מדד המחירים לצרכן":
            sys.exit(f"{name} is linked to {data['Linkage']}")
        found.append((entity["SubId"], name, int(entity["Id"]), linkage, repaid))
    return sorted(found, key=lambda bond: bond[4])


def funder_fees(number: int) -> tuple[Decimal, Decimal, Decimal]:
    """An ETF's fixed fee, the most its variable fee can be, and its trustee's,
    in % a year, from Funder's page (the exchange's API doesn't give them)."""
    page = html.unescape(get(f"https://www.funder.co.il/etf/{number}").decode("utf-8", "replace"))
    costs = page[page.index("<strong>עלויות</strong>") :]
    # The three values follow the three headings; a cell that isn't a
    # percentage would shift them, so it stops the script instead.
    cells = re.findall(r'bg-grey2 brd_rht3">([^<]*)<', costs)[:3]
    if not all(re.fullmatch(r"\s*[\d.]+%\s*", cell) for cell in cells):
        sys.exit(f"Funder's fees of {number} read {cells}")
    fixed, variable, trustee = (Decimal(cell.strip().rstrip("%")) for cell in cells)
    return fixed, variable, trustee


def tel_aviv_etfs(listed: list[dict]) -> list[tuple]:
    by_number = {int(e["Id"]): e for e in listed if e["Type"] == 1}
    plain = re.compile(r"S&P 500|SP500")
    other = re.compile(r"מנוטרלת|Equal|Low Vol|Dividend|TOP|Scored|ממונפת|משולבת")
    for number, entity in by_number.items():
        name = entity["Name"]
        if (
            entity["SubTypeDesc"] == "קרן סל מניות"
            and plain.search(name)
            and not other.search(name)
            and number not in TEL_AVIV_ETFS
        ):
            print(f"warning: {number} {clean(name)} is a plain S&P 500 ETF not in TEL_AVIV_ETFS", file=sys.stderr)
    found = []
    for number in TEL_AVIV_ETFS:
        data = security(number)
        fixed, variable, trustee = funder_fees(number)
        found.append((clean(by_number[number]["Name"]), number, fixed + trustee, variable, data["MarketValue"]))
    return sorted(found, key=lambda etf: -etf[4])


def foreign_in_tel_aviv() -> list[tuple]:
    found = []
    for number in FOREIGN_IN_TEL_AVIV:
        data = security(number)
        found.append((clean(data["LongName"]), number, data["ISIN"], data["MarketValue"]))
    return sorted(found, key=lambda etf: -etf[3])


# ─────────────────────────── Maya ───────────────────────────


def maya_funds() -> list[dict]:
    funds, page = [], 1
    while True:
        body = json.dumps({"pageSize": 30, "pageNumber": page}).encode()
        batch = json.loads(get(MAYA, MAYA_HEADERS, body))
        funds += batch
        if len(batch) < 30:
            return funds
        page += 1
        time.sleep(0.2)


def is_sp500_index_fund(fund: dict) -> bool:
    """An unhedged S&P 500 index fund priced in shekels, as
    `index-tracking.py` picks them: all of it on the index, all of it in
    dollars."""
    assets = {asset["name"]: asset["weight"] for asset in fund.get("underlyingAssets") or []}
    index_fund = any(f["enabled"] and "קרן מחקה" in f["title"] for f in fund["features"])
    in_dollars = any(f["enabled"] and "נקובות ב- $" in f["title"] for f in fund["features"])
    return (
        index_fund
        and not in_dollars
        and assets.get("S&P 500 - NTR") == 100
        and assets.get('דולר ארה"ב') == 100
    )


def decimal(value: float | None) -> Decimal:
    """A fee as Maya writes it, without a float's last-digit noise."""
    return Decimal(str(value or 0))


def fee(fund: dict) -> Decimal:
    return decimal(fund["managementFee"]) + decimal(fund["trusteeFee"])


# ─────────────────────────── The issuers ───────────────────────────


def ishares(url: str) -> tuple[Decimal, dict[str, tuple[str, str]]]:
    """A fund's total expense ratio, and its listings: exchange -> (ticker, currency)."""
    page = html.unescape(get(url).decode("utf-8", "replace"))
    ratio = re.search(r'"formattedValue":"([\d.]+)%","label":"Total Expense Ratio"', page)
    if ratio is None:
        ratio = re.search(r'"name":"Expense Ratio:","value":"([\d.]+)"', page)
    listings: dict[str, tuple[str, str]] = {}
    marker = page.find('"fullName":"listings.BBEQTICK"')
    if marker >= 0:
        start = page.rfind("dataPointsByNameMap", 0, marker)
        block = page[start : start + 30000]

        def column(name: str) -> list[str]:
            found = re.search(rf'"{name}":\{{"name":"{name}".*?"formattedValue":(\[[^\]]*\])', block)
            return json.loads(found[1]) if found else []

        for exchange, ticker, currency in zip(column("exchange"), column("ticker"), column("currencyCode")):
            listings.setdefault(exchange, (ticker, currency))
    return Decimal(ratio[1]), listings


def vanguard_voo() -> Decimal:
    profile = json.loads(get("https://investor.vanguard.com/vmf/api/VOO/profile"))["fundProfile"]
    return Decimal(profile["expenseRatio"]).normalize()


def ssga_spy() -> Decimal:
    page = html.unescape(get("https://www.ssga.com/us/en/intermediary/etfs/spdr-sp-500-etf-trust-spy").decode())
    return Decimal(re.search(r'Gross Expense Ratio","value":"([\d.]+)%"', page)[1])


# ─────────────────────────── Rust ───────────────────────────


def percent(value: Decimal | None) -> str:
    return "None" if value is None else f"Some(Percent(dec!({value.normalize():f})))"


def rust_date(value: datetime.date) -> str:
    # As rustfmt writes the `time` crate's macro.
    return f"date!({value.year} - {value.month:02d} - {value.day:02d})"


def block() -> str:
    today = datetime.date.today()
    listed = entities()
    series = bonds(listed)
    every_fund = maya_funds()
    index_funds = sorted((f for f in every_fund if is_sp500_index_fund(f)), key=lambda f: -(f["assetValue"] or 0))
    money_market = sorted(
        (f for f in every_fund if (f.get("classification") or {}).get("main") == "כספית שקלית"),
        key=lambda f: -(f["assetValue"] or 0),
    )
    etfs = tel_aviv_etfs(listed)
    foreign = foreign_in_tel_aviv()
    cspx_fee, cspx_listings = ishares(
        "https://www.ishares.com/uk/individual/en/products/253743/"
        "ishares-sp-500-b-ucits-etf-acc-fund?switchLocale=y&siteEntryPassthrough=true"
    )
    ivv_fee, _ = ishares("https://www.ishares.com/us/products/239726/ishares-core-sp-500-etf")
    voo_fee, spy_fee = vanguard_voo(), ssga_spy()
    amsterdam = cspx_listings.get("Euronext Amsterdam")
    if amsterdam != ("CSPX", "EUR"):
        sys.exit(f"CSPX in Amsterdam is now {amsterdam}")

    lines = [BEGIN, f"const CHECKED: Date = {rust_date(today)};", ""]
    lines.append("/// Every series, by the day it's repaid.")
    lines.append("const GOVERNMENT_BONDS: &[BondSeries] = &[")
    for issuer, name, number, linkage, repaid in series:
        if issuer != MAKAM:
            lines.append(
                f'    BondSeries {{ name: "{name}", number: SecurityNumber({number}), '
                f"linkage: Linkage::{linkage}, repaid: {rust_date(repaid)} }},"
            )
    lines += ["];", "", "/// Every bill, by the day it's repaid.", "const MAKAM_SERIES: &[BondSeries] = &["]
    for issuer, name, number, linkage, repaid in series:
        if issuer == MAKAM:
            lines.append(
                f'    BondSeries {{ name: "{name}", number: SecurityNumber({number}), '
                f"linkage: Linkage::{linkage}, repaid: {rust_date(repaid)} }},"
            )
    lines += ["];", "", "/// Largest first, in each group, by what its source reports.", "const FUNDS: &[Example] = &["]

    def example(name, find_by, product, venue, fee_, variable, source):
        lines.append(
            f'    Example {{ name: "{name}", find_by: {find_by}, product: Product::{product}, '
            f"venue: Venue::{venue}, fee: {percent(fee_)}, variable_fee_up_to: {percent(variable)}, "
            f"source: Source::{source} }},"
        )

    for fund in index_funds:
        example(clean(fund["name"]), f"FindBy::Number(SecurityNumber({fund['fundId']}))", "IsraeliIndexFund", "TelAviv", fee(fund), fund.get("variableFee") and decimal(fund["variableFee"]), "Maya")
    for name, number, fixed, variable, _ in etfs:
        example(name, f"FindBy::Number(SecurityNumber({number}))", "IsraeliEtf", "TelAviv", fixed, variable, "Funder")
    for name, number, isin, _ in foreign:
        # iShares' fee is the same fund's as CSPX's (one ISIN); Invesco's page
        # doesn't answer a script, so its fee is left out rather than guessed.
        known = cspx_fee if isin == "IE00B5BMR087" else None
        example(name, f"FindBy::Number(SecurityNumber({number}))", "ForeignEtfInTelAviv", "TelAviv", known, None, "TelAvivExchange")
    # The US ones by net assets as their issuers report them (October 2026):
    # VOO's share class $1.0 trillion, IVV $880 billion, SPY $810 billion.
    example("Vanguard S&P 500 ETF", 'FindBy::Ticker("VOO")', "UsFund", "NyseArca", voo_fee, None, "Vanguard")
    example("iShares Core S&P 500 ETF", 'FindBy::Ticker("IVV")', "UsFund", "NyseArca", ivv_fee, None, "IsharesUs")
    example("SPDR S&P 500 ETF Trust", 'FindBy::Ticker("SPY")', "UsFund", "NyseArca", spy_fee, None, "Ssga")
    example("iShares Core S&P 500 UCITS ETF", 'FindBy::Ticker("CSPX")', "IrishFund", "EuronextAmsterdam", cspx_fee, None, "IsharesEurope")
    lines += ["];", "", "/// Largest first.", "const MONEY_MARKET_FUNDS: &[MoneyMarketFund] = &["]
    for fund in money_market:
        lines.append(
            f'    MoneyMarketFund {{ name: "{clean(fund["name"])}", number: SecurityNumber({fund["fundId"]}), '
            f"fee: Percent(dec!({fee(fund).normalize():f})) }},"
        )
    lines += ["];", END]
    return "\n".join(lines)


def main() -> None:
    text = block()
    if "--write" in sys.argv:
        source = RUST.read_text()
        start, end = source.index(BEGIN), source.index(END) + len(END)
        RUST.write_text(source[:start] + text + source[end:])
        print(f"wrote {RUST}; run cargo fmt")
    else:
        print(text)


if __name__ == "__main__":
    main()
