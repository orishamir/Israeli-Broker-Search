"""Measures what holding the S&P 500 through each kind of product has cost,
for `crates/core/src/products.rs` and the funds' tracks in
`crates/core/src/funds.rs`: how far each product's own return trailed the
index with all its dividends, in shekels, over the last five years.

    python3 policies/index-tracking.py

The index is the S&P 500 Total Return (gross dividends, Yahoo's ^SP500TR),
turned into shekels at the Bank of Israel's representative rate. Each
product's gap is its yearly return less the index's over the same days.

- Israeli index funds (קרנות מחקות): every unhedged shekel S&P 500 index
  fund on Maya, at its daily price.
- ETFs listed in Tel Aviv: the six Israeli S&P 500 ETFs and the two foreign
  ones, at their closing prices on the exchange. They close while New York
  trades, so a day's price is compared with the index's average over the
  first and the last month of the five years, not one day's.
- The funds' S&P 500 tracks: Gemel Net's and Bituach Net's monthly
  returns, which are before the management fee, so the gap is the track's
  own costs alone.

Each kind is averaged by the money in it. Products younger than five years
are left out of the measuring, not of the published fees.
"""

import csv
import datetime
import html
import io
import json
import re
import sys
import time
import urllib.parse
import urllib.request
from collections import defaultdict

YEARS = 5
BROWSER = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0 Safari/537.36"

# The exchange's numbers for the S&P 500 ETFs listed in Tel Aviv, unhedged.
ISRAELI_ETFS = {
    1146471: "Kesem",
    1144385: "Tachlit",
    1149020: "Harel",
    1150333: "MTF",
    1165810: "Mor",
    1148162: "IBI",
}
# The foreign ones publish their fees in English, not on Funder: each with
# its yearly fee and the share of the dividends US tax keeps from it (an
# Irish fund that holds the shares pays 15%; one that swaps for the index,
# none).
FOREIGN_ETFS = {
    1159250: ("iShares Core S&P 500 (Irish, holds the shares)", 0.07, 15),
    1183441: ("Invesco S&P 500 (Irish, by swaps)", 0.05, 0),
}
# A product younger than five years is measured since it began, if that's
# at least this long.
SHORTEST = 3


def get(url: str, headers: dict | None = None, data: bytes | None = None) -> bytes:
    request = urllib.request.Request(
        url, data=data, headers={"User-Agent": BROWSER, **(headers or {})}
    )
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


# ─────────────────────────── The index ───────────────────────────


def yahoo_daily(symbol: str) -> dict[datetime.date, float]:
    start = int(datetime.datetime(2015, 1, 1, tzinfo=datetime.UTC).timestamp())
    url = f"https://query1.finance.yahoo.com/v8/finance/chart/{urllib.parse.quote(symbol)}?interval=1d&period1={start}&period2=9999999999"
    chart = json.loads(get(url))["chart"]["result"][0]
    closes = chart["indicators"]["quote"][0]["close"]
    return {
        datetime.datetime.fromtimestamp(t, datetime.UTC).date(): close
        for t, close in zip(chart["timestamp"], closes)
        if close is not None
    }


def representative_rates() -> dict[datetime.date, float]:
    url = (
        "https://edge.boi.org.il/FusionEdgeServer/sdmx/v2/data/dataflow/BOI.STATISTICS/EXR/1.0/"
        "RER_USD_ILS?c%5BTIME_PERIOD%5D=ge:2015-01-01&format=csv"
    )
    rows = csv.DictReader(io.StringIO(get(url).decode()))
    return {
        datetime.date.fromisoformat(row["TIME_PERIOD"]): float(row["OBS_VALUE"]) for row in rows
    }


def on_or_before(series: dict[datetime.date, float], day: datetime.date) -> float:
    while day not in series:
        day -= datetime.timedelta(days=1)
    return series[day]


class Index:
    """The S&P 500 with gross dividends, in shekels, and its dividends."""

    def __init__(self) -> None:
        self.total_return = yahoo_daily("^SP500TR")
        self.price = yahoo_daily("^GSPC")
        self.dollar = representative_rates()

    def in_shekels(self, day: datetime.date) -> float:
        return on_or_before(self.total_return, day) * on_or_before(self.dollar, day)

    def dividend_yield(self, start: datetime.date, end: datetime.date) -> float:
        """The dividends a year, in %, as the gross index's lead over the price index."""
        years = (end - start).days / 365.25
        total = on_or_before(self.total_return, end) / on_or_before(self.total_return, start)
        price = on_or_before(self.price, end) / on_or_before(self.price, start)
        return ((total / price) ** (1 / years) - 1) * 100


