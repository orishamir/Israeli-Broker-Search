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
    let plan = |name: &str, us_stocks_and_etfs: Price| Plan {
        name: name.into(),
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
            // The tariff has no row for European stocks or ETFs.
        ],
        custody: vec![CustodyFee {
            exchanges: vec![],
            percent: dec!(0.15),
            // VERIFY: the row says "0.15%, minimum ₪75 a month" without saying
            // whether 0.15% is per month or per year. Per year is assumed.
            per: Period::Year,
            billed: Period::Month,
            min: Some(ils(dec!(75))),
        }],
        conversion: ConversionFee {
            percent: dec!(0),
            min: None,
            max: None,
            spread_percent: dec!(0.7),
        },
    };

    Broker {
        name: "Altshuler Shaham Trade".into(),
        tariff_date: "not stated in the PDF".into(),
        plans: vec![
            plan(
                "US 1¢/share",
                Price::PerShare {
                    per_share: usd(dec!(0.01)),
                    min: Some(usd(dec!(9))),
                    max: None,
                },
            ),
            plan("US $11 flat", Price::Flat(usd(dec!(11)))),
            plan(
                "US 0.15%",
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
    // Online prices (Nispach Heh). The branch prices in the main tables are higher.
    let online = Plan {
        name: "Online".into(),
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
            // Not published in the tariff.
            spread_percent: dec!(0),
        },
    };

    // Customer group from Nispach Alef. Only the parts that differ from Online.
    let plus18 = Plan {
        name: "Online, 'Leumi 18+'".into(),
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
        ..online.clone()
    };

    // Buying an index fund on Tel Aviv by standing order (tariff 4(a)(1), note
    // 10). Any Online customer can set one up; it's a plan here because a
    // plan's rows can't tell a standing order from a one-off purchase.
    let standing_order = Plan {
        name: "Online, monthly standing order".into(),
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
        ..online.clone()
    };

    let pepper = Plan {
        name: "Pepper".into(),
        trading: vec![
            // Pepper's ₪4 covers "stocks, T-bills and bonds"; index funds
            // aren't listed, so they keep the Online price.
            TradeFee {
                securities: vec![MutualFund],
                ..online.trading[0].clone()
            },
            // "₪4 per order up to ₪30,000"; the tariff doesn't say what larger orders cost.
            TradeFee {
                securities: vec![],
                exchanges: vec![Tlv],
                price: Price::Flat(ils(dec!(4))),
            },
            // "$4 per order up to $8,000"; the tariff doesn't say what larger orders cost.
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
    };

    Broker {
        name: "Bank Leumi".into(),
        tariff_date: "2026-06-29".into(),
        plans: vec![online, plus18, standing_order, pepper],
    }
}
