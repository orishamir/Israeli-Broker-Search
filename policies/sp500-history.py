"""Prints what the S&P 500 did over every rolling stretch of 1 to 30 years,
as `crates/core/src/history/sp500.rs`, from Robert Shiller's monthly data.

    uv run --with xlrd policies/sp500-history.py [--from 1926-01] [ie_data.xls] \
        > crates/core/src/history/sp500.rs && cargo fmt
    uv run --with xlrd policies/sp500-history.py --months [ie_data.xls] \
        > crates/core/src/history/sp500_months.rs && cargo fmt

With `--months` it prints the total return index itself, month by month,
instead of the table: the guide runs the saver's own deposits through every
stretch of it (money that comes in every month meets each stretch at many
prices, which a yearly rate of one sum doesn't say).

Without a file it downloads http://www.econ.yale.edu/~shiller/data/ie_data.xls
(the page that lists the data, http://www.econ.yale.edu/~shiller/data.htm,
now links only the newer chapter files, but this one is still served; the
copy there was last saved in September 2023, so the history ends there).
Read a downloaded file with `python3 -I`-style care: it is only data.

How the index is built. The sheet gives, per month, the S&P composite's
price (the month's average of daily closes, not the last day's) and the
year's dividends at an annual rate. The total return index grows each month
by (price + dividend / 12) / last month's price: the month's share of the
dividend is counted as reinvested at once. This approximates the total
return index; it is in nominal dollars, before any fund's fee or tax. The
sheet has no dividend for its last three months, so the history ends at the
last month that has one.

Why 1926. It is the usual start of long US stock return series (the CRSP
data that Ibbotson's and most textbooks' tables begin there), and the years
before it are thinner and less like the index someone buys now. The data
begins in 1871; `--from 1871-01` uses all of it.

For each stretch of N whole years (N = 1..30) the script takes every window
that starts in any month (windows overlap, so they are far fewer separate
experiences than their number says) and works out the annualized return:
the worst, the 10th percentile, the median, the 90th percentile, the best
(percentiles by linear interpolation), and in how many windows the money
ended lower than it started (total return below 1).
"""

import argparse
import io
import sys
import urllib.request

URL = "http://www.econ.yale.edu/~shiller/data/ie_data.xls"
HORIZONS = range(1, 31)


def months(path: str | None, first: tuple[int, int]) -> list[tuple[int, int, float]]:
    """(year, month, total return index) for every month from `first` on."""
    import xlrd

    if path:
        book = xlrd.open_workbook(path)
    else:
        with urllib.request.urlopen(URL, timeout=120) as response:
            book = xlrd.open_workbook(file_contents=response.read())
    sheet = book.sheet_by_name("Data")
    out: list[tuple[int, int, float]] = []
    index = 1.0
    last_price = None
    for r in range(8, sheet.nrows):
        date, price, dividend = sheet.row_values(r)[:3]
        if not isinstance(date, float) or price == "":
            continue
        # 1871.1 is October: the sheet writes the month as the decimals.
        year, month = int(date), round((date - int(date)) * 100)
        if dividend == "":
            break
        if (year, month) < first:
            last_price = price
            continue
        if last_price is not None:
            index *= (price + dividend / 12) / last_price
            out.append((year, month, index))
        last_price = price
    return out


def percentile(sorted_values: list[float], share: float) -> float:
    at = share * (len(sorted_values) - 1)
    low = int(at)
    high = min(low + 1, len(sorted_values) - 1)
    return sorted_values[low] + (sorted_values[high] - sorted_values[low]) * (at - low)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("file", nargs="?")
    parser.add_argument("--from", dest="start", default="1926-01")
    parser.add_argument("--months", action="store_true")
    args = parser.parse_args()
    y, m = args.start.split("-")
    # The index starts at the month before `--from`'s price, so the first
    # month listed is the one after the start month.
    series = months(args.file, (int(y), int(m)))
    values = [v for _, _, v in series]
    first, last = series[0], series[-1]
    if args.months:
        print_months(series)
        return

    rows = []
    for years in HORIZONS:
        n = years * 12
        returns = []
        lower = 0
        for i in range(n, len(values)):
            ratio = values[i] / values[i - n]
            returns.append((ratio ** (1 / years) - 1) * 100)
            lower += ratio < 1
        returns.sort()
        rows.append((years, returns, lower))

    out = io.StringIO()
    w = out.write
    w("// Made by `policies/sp500-history.py`; do not edit by hand.\n")
    w(f"// Months {first[0]}-{first[1]:02} to {last[0]}-{last[1]:02} of Shiller's data.\n\n")
    w("use rust_decimal_macros::dec;\nuse time::Date;\nuse time::macros::date;\n\n")
    w("use super::Row;\n\n")
    w(f"pub(super) const FROM: Date = date!({first[0]}-{first[1]:02}-01);\n")
    w(f"pub(super) const TO: Date = date!({last[0]}-{last[1]:02}-01);\n")
    w(f"pub(super) const MONTHS: u32 = {len(values)};\n\n")
    w("/// Annualized returns in percent, one row per whole year.\n")
    w("#[allow(clippy::too_many_lines, reason = \"a table\")]\n")
    w(f"pub(super) fn rows() -> [Row; {len(rows)}] {{\n    [\n")
    for years, r, lower in rows:
        f = lambda x: f"dec!({x:.2f})"
        w(
            f"        Row {{ years: {years}, worst: {f(r[0])}, p10: {f(percentile(r, 0.1))}, "
            f"median: {f(percentile(r, 0.5))}, p90: {f(percentile(r, 0.9))}, best: {f(r[-1])}, "
            f"lower: {lower}, windows: {len(r)} }},\n"
        )
    w("    ]\n}\n")
    sys.stdout.write(out.getvalue())


def print_months(series: list[tuple[int, int, float]]) -> None:
    """The index month by month, from 1 at the month before the first."""
    out = io.StringIO()
    w = out.write
    first, last = series[0], series[-1]
    w("// Made by `policies/sp500-history.py --months`; do not edit by hand.\n")
    w(f"// Months {first[0]}-{first[1]:02} to {last[0]}-{last[1]:02} of Shiller's data.\n\n")
    w("/// The total return index at the end of each month, in nominal dollars,\n")
    w("/// from 1 at the start of the first: the same months as `sp500.rs`.\n")
    w("#[allow(clippy::unreadable_literal, reason = \"a table\")]\n")
    w(f"pub(super) const INDEX: [f64; {len(series)}] = [\n")
    for _, _, v in series:
        w(f"    {v:.10e},\n")
    w("];\n")
    sys.stdout.write(out.getvalue())


main()