def window() -> tuple[datetime.date, datetime.date]:
    """The five years to the end of last month."""
    end = datetime.datetime.now(datetime.UTC).date().replace(day=1) - datetime.timedelta(days=1)
    start = end.replace(year=end.year - YEARS)
    return start, end


def yearly(ratio: float, start: datetime.date, end: datetime.date) -> float:
    return (ratio ** (365.25 / (end - start).days) - 1) * 100


def gap(
    prices: dict[datetime.date, float],
    index: Index,
    start: datetime.date,
    end: datetime.date,
    in_dollars: bool = False,
) -> float | None:
    """The product's yearly return less the index's, in percentage points,
    from the product's price over against the index's at both ends, each
    averaged over the month, so that a day's timing doesn't count. From the
    product's first full month if it's younger, and `None` if that leaves
    less than `SHORTEST` years."""
    value = (lambda d: on_or_before(index.total_return, d)) if in_dollars else index.in_shekels

    def month_average(last_day: datetime.date) -> float | None:
        days = [d for d in prices if last_day - datetime.timedelta(days=30) < d <= last_day]
        # Tel Aviv's holidays can leave a September with a few days only.
        if len(days) < 5:
            return None
        return sum(prices[d] / value(d) for d in days) / len(days)

    if not prices:
        return None
    launched = min(prices) + datetime.timedelta(days=30)
    if launched > start:
        start = launched.replace(day=1) + datetime.timedelta(days=31)
        start = start.replace(day=1) - datetime.timedelta(days=1)
    if (end - start).days < SHORTEST * 365:
        return None
    first, last = month_average(start), month_average(end)
    if first is None or last is None:
        return None
    return yearly(last / first, start, end)


def weighted(items: list[tuple[float, float]]) -> float:
    """The average of (value, weight) pairs, by weight."""
    return sum(value * weight for value, weight in items) / sum(weight for _, weight in items)


# ─────────────────────────── Index funds (Maya) ───────────────────────────

MAYA = "https://maya.tase.co.il/api/v1/funds/mutual"
MAYA_HEADERS = {
    "content-type": "application/json",
    "accept": "application/json",
    "origin": "https://maya.tase.co.il",
    "referer": "https://maya.tase.co.il/he/funds/mutual-funds",
}


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
    """An unhedged S&P 500 index fund priced in shekels: all of it on the
    index, all of it in dollars."""
    assets = {asset["name"]: asset["weight"] for asset in fund.get("underlyingAssets") or []}
    index_fund = any(f["enabled"] and "קרן מחקה" in f["title"] for f in fund["features"])
    in_dollars = any(f["enabled"] and "נקובות ב- $" in f["title"] for f in fund["features"])
    return (
        index_fund
        and not in_dollars
        and assets.get("S&P 500 - NTR") == 100
        and assets.get('דולר ארה"ב') == 100
    )


def maya_history(
    fund_id: int, start: datetime.date, end: datetime.date
) -> dict[datetime.date, float]:
    """The fund's daily price. Maya dates a price by the New York close it
    was set from."""
    prices, page = {}, 1
    while True:
        body = json.dumps(
            {
                "period": 4,
                "fromDate": f"{start}T00:00:00",
                "toDate": f"{end}T00:00:00",
                "pageNumber": page,
            }
        ).encode()
        batch = json.loads(
            get(
                f"{MAYA}/{fund_id}/history",
                MAYA_HEADERS
                | {"referer": f"https://maya.tase.co.il/he/funds/mutual-funds/{fund_id}"},
                body,
            )
        )
        for row in batch:
            prices[datetime.date.fromisoformat(row["tradeDate"][:10])] = row["sellPrice"]
        if len(batch) < 30:
            return prices
        page += 1
        time.sleep(0.15)


