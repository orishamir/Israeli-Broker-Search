"""Prints the government bond yield curve in
`crates/core/src/short_term/government_bonds.rs`, from the Bank of Israel's
zero-coupon yields (עקום 0): what a shekel government bond held to maturity
earns a year, by years to maturity, unlinked (nominal) and linked to the
price index (real, before the index).

    python3 -I policies/yield-curve.py [YYYY-MM] [--write]

Without a month, the latest one published. With `--write`, the block of the
Rust file between its two `yield-curve.py` marker lines is replaced; without
it the block is printed.

The series are in the SDMX dataflow BOI.STATISTICS/ZCM, monthly averages:
`ZC_TSB_ZND_{NN}Y_MA` (NOMINAL_REAL=N) and `ZC_TSB_ZRD_{NN}Y_MA` (R), all
with DATA_TYPE=ZC_YTM. The real curve also has 20 years, and the app stops
at 15, where the nominal one does.
"""

import csv
import io
import re
import sys
import urllib.request
from pathlib import Path

URL = (
    "https://edge.boi.org.il/FusionEdgeServer/sdmx/v2/data/dataflow/"
    "BOI.STATISTICS/ZCM/1.0?format=csv"
)
YEARS = [1, 2, 3, 4, 5, 7, 10, 15]
RUST = Path(__file__).resolve().parent.parent / "crates/core/src/short_term/government_bonds.rs"
BEGIN = "// BEGIN yield-curve.py"
END = "// END yield-curve.py"


def fetch(month: str | None) -> dict[str, dict[str, float]]:
    """Every wanted series' observations: series code -> month -> yield."""
    # One observation per series is enough for the latest month; a given month
    # is asked for by its period.
    query = "&lastNObservations=1" if month is None else f"&startPeriod={month}&endPeriod={month}"
    request = urllib.request.Request(URL + query, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(request, timeout=120) as response:
        text = response.read().decode("utf-8")
    series: dict[str, dict[str, float]] = {}
    for row in csv.DictReader(io.StringIO(text)):
        code = row["SERIES_CODE"]
        if re.fullmatch(r"ZC_TSB_Z[NR]D_\d\dY_MA", code) and row["DATA_TYPE"] == "ZC_YTM":
            series.setdefault(code, {})[row["TIME_PERIOD"]] = float(row["OBS_VALUE"])
    return series


def block(series: dict[str, dict[str, float]], month: str) -> str:
    lines = [BEGIN, f"const MONTH: (i32, u8) = ({month[:4]}, {int(month[5:])});"]
    for name, code in (("NOMINAL", "ZND"), ("REAL", "ZRD")):
        what = "unlinked" if code == "ZND" else "before the index"
        lines.append(f"/// (years to maturity, % a year), {what}.")
        lines.append(f"const {name}: [(u32, Decimal); {len(YEARS)}] = [")
        for years in YEARS:
            value = series[f"ZC_TSB_{code}_{years:02d}Y_MA"][month]
            lines.append(f"    ({years}, dec!({value:.2f})),")
        lines.append("];")
    lines.append(END)
    return "\n".join(lines)


def main() -> None:
    args = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
    month = args[0] if args else None
    series = fetch(month)
    months = {m for by_month in series.values() for m in by_month}
    month = month or max(months)
    missing = [
        f"ZC_TSB_{code}_{years:02d}Y_MA"
        for code in ("ZND", "ZRD")
        for years in YEARS
        if month not in series.get(f"ZC_TSB_{code}_{years:02d}Y_MA", {})
    ]
    if missing:
        sys.exit(f"{month} lacks {missing}: the dataflow has changed or is not out yet")
    text = block(series, month)
    if "--write" in sys.argv:
        source = RUST.read_text()
        start, end = source.index(BEGIN), source.index(END) + len(END)
        RUST.write_text(source[:start] + text + source[end:])
        print(f"wrote {month} to {RUST}")
    else:
        print(text)


if __name__ == "__main__":
    main()
