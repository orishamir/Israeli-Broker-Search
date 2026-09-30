"""Prints the money market funds' fees in
`crates/core/src/short_term/money_market.rs`, from the Tel Aviv Stock
Exchange's list of mutual funds (Maya), as the fund managers report them.

    python3 policies/money-market-funds.py

Shekel money market funds only (כספית שקלית), with and without corporate
bonds. A fund costs its saver the management fee and the trustee's fee,
both taken from its assets; the distribution fee (עמלת הפצה) is paid by the
manager out of its own fee, not by the saver. The average counts each fund
by its assets. A load on buying (שיעור הוספה) shows as a purchase price
above the redemption price, and is printed beside the fund.
"""

import json
import time
import urllib.request

FUNDS = "https://maya.tase.co.il/api/v1/funds/mutual"
# Maya answers at most 30 funds a page, and turns away Python's own user agent.
HEADERS = {
    "content-type": "application/json",
    "accept": "application/json, text/plain, */*",
    "origin": "https://maya.tase.co.il",
    "referer": "https://maya.tase.co.il/he/funds/mutual-funds",
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
}


def every_fund() -> list[dict]:
    funds: list[dict] = []
    page = 1
    while True:
        body = json.dumps({"pageSize": 30, "pageNumber": page}).encode()
        request = urllib.request.Request(FUNDS, data=body, headers=HEADERS, method="POST")
        with urllib.request.urlopen(request, timeout=60) as response:
            batch = json.load(response)
        funds += batch
        if len(batch) < 30:
            return funds
        page += 1
        time.sleep(0.3)


def cost(fund: dict) -> float:
    """What the saver pays a year, in %: the manager's fee and the trustee's."""
    return (fund["managementFee"] or 0) + (fund["trusteeFee"] or 0)


def load(fund: dict) -> float:
    """The load on buying, in %: how much more a unit costs than it's redeemed for."""
    buy, redeem = fund["purchasePrice"], fund["redemptionPrice"]
    return (buy / redeem - 1) * 100 if buy and redeem else 0.0


def described(fund: dict) -> str:
    extra = f", a {load(fund):.2f}% load on buying" if load(fund) > 0.001 else ""
    return (
        f"{fund['name']} ({fund['managerName']}): {fund['managementFee']}% + "
        f"{fund['trusteeFee']}% for the trustee = {cost(fund):.3f}%{extra}; "
        f"₪{fund['assetValue']:,.0f} million"
    )


def main() -> None:
    funds = [
        fund
        for fund in every_fund()
        if (fund.get("classification") or {}).get("main") == "כספית שקלית"
    ]
    assets = sum(fund["assetValue"] for fund in funds)
    average = sum(cost(fund) * fund["assetValue"] for fund in funds) / assets
    by_cost = sorted(funds, key=cost)
    print(f"{len(funds)} shekel money market funds, ₪{assets:,.0f} million, rates of {funds[0]['ratesAsOf']}")
    print(f"Average, by assets: {average:.3f}% a year")
    print("Cheapest:")
    for fund in by_cost[:4]:
        print("  " + described(fund))
    print("Dearest:")
    for fund in by_cost[-2:]:
        print("  " + described(fund))


if __name__ == "__main__":
    main()