def index_funds(index: Index, start: datetime.date, end: datetime.date) -> None:
    funds = [fund for fund in maya_funds() if is_sp500_index_fund(fund)]
    print(f"Israeli S&P 500 index funds, unhedged, in shekels: {len(funds)}")
    published, measured = [], []
    for fund in sorted(funds, key=lambda fund: -(fund["assetValue"] or 0)):
        details = json.loads(
            get(
                f"{MAYA}/{fund['fundId']}",
                {
                    "accept": "application/json",
                    "referer": f"https://maya.tase.co.il/he/funds/mutual-funds/{fund['fundId']}",
                },
            )
        )
        fee = (details["managementFee"] or 0) + (details["trusteeFee"] or 0)
        variable = details.get("variableFee") or 0
        assets = fund["assetValue"] or 0
        prices = maya_history(fund["fundId"], start - datetime.timedelta(days=40), end)
        measured_gap = gap(prices, index, start, end)
        published.append((fee + variable, assets))
        if measured_gap is not None:
            measured.append((-measured_gap, assets))
        print(
            f"  {fund['fundId']} {fund['name']}: {fee:.3f}% + up to {variable}% variable,"
            f" ₪{assets:,.0f} million; trailed by {'-' if measured_gap is None else f'{-measured_gap:.2f}'}"
        )
    summary(published, measured)


# ─────────────────────────── ETFs (the exchange) ───────────────────────────


def tase_history(
    security: int, start: datetime.date, end: datetime.date
) -> dict[datetime.date, float]:
    """The closing price, adjusted for distributions: each day's change from
    its base price, chained."""
    rows, page = [], 1
    while True:
        body = json.dumps(
            {
                "dFrom": str(start),
                "dTo": str(end),
                "oId": str(security),
                "pageNum": page,
                "pType": "8",
                "TotalRec": 1,
                "lang": "0",
            }
        ).encode()
        answer = json.loads(
            get(
                "https://api.tase.co.il/api/security/historyeod",
                {"Content-Type": "application/json", "Referer": "https://market.tase.co.il/"},
                body,
            )
        )
        items = answer.get("Items") or []
        rows += items
        if not items or len(rows) >= answer.get("TotalRec", 0):
            break
        page += 1
        time.sleep(0.2)
    rows.sort(key=lambda row: tase_date(row["TradeDate"]))
    prices, value = {}, 1.0
    for row in rows:
        # A day without trading has no prices.
        if not row["CloseRate"] or not row["BaseRate"]:
            continue
        value *= row["CloseRate"] / row["BaseRate"]
        prices[tase_date(row["TradeDate"])] = value
    return prices


def tase_date(text: str) -> datetime.date:
    """The exchange's 30/09/2026."""
    day, month, year = map(int, text.split("/"))
    return datetime.date(year, month, day)


def funder_fees(security: int) -> tuple[float, float, float] | None:
    """The fixed, variable and trustee's fees, in %, as Funder lists them."""
    page = get(
        f"https://www.funder.co.il/etf/{security}", {"Accept-Language": "he-IL,he;q=0.9"}
    ).decode("utf-8", "ignore")
    text = html.unescape(
        re.sub(
            r"<[^>]+>",
            " ",
            re.sub(r"<script.*?</script>|<style.*?</style>", " ", page, flags=re.DOTALL),
        )
    )
    text = " ".join(text.split())
    found = re.search(r"דמי ניהול דמי ניהול משתנים דמי נאמנות ([\d.]+)% ([\d.]+)% ([\d.]+)%", text)
    return tuple(float(x) for x in found.groups()) if found else None


def tase_market_value(security: int) -> float:
    """What's held in Tel Aviv, in ₪ million."""
    data = json.loads(
        get(
            f"https://api.tase.co.il/api/company/securitydata?securityId={security}&lang=0",
            {"Content-Type": "application/json", "Referer": "https://market.tase.co.il/"},
        )
    )
    return data["MarketValue"] / 1000


def summary(published: list[tuple[float, float]], measured: list[tuple[float, float]]) -> None:
    """The kind's published cost and gap, each averaged by assets, and the
    larger of the two, which the app counts."""
    cost = weighted(published)
    trailed = weighted(measured) if measured else None
    counted = cost if trailed is None else max(cost, trailed)
    print(
        f"  by assets: published {cost:.3f}%, trailed {'-' if trailed is None else f'{trailed:.3f}%'}"
        f" → counted {counted:.2f}%"
    )


def israeli_etfs(index: Index, start: datetime.date, end: datetime.date) -> None:
    print("Israeli S&P 500 ETFs in Tel Aviv:")
    published, measured = [], []
    for security, name in ISRAELI_ETFS.items():
        fees = funder_fees(security)
        if fees is None:
            sys.exit(f"Funder's page for {security} has changed")
        assets = tase_market_value(security)
        measured_gap = gap(
            tase_history(security, start - datetime.timedelta(days=40), end), index, start, end
        )
        published.append((sum(fees), assets))
        if measured_gap is not None:
            measured.append((-measured_gap, assets))
        print(
            f"  {security} {name}: {fees[0]}% + up to {fees[1]}% variable + {fees[2]}% for the trustee,"
            f" ₪{assets:,.0f} million; trailed by {'-' if measured_gap is None else f'{-measured_gap:.2f}'}"
        )
    summary(published, measured)


