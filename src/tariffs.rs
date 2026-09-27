//! Two tariffs, entered from the PDFs. Eventually this belongs in one JSON
//! file per broker (the types already serialize); it's Rust for now so the
//! compiler checks it.

use rust_decimal_macros::dec;

use crate::Exchange::*;
use crate::Security::*;
use crate::*;

pub fn altshuler() -> Broker {
    // Everything is the same across plans except the US stock/ETF row: the
    // customer picks one of three options (tariff 4(Alef)(7)(Alef)).
    let plan = |name: &str, description: &str, us_stocks_and_etfs: Price| Plan {
        name: name.into(),
        description: format!(
            "{description} Everything else (Tel Aviv, custody, conversion) is the same \
             on all three tracks."
        ),
        trading: vec![
            // ETFs and mutual funds share a row, but ETFs have a lower minimum.
            TradeFee {
                securities: vec![Etf],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: dec!(0.15),
                    min: Some(ils(dec!(3.5))),
                    max: None,
                },
            },
            TradeFee {
                securities: vec![MutualFund],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: dec!(0.15),
                    min: Some(ils(dec!(16))),
                    max: None,
                },
            },
            TradeFee {
                securities: vec![Stock, Bond],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: dec!(0.15),
                    min: Some(ils(dec!(3.5))),
                    max: None,
                },
            },
            TradeFee {
                securities: vec![Stock, Etf],
                exchanges: vec![Usa],
                price: us_stocks_and_etfs,
            },
            TradeFee {
                securities: vec![Bond, MutualFund],
                exchanges: vec![Usa, Europe],
                price: Price::Percent {
                    percent: dec!(0.3),
                    min: Some(usd(dec!(24))),
                    max: None,
                },
            },
        ],
        custody: vec![CustodyFee {
            exchanges: vec![],
            percent: dec!(0.15),
            per: Period::Year, // VERIFY: see the first note

            billed: Period::Month,
            min: Some(ils(dec!(75))),
        }],
        conversion: ConversionFee {
            percent: dec!(0),
            min: None,
            max: None,
            spread_percent: dec!(0.7),
        },
        notes: vec![],
    };

    Broker {
        name: "Altshuler Shaham Trade".into(),
        description: "An investment house's trading platform, not a bank. It has one \
                      price list, except for US stocks and ETFs, where you choose one of \
                      three commission tracks; each track is a plan here."
            .into(),
        tariff_date: "not stated in the PDF".into(),
        source_url: Some(
            "https://www.as-invest.co.il/media/oqdfpafo/\
             %D7%AA%D7%A2%D7%A8%D7%99%D7%A4%D7%95%D7%9F-%D7%9E%D7%9C%D7%90-\
             %D7%90%D7%9C%D7%98%D7%A9%D7%95%D7%9C%D7%A8-%D7%A9%D7%97%D7%9D-\
             %D7%98%D7%A8%D7%99%D7%99%D7%93.pdf"
                .into(),
        ),
        notes: notes(&[
            "Custody is listed as \"0.15%, minimum ₪75 a month\" without saying whether \
             0.15% is per year or per month. Per year is assumed; per month would be \
             1.8% a year.",
            "A \"periodic management fee\" of up to ₪80 a month is listed, without \
             saying when it applies. It isn't included.",
            "Conversion has no fee, but the exchange rate can be up to 0.7% worse than \
             the market's. The full 0.7% is assumed.",
            "No price is listed for European stocks or ETFs.",
        ]),
        plans: vec![
            plan(
                "US 1¢/share",
                "US trades cost 1¢ per share, at least $9. Cheapest when buying a few \
                 expensive shares, like an S&P 500 ETF.",
                Price::PerShare {
                    per_share: usd(dec!(0.01)),
                    min: Some(usd(dec!(9))),
                    max: None,
                },
            ),
            plan(
                "US $11 flat",
                "Every US trade costs $11, whatever its size. Cheapest for large trades.",
                Price::Flat(usd(dec!(11))),
            ),
            plan(
                "US 0.15%",
                "US trades cost 0.15% of their value, at least $9. Cheapest for small \
                 trades.",
                Price::Percent {
                    percent: dec!(0.15),
                    min: Some(usd(dec!(9))),
                    max: None,
                },
            ),
        ],
    }
}

