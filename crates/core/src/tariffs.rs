//! Two tariffs, entered from the PDFs. Eventually this belongs in one JSON
//! file per broker (the types already serialize); it's Rust for now so the
//! compiler checks it.

use rust_decimal_macros::dec;
use time::macros::date;

use crate::Exchange::{Europe, Tlv, Usa};
use crate::Security::{Bond, Etf, MutualFund, Stock};
use crate::{
    Broker, Caveat, ConversionFee, CustodyFee, Markup, Percent, Period, Plan, Price, TradeFee, ils,
    usd,
};

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn altshuler() -> Broker {
    // Everything is the same across the regular tracks except the US
    // stock/ETF row: the customer picks one of three options (tariff
    // 4(Alef)(7)(Alef)).
    let track = |name: &str, description: &str, us_stocks_and_etfs: Price| Plan {
        name: name.into(),
        description: format!(
            "{description} Everything else (Tel Aviv, custody, conversion) is the same \
             on all three regular tracks."
        ),
        trading: vec![
            // ETFs and mutual funds share a row, but ETFs have a lower minimum.
            TradeFee {
                securities: vec![Etf],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: Percent(dec!(0.15)),
                    min: Some(ils(dec!(3.5))),
                    max: None,
                },
            },
            TradeFee {
                securities: vec![MutualFund],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: Percent(dec!(0.15)),
                    min: Some(ils(dec!(16))),
                    max: None,
                },
            },
            TradeFee {
                securities: vec![Stock, Bond],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: Percent(dec!(0.15)),
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
                    percent: Percent(dec!(0.3)),
                    min: Some(usd(dec!(24))),
                    max: None,
                },
            },
        ],
        custody: vec![CustodyFee {
            exchanges: vec![],
            percent: Percent(dec!(0.15)),
            per: Period::Year, // VERIFY: see the first note
            billed: Period::Month,
            min: Some(ils(dec!(75))),
        }],
        conversion: ConversionFee {
            markup: Markup::UpTo(Percent(dec!(0.7))),
            ..ConversionFee::default()
        },
        min_first_deposit: None,
        caveats: vec![
            Caveat::new(
                "Custody is listed as \"0.15%, minimum ₪75 a month\" without saying \
                 whether 0.15% is per year or per month. Per year is assumed; per month \
                 would be 1.8% a year.",
            ),
            Caveat::new(
                "A \"periodic management fee\" of up to ₪80 a month is listed, without \
                 saying when it applies. It isn't included.",
            ),
        ],
    };

    // The offer for new customers (trading_benefits page): cheaper Tel Aviv
    // and US trades, no custody. Anything it doesn't mention is priced as the
    // regular tariff, which its fine print refers to.
    let regular = track(
        "",
        "",
        Price::PerShare {
            per_share: usd(dec!(0.01)),
            min: Some(usd(dec!(6))),
            max: None,
        },
    );
    let new_customers = Plan {
        name: "New customers".into(),
        description: "Altshuler's offer for opening a new account with at least ₪5,000: no \
                      custody fee, with no end date; Tel Aviv stocks, ETFs and index funds \
                      for 0.07% (at least ₪2.90); and US stocks and ETFs for 1¢ a share \
                      (at least $6)."
            .into(),
        trading: [
            vec![TradeFee {
                // The offer names continuously-traded ETFs (במסלול רציף), which
                // is how Tel Aviv ETFs trade.
                securities: vec![Stock, Etf, MutualFund],
                exchanges: vec![Tlv],
                price: Price::Percent {
                    percent: Percent(dec!(0.07)),
                    min: Some(ils(dec!(2.9))),
                    max: None,
                },
            }],
            regular.trading.clone(),
        ]
        .concat(),
        custody: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            Caveat::new(
                "The 0.07% is for index funds (קרנות\u{a0}מחקות). Active and money-market \
                 funds cost nothing to trade; every mutual fund is treated as an index \
                 fund here.",
            )
            .about(&[MutualFund])
            .on(&[Tlv]),
            Caveat::new(
                "The offer doesn't mention bonds, so the regular tariff's prices are \
                 assumed.",
            )
            .about(&[Bond]),
            Caveat::new(
                "The offer doesn't mention mutual funds abroad, so the regular tariff's \
                 prices are assumed.",
            )
            .about(&[MutualFund])
            .on(&[Usa, Europe]),
            Caveat::new("The ₪200 gift for opening an account isn't included."),
        ],
        ..regular
    };

    Broker {
        name: "Altshuler Shaham Trade".into(),
        description: "An investment house's trading platform, not a bank. It has one \
                      price list, except for US stocks and ETFs, where you choose one of \
                      three commission tracks; each track is a plan here. New customers \
                      get a cheaper offer instead."
            .into(),
        tariff_date: None, // not stated in the PDF
        source_url: Some(
            "https://www.as-invest.co.il/media/oqdfpafo/\
             %D7%AA%D7%A2%D7%A8%D7%99%D7%A4%D7%95%D7%9F-%D7%9E%D7%9C%D7%90-\
             %D7%90%D7%9C%D7%98%D7%A9%D7%95%D7%9C%D7%A8-%D7%A9%D7%97%D7%9D-\
             %D7%98%D7%A8%D7%99%D7%99%D7%93.pdf"
                .into(),
        ),
        caveats: vec![
            Caveat::new("The conversion markup is stated as up to 0.7%; the full 0.7% is assumed.")
                .on(&[Usa, Europe]),
            Caveat::new("No price is listed for European stocks or ETFs.")
                .about(&[Stock, Etf])
                .on(&[Europe]),
        ],
        plans: vec![
            track(
                "US 1¢/share",
                "US trades cost 1¢ per share, at least $9. Cheapest when buying a few \
                 expensive shares, like an S&P 500 ETF.",
                Price::PerShare {
                    per_share: usd(dec!(0.01)),
                    min: Some(usd(dec!(9))),
                    max: None,
                },
            ),
            track(
                "US $11 flat",
                "Every US trade costs $11, whatever its size. Cheapest for large trades.",
                Price::Flat(usd(dec!(11))),
            ),
            track(
                "US 0.15%",
                "US trades cost 0.15% of their value, at least $9. Cheapest for small \
                 trades.",
                Price::Percent {
                    percent: Percent(dec!(0.15)),
                    min: Some(usd(dec!(9))),
                    max: None,
                },
            ),
            new_customers,
        ],
    }
}

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn leumi() -> Broker {
    let broker_caveats = vec![
        Caveat::new(
            "The bank doesn't publish its conversion markup, so it's counted as 0: \
             conversions may cost more than shown.",
        )
        .on(&[Usa, Europe]),
        Caveat::new(
            "Active (non-index) mutual funds on Tel Aviv have no trade fee; every mutual \
             fund is treated as an index fund here.",
        )
        .about(&[MutualFund])
        .on(&[Tlv]),
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
                    percent: Percent(dec!(0.4)),
                    min: Some(ils(dec!(26))),
                    max: Some(ils(dec!(6300))),
                },
            },
            TradeFee {
                securities: vec![],
                exchanges: vec![Usa, Europe],
                price: Price::Percent {
                    percent: Percent(dec!(0.3)),
                    min: Some(usd(dec!(24))),
                    max: Some(usd(dec!(6750))),
                },
            },
        ],
        custody: vec![
            CustodyFee {
                exchanges: vec![Tlv],
                percent: Percent(dec!(0.15)),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
            CustodyFee {
                exchanges: vec![Usa, Europe],
                percent: Percent(dec!(0.2)),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
        ],
        conversion: ConversionFee {
            percent: Percent(dec!(0.16)),
            min: Some(usd(dec!(5.76))),
            max: Some(usd(dec!(2400))),
            markup: Markup::NotPublished,
        },
        min_first_deposit: None,
        caveats: vec![],
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
                percent: Percent(dec!(0.075)),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
            CustodyFee {
                exchanges: vec![Usa, Europe],
                percent: Percent(dec!(0.1)),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
        ],
        // 50% off the branch fee. Cheaper than Online above ~$4,500 a conversion.
        conversion: ConversionFee {
            percent: Percent(dec!(0.1)),
            min: Some(usd(dec!(7.2))),
            max: Some(usd(dec!(3000))),
            markup: Markup::NotPublished,
        },
        min_first_deposit: None,
        caveats: vec![
            Caveat::new(
                "Conversion is 50% off the branch fee. Whether its $3,000 maximum is \
                 halved too isn't stated; it's assumed not.",
            )
            .on(&[Usa, Europe]),
        ],
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
                    percent: Percent(dec!(0.225)),
                    min: Some(ils(dec!(5))),
                    max: Some(ils(dec!(6300))),
                },
            }],
            online.trading.clone(),
        ]
        .concat(),
        caveats: vec![Caveat::new(
            "Only buying index funds on Tel Aviv by standing order is cheaper; \
             everything else costs the same as Online.",
        )],
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
                percent: Percent(dec!(0.15)),
                per: Period::Quarter,
                billed: Period::Quarter,
                min: None,
            },
        ],
        // 50% off the branch fee (0.2%, maximum $3,000), minimum $3.
        conversion: ConversionFee {
            percent: Percent(dec!(0.1)),
            min: Some(usd(dec!(3))),
            max: Some(usd(dec!(1500))),
            markup: Markup::NotPublished,
        },
        min_first_deposit: None,
        caveats: vec![
            Caveat::new(
                "₪4 is only stated for orders up to ₪30,000. Larger orders are assumed to \
                 cost the same.",
            )
            .on(&[Tlv]),
            Caveat::new(
                "$4 is only stated for orders up to $8,000. Larger orders are assumed to \
                 cost the same.",
            )
            .on(&[Usa, Europe]),
            Caveat::new(
                "Pepper's ₪4 covers stocks, T-bills and bonds. Index funds on Tel Aviv \
                 aren't mentioned, so they're given the Online price.",
            )
            .about(&[MutualFund])
            .on(&[Tlv]),
            Caveat::new(
                "Conversion is 50% off the branch fee with a $3 minimum. Its $1,500 \
                 maximum (half the branch's) is assumed.",
            )
            .on(&[Usa, Europe]),
        ],
    };

    Broker {
        name: "Bank Leumi".into(),
        description: "One of Israel's largest banks. Its securities prices depend on how \
                      you trade (online or at a branch), on customer groups, and on the \
                      Pepper app, so it has several plans here."
            .into(),
        tariff_date: Some(date!(2026 - 06 - 29)),
        source_url: Some(
            "https://www.bankleumi.co.il/static-files/Commissions_Leumi/AmlotYechidimL.pdf".into(),
        ),
        caveats: broker_caveats,
        plans: vec![online, plus18, standing_order, pepper],
    }
}