def foreign_etfs(index: Index, start: datetime.date, end: datetime.date) -> None:
    print("Foreign S&P 500 ETFs listed in Tel Aviv:")
    published, measured = [], []
    for security, (name, fee, dividend_tax) in FOREIGN_ETFS.items():
        assets = tase_market_value(security)
        measured_gap = gap(
            tase_history(security, start - datetime.timedelta(days=40), end), index, start, end
        )
        cost = fee + dividend_tax / 100 * index.dividend_yield(start, end)
        published.append((cost, assets))
        if measured_gap is not None:
            measured.append((-measured_gap, assets))
        print(
            f"  {security} {name}: {fee}% + {dividend_tax}% of the dividends = {cost:.3f}%,"
            f" ₪{assets:,.0f} million; trailed by {'-' if measured_gap is None else f'{-measured_gap:.2f}'}"
        )
    summary(published, measured)


def irish_in_london(index: Index, start: datetime.date, end: datetime.date) -> None:
    """iShares' Irish fund as it trades in London, in dollars: what a
    European exchange sells."""
    measured_gap = gap(yahoo_daily("CSPX.L"), index, start, end, in_dollars=True)
    cost = 0.07 + 0.15 * index.dividend_yield(start, end)
    print(
        f"iShares Core S&P 500 in London (CSPX): 0.07% + 15% of the dividends = {cost:.3f}%;"
        f" trailed by {'-' if measured_gap is None else f'{-measured_gap:.2f}'}"
    )


# ─────────────────────────── The funds' tracks ───────────────────────────

SEARCH = "https://data.gov.il/api/3/action/datastore_search"
GEMEL_NET = [
    "a30dcbea-a1d2-482c-ae29-8f781f5025fb",
    "2016d770-f094-4a2e-983e-797c26479720",
    "91c849ed-ddc4-472b-bd09-0f5486cea35c",
]
BITUACH_NET = [
    "c6c62cc7-fe02-4b18-8f3e-813abfbb4647",
    "672090ba-7893-4496-a07c-dc7e822cbf18",
    "584e6b69-174f-46c9-b8db-03925b4c68c6",
]


def records(resource: str, filters: dict) -> list[dict]:
    out, offset = [], 0
    while True:
        query = urllib.parse.urlencode(
            {
                "resource_id": resource,
                "limit": 32000,
                "offset": offset,
                "filters": json.dumps(filters, ensure_ascii=False),
            }
        )
        # The site refuses other user agents.
        request = urllib.request.Request(
            f"{SEARCH}?{query}", headers={"User-Agent": "datagov-external-client"}
        )
        with urllib.request.urlopen(request, timeout=300) as response:
            batch = json.load(response)["result"]["records"]
        out += batch
        if len(batch) < 32000:
            return out
        offset += 32000