pub fn leumi() -> Broker {
    let broker_notes = [
        "The bank's exchange-rate spread isn't published, so conversions may cost \
         more than shown.",
        "Active (non-index) mutual funds on Tel Aviv have no trade fee; every mutual \
         fund is treated as an index fund here.",
    ];

    // Online prices (Nispach Heh). The branch prices in the main tables are higher.
    let online = Plan {
        name: "Online".into(),
        description: "Leumi's standard prices for trading yourself on the website or app \
                      (the tariff's appendix on direct channels). Trading through a banker \
                      at a branch costs more."
            .into(),
        trading: vec![
            // Tariff 4(a)(1) excludes mutual funds, but its footnote puts
            // index-tracking funds (Keren Mechaka) and ETFs back in. Active
            // funds carry no trade fee, at most a distribution fee in some
            // cases (4(b)(2)); we treat every MutualFund as an index fund.
            TradeFee {
                securities: vec![],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: dec!(0.4),
                    min: Some(ils(dec!(26))),
                    max: Some(ils(dec!(6300))),
                },
            },
            TradeFee {
                securities: vec![],
                exchanges: vec![Usa, Europe],
                price: Price::Percent {
                    percent: dec!(0.3),
                    min: Some(usd(dec!(24))),
                    max: Some(usd(dec!(6750))),
                },
            },
        ],
        custody: vec![
            CustodyFee {
                exchanges: vec![Tlv],
                percent: dec!(0.15),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
            CustodyFee {
                exchanges: vec![Usa, Europe],
                percent: dec!(0.2),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
        ],
        conversion: ConversionFee {
            percent: dec!(0.16),
            min: Some(usd(dec!(5.76))),
            max: Some(usd(dec!(2400))),
            spread_percent: dec!(0), // not published
        },
        notes: vec![],
    };

    // Customer group from Nispach Alef. Only the parts that differ from Online.
    let plus18 = Plan {
        name: "Online, 'Leumi 18+'".into(),
        description: "Online prices with the 'Leumi 18+' customer-group discount: half \
                      the custody fee and half the conversion fee. It's a group for young \
                      customers; check your eligibility with the bank."
            .into(),
        custody: vec![
            CustodyFee {
                exchanges: vec![Tlv],
                percent: dec!(0.075),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
            CustodyFee {
                exchanges: vec![Usa, Europe],
                percent: dec!(0.1),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
        ],
        // 50% off the branch fee. Cheaper than Online above ~$4,500 a conversion.
        conversion: ConversionFee {
            percent: dec!(0.1),
            min: Some(usd(dec!(7.2))),
            max: Some(usd(dec!(3000))),
            spread_percent: dec!(0),
        },
        notes: notes(&[
            "Conversion is 50% off the branch fee. Whether its $3,000 maximum \
             is halved too isn't stated; it's assumed not.",
        ]),
        ..online.clone()
    };

    // Buying an index fund on Tel Aviv by standing order (tariff 4(a)(1), note
    // 10). Any Online customer can set one up; it's a plan here because a
    // plan's rows can't tell a standing order from a one-off purchase.
    let standing_order = Plan {
        name: "Online, monthly standing order".into(),
        description: "Not a separate account: an Online customer buying Tel Aviv index \
                      funds by monthly standing order, which costs 0.225% (at least ₪5) \
                      instead of 0.4% (at least ₪26). Everything else is priced as Online."
            .into(),
        trading: [
            vec![TradeFee {
                securities: vec![MutualFund],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: dec!(0.225),
                    min: Some(ils(dec!(5))),
                    max: Some(ils(dec!(6300))),
                },
            }],
            online.trading.clone(),
        ]
        .concat(),
        notes: notes(&[
            "Only buying index funds on Tel Aviv by standing order is cheaper; \
             everything else costs the same as Online.",
        ]),
        ..online.clone()
    };

    let pepper = Plan {
        name: "Pepper".into(),
        description: "Leumi's digital-bank app. Flat trade fees (₪4 in Tel Aviv, $4 abroad \
                      per order), lower custody on foreign holdings, and half-price \
                      conversion."
            .into(),
        trading: vec![
            TradeFee {
                securities: vec![MutualFund],
                ..online.trading[0].clone()
            },
            TradeFee {
                securities: vec![],
                exchanges: vec![Tlv],
                price: Price::Flat(ils(dec!(4))),
            },
            TradeFee {
                securities: vec![],
                exchanges: vec![Usa, Europe],
                price: Price::Flat(usd(dec!(4))),
            },
        ],
        custody: vec![
            online.custody[0].clone(),
            CustodyFee {
                exchanges: vec![Usa, Europe],
                percent: dec!(0.15),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
        ],
        // 50% off the branch fee (0.2%, maximum $3,000), minimum $3.
        conversion: ConversionFee {
            percent: dec!(0.1),
            min: Some(usd(dec!(3))),
            max: Some(usd(dec!(1500))),
            spread_percent: dec!(0),
        },
        notes: notes(&[
            "₪4 is only stated for orders up to ₪30,000, and $4 for orders up to \
             $8,000. Larger orders are assumed to cost the same.",
            "Pepper's ₪4 covers stocks, T-bills and bonds. Index funds on Tel Aviv \
             aren't mentioned, so they're given the Online price.",
            "Conversion is 50% off the branch fee with a $3 minimum. Its $1,500 \
             maximum (half the branch's) is assumed.",
        ]),
    };

    Broker {
        name: "Bank Leumi".into(),
        description: "One of Israel's largest banks. Its securities prices depend on how \
                      you trade (online or at a branch), on customer groups, and on the \
                      Pepper app, so it has several plans here."
            .into(),
        tariff_date: "2026-06-29".into(),
        source_url: Some(
            "https://www.bankleumi.co.il/static-files/Commissions_Leumi/AmlotYechidimL.pdf".into(),
        ),
        notes: notes(&broker_notes),
        plans: vec![online, plus18, standing_order, pepper],
    }
}

fn notes(notes: &[&str]) -> Vec<String> {
    notes.iter().map(|note| (*note).to_owned()).collect()
}
