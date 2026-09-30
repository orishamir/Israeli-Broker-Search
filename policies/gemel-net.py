"""Works out the funds' fees in `crates/core/src/funds.rs` from the Capital
Market Authority's open data (data.gov.il): what savers paid on average, as
the funds and insurers report it every month.

    python3 policies/gemel-net.py [YYYYMM]

Without a month, the latest one reported. Each track's fee is weighted by
the money in it. Left out: tracks that report no fee (new ones), funds for
one employer's or one sector's workers, and self-managed tracks (ניהול
אישי, IRA), whose savers pay a broker's trade fees on top.
"""

import json
import sys
import urllib.parse
import urllib.request
from collections import defaultdict

SEARCH = "https://data.gov.il/api/3/action/datastore_search"
# "נתוני הגמל נט לשנים 2024-היום" and "נתוני הביטוח נט לשנים 2024-היום".
GEMEL_NET = "a30dcbea-a1d2-482c-ae29-8f781f5025fb"
BITUACH_NET = "c6c62cc7-fe02-4b18-8f3e-813abfbb4647"


def records(resource: str, filters: dict) -> list[dict]:
    query = urllib.parse.urlencode(
        {
            "resource_id": resource,
            "limit": 32000,
            "filters": json.dumps(filters, ensure_ascii=False),
        }
    )
    # The site refuses the default Python user agent.
    request = urllib.request.Request(
        f"{SEARCH}?{query}", headers={"User-Agent": "datagov-external-client"}
    )
    with urllib.request.urlopen(request, timeout=120) as response:
        return json.load(response)["result"]["records"]


def weighted(tracks: list[dict], field: str) -> float:
    """The average of `field`, each track counted by its assets."""
    assets = sum(track["TOTAL_ASSETS"] for track in tracks)
    return sum((track[field] or 0) * track["TOTAL_ASSETS"] for track in tracks) / assets


def reporting_a_fee(tracks: list[dict]) -> list[dict]:
    return [
        track
        for track in tracks
        if track["AVG_ANNUAL_MANAGEMENT_FEE"] is not None and track["TOTAL_ASSETS"]
    ]


def self_managed(track: dict) -> bool:
    return "ניהול אישי" in track["FUND_NAME"] or "IRA" in track["FUND_NAME"]


def funds_of(classification: str, month: int | None) -> int:
    """Prints what savers in the funds classified `classification` paid, all
    together and by company, and returns the month reported on."""
    funds = records(GEMEL_NET, {"FUND_CLASSIFICATION": classification})
    month = month or max(fund["REPORT_PERIOD"] for fund in funds)
    funds = reporting_a_fee(
        [
            fund
            for fund in funds
            if fund["REPORT_PERIOD"] == month
            and fund["TARGET_POPULATION"] == "כלל האוכלוסיה"
            and not self_managed(fund)
        ]
    )
    by_company = defaultdict(list)
    for fund in funds:
        by_company[fund["MANAGING_CORPORATION"]].append(fund)

    print(f"{classification}, {month}: {len(funds)} tracks of {len(by_company)} companies")
    print(
        f"  all: {weighted(funds, 'AVG_ANNUAL_MANAGEMENT_FEE'):.4f}% of the balance,"
        f" {weighted(funds, 'AVG_DEPOSIT_FEE'):.4f}% of deposits"
    )
    companies = sorted(
        by_company.items(), key=lambda item: weighted(item[1], "AVG_ANNUAL_MANAGEMENT_FEE")
    )
    for company, tracks in companies:
        print(
            f"  {weighted(tracks, 'AVG_ANNUAL_MANAGEMENT_FEE'):.4f}%"
            f"  {weighted(tracks, 'AVG_DEPOSIT_FEE'):.4f}%  {company}"
        )
    return month


def main() -> None:
    month = int(sys.argv[1]) if len(sys.argv) > 1 else None
    month = funds_of("קופת גמל להשקעה", month)
    funds_of("קרנות השתלמות", month)

    # Savings policies aren't reported apart: every investment policy sold
    # since 2004, managers' insurance included.
    policies = reporting_a_fee(
        [
            policy
            for policy in records(BITUACH_NET, {"REPORT_PERIOD": month})
            if policy["FUND_CLASSIFICATION"] == "פוליסות שהונפקו החל משנת 2004"
        ]
    )
    insurers = {policy["PARENT_COMPANY_NAME"] for policy in policies}
    with_deposit_fee = [p for p in policies if p["AVG_DEPOSIT_FEE"] is not None]
    print(f"Policies sold since 2004, {month}: {len(policies)} tracks of {len(insurers)} insurers")
    print(
        f"  all: {weighted(policies, 'AVG_ANNUAL_MANAGEMENT_FEE'):.4f}% of the balance;"
        f" {weighted(with_deposit_fee, 'AVG_DEPOSIT_FEE'):.4f}% of deposits"
        f" in the {len(with_deposit_fee)} that report it"
    )


if __name__ == "__main__":
    main()