def month_end(period: int) -> datetime.date:
    year, month = divmod(period, 100)
    return datetime.date(year + month // 12, month % 12 + 1, 1) - datetime.timedelta(days=1)


def is_sp500_track(track: dict) -> bool:
    name = track["FUND_NAME"] or ""
    return (track.get("SUB_SPECIALIZATION") or "").lower() == "עוקב מדד s&p 500" or (
        "S&P" in name.upper() and "500" in name and "עוקב" in name
    )


def tracks(title: str, monthly: list[dict], index: Index, end: datetime.date) -> None:
    """Each S&P 500 track's gap before its fee, over the five years to the
    regulator's last report (or since it began, if that's long enough), and
    what its savers paid."""
    by_track = defaultdict(dict)
    latest = {}
    for row in monthly:
        if row["MONTHLY_YIELD"] is None:
            continue
        by_track[row["FUND_ID"]][row["REPORT_PERIOD"]] = row["MONTHLY_YIELD"]
        if (
            row["FUND_ID"] not in latest
            or row["REPORT_PERIOD"] > latest[row["FUND_ID"]]["REPORT_PERIOD"]
        ):
            latest[row["FUND_ID"]] = row
    # The regulator reports a month late: the five years end with its last report.
    last_period = max(
        row["REPORT_PERIOD"]
        for row in monthly
        if row["REPORT_PERIOD"] <= end.year * 100 + end.month
    )
    end = month_end(last_period)
    first_period = last_period - YEARS * 100
    print(f"{title} the five years to {end}")
    measured, fees = [], []
    for track_id, row in sorted(latest.items(), key=lambda item: -(item[1]["TOTAL_ASSETS"] or 0)):
        if row["REPORT_PERIOD"] != last_period or not row["TOTAL_ASSETS"]:
            continue
        if row.get("TARGET_POPULATION") not in (None, "כלל האוכלוסיה"):
            continue
        months = [p for p in sorted(by_track[track_id]) if first_period < p <= last_period]
        value = 1.0
        for period in months:
            value *= 1 + by_track[track_id][period] / 100
        track_gap = None
        # A younger track is measured since it began, if that's long enough.
        if len(months) >= SHORTEST * 12:
            began = month_end(months[0] - 89 if months[0] % 100 == 1 else months[0] - 1)
            the_index = index.in_shekels(end) / index.in_shekels(began)
            track_gap = yearly(value, began, end) - yearly(the_index, began, end)
            measured.append((-track_gap, row["TOTAL_ASSETS"]))
        if row["AVG_ANNUAL_MANAGEMENT_FEE"] is not None:
            fees.append(
                (
                    row["AVG_ANNUAL_MANAGEMENT_FEE"],
                    row["TOTAL_ASSETS"],
                    row.get("AVG_DEPOSIT_FEE") or 0,
                )
            )
        print(
            f"  {track_id} {row['FUND_NAME']}: fee {row['AVG_ANNUAL_MANAGEMENT_FEE']}% + {row.get('AVG_DEPOSIT_FEE')}%"
            f" of deposits, ₪{row['TOTAL_ASSETS']:,.0f} million;"
            f" trailed by {'-' if track_gap is None else f'{-track_gap:.2f}'} before the fee"
        )
    if measured:
        print(f"  by assets: trailed {weighted(measured):.3f}% before the fee", end="")
    if fees:
        print(
            f"; fee {weighted([(fee, assets) for fee, assets, _ in fees]):.3f}%"
            f" + {weighted([(deposit, assets) for _, assets, deposit in fees]):.3f}% of deposits"
        )
        by_company = defaultdict(list)
        for row in latest.values():
            if (
                row["REPORT_PERIOD"] == last_period
                and row["TOTAL_ASSETS"]
                and row["AVG_ANNUAL_MANAGEMENT_FEE"] is not None
                and row.get("TARGET_POPULATION") in (None, "כלל האוכלוסיה")
            ):
                company = row.get("MANAGING_CORPORATION") or row.get("PARENT_COMPANY_NAME")
                by_company[company].append((row["AVG_ANNUAL_MANAGEMENT_FEE"], row["TOTAL_ASSETS"]))
        ranked = sorted((weighted(rows), company) for company, rows in by_company.items())
        print(
            f"  cheapest company {ranked[0][1]} {ranked[0][0]:.3f}%, dearest {ranked[-1][1]} {ranked[-1][0]:.3f}%"
        )


def main() -> None:
    start, end = window()
    index = Index()
    print(
        f"From {start} to {end}: the S&P 500 with dividends made"
        f" {yearly(index.in_shekels(end) / index.in_shekels(start), start, end):.2f}% a year in shekels;"
        f" dividends {index.dividend_yield(start, end):.2f}% a year"
    )
    index_funds(index, start, end)
    israeli_etfs(index, start, end)
    foreign_etfs(index, start, end)
    irish_in_london(index, start, end)
    for classification, title in [
        ("קרנות השתלמות", "Study funds' S&P 500 tracks:"),
        ("קופת גמל להשקעה", "Provident funds for investment, S&P 500 tracks:"),
    ]:
        rows = [
            row
            for resource in GEMEL_NET
            for row in records(resource, {"FUND_CLASSIFICATION": classification})
        ]
        tracks(title, sp500_tracks(rows), index, end)
    rows = [
        row
        for resource in BITUACH_NET
        for row in records(resource, {"FUND_CLASSIFICATION": "פוליסות שהונפקו החל משנת 2004"})
    ]
    tracks("Savings policies, S&P 500 tracks:", sp500_tracks(rows), index, end)


def sp500_tracks(rows: list[dict]) -> list[dict]:
    """Every month of the tracks that follow the S&P 500 now: older reports
    name and classify a track differently, so it's found by its number."""
    latest = max(row["REPORT_PERIOD"] for row in rows)
    ids = {row["FUND_ID"] for row in rows if row["REPORT_PERIOD"] == latest and is_sp500_track(row)}
    return [row for row in rows if row["FUND_ID"] in ids]


if __name__ == "__main__":
    sys.exit(main())
