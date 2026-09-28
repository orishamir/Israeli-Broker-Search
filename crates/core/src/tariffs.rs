//! The brokers' tariffs, entered from the documents in `policies` (see
//! `policies/sources.md` for where each number comes from). Eventually this
//! belongs in one JSON file per broker (the types already serialize); it's
//! Rust for now so the compiler checks it.
//!
//! Investment houses publish a full tariff, the most they may charge, and
//! offer new customers far less. Where that offer isn't published, a plan
//! named "Typical offer" holds what comparison sites list for joining, and
//! the full tariff wherever the offer doesn't say.
//!
//! Every plan spells out all its fields: nothing here comes from a default,
//! since a defaulted fee would read as free. Every caveat is made with the
//! constructor that says how sure it is ([`Caveat::published`],
//! [`Caveat::reading`], [`Caveat::at_most`], [`Caveat::may_cost_more`],
//! [`Caveat::not_counted`]), and names the fee it's about where it's about
//! one.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::Date;
use time::macros::date;

use crate::Exchange::{self, Europe, Tlv, Usa};
use crate::Security::{self, Bond, Etf, MutualFund, Stock};
use crate::{
    Broker, Caveat, ConversionFee, CustodyFee, FeeKind, HandlingFee, Markup, Money, Percent,
    PercentFee, Period, Plan, Price, TariffDate, Track, TradeFee, ils, iso, usd,
};

/// Every broker the app knows, in the order to offer them.
#[must_use]
pub fn all() -> Vec<Broker> {
    vec![
        altshuler(),
        leumi(),
        excellence(),
        ibi(),
        interactive(),
        meitav(),
    ]
}

/// When the numbers were last checked against the brokers' documents and
/// sites.
#[must_use]
pub fn checked() -> Date {
    date!(2026 - 09 - 28)
}

// ─────────────────────────── Helpers ───────────────────────────

/// A trade row: `price` for `securities` on `exchanges` (empty means all).
fn trade(securities: &[Security], exchanges: &[Exchange], price: Price) -> TradeFee {
    TradeFee {
        securities: securities.to_vec(),
        exchanges: exchanges.to_vec(),
        price,
    }
}

/// "`percent`%, min `min`"
fn percent(percent: Decimal, min: Option<Money>) -> Price {
    Price::Percent {
        percent: Percent(percent),
        min,
        max: None,
    }
}

/// "`per_share` per share, min `min`"
fn per_share(per_share: Money, min: Money) -> Price {
    Price::PerShare {
        per_share,
        min: Some(min),
        max: None,
    }
}

/// Custody on everything: `percent` a `per`, charged every `billed`.
fn custody(percent: Decimal, per: Period, billed: Period, min: Option<Money>) -> CustodyFee {
    CustodyFee {
        securities: vec![],
        exchanges: vec![],
        percent: Percent(percent),
        per,
        billed,
        min,
    }
}

/// A handling fee of `per_month` shekels, free for `free_months`.
fn handling(per_month: Decimal, free_months: u32) -> HandlingFee {
    HandlingFee {
        per_month: ils(per_month),
        free_months,
        less_trade_fees: false,
    }
}

fn eur(amount: Decimal) -> Money {
    Money::from_decimal(amount, iso::EUR)
}

/// No conversion fee, only a markup.
fn markup_only(markup: Markup) -> ConversionFee {
    ConversionFee {
        fee: PercentFee::FREE,
        or_if_less: None,
        markup,
    }
}

/// The caveat every plan whose markup is "up to `percent`" carries.
fn markup_up_to(percent: Decimal) -> Caveat {
    let percent = Percent(percent);
    Caveat::at_most(&format!(
        "The conversion markup is stated as up to {percent}; the full {percent} is assumed."
    ))
    .about_fee(FeeKind::Markup)
    .on(&[Usa, Europe])
}

/// What a typical-offer plan's caveats begin with: where its numbers come
/// from, and why they can be trusted.
fn typical_offer_caveat(broker: &str, sites: &str) -> Caveat {
    Caveat::reading(
        &format!(
            "Not published by {broker}: the terms comparison sites list for joining \
             ({sites}, September 2026). Ask for them when you open the account."
        ),
        "the Tel Aviv Stock Exchange's actual average fees for June 2026 match them: \
         0.07%–0.085% on Tel Aviv stocks, against full tariffs of 0.15%–0.4%",
    )
}

/// Why a typical offer prices some securities as the full tariff does.
fn not_in_the_offer(text: &str) -> Caveat {
    Caveat::at_most(text).about_fee(FeeKind::Trade)
}

/// Why a typical offer charges no custody, on what the comparison sites say.
fn no_custody_in_the_offer(text: &str) -> Caveat {
    Caveat::reading(
        text,
        "the comparison sites list a holding cost wherever there is one (\u{201c}plus \
         custody\u{201d} for the banks), and the handling fee is the only one they list \
         for the offer",
    )
    .about_fee(FeeKind::Custody)
}

// ─────────────────────────── Altshuler Shaham Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn altshuler() -> Broker {
    let full_tariff = Plan {
        name: "Full tariff".into(),
        description: "Altshuler's published price list. For US stocks and ETFs you choose \
                      one of three tracks when opening the account; the app uses the \
                      cheapest for your inputs."
            .into(),
        trading: vec![
            // ETFs and mutual funds share a row, but ETFs have a lower minimum.
            trade(&[Etf], &[Tlv], percent(dec!(0.15), Some(ils(dec!(3.5))))),
            trade(
                &[MutualFund],
                &[Tlv],
                percent(dec!(0.15), Some(ils(dec!(16)))),
            ),
            trade(
                &[Stock, Bond],
                &[Tlv],
                percent(dec!(0.15), Some(ils(dec!(3.5)))),
            ),
            trade(
                &[Bond, MutualFund],
                &[Usa, Europe],
                percent(dec!(0.3), Some(usd(dec!(24)))),
            ),
        ],
        // Tariff 4(a)(7)(a): US stocks and ETFs on one of three tracks.
        tracks: vec![
            Track {
                name: "1¢ a share".into(),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(9))),
                )],
            },
            Track {
                name: "$11 per order".into(),
                trading: vec![trade(&[Stock, Etf], &[Usa], Price::Flat(usd(dec!(11))))],
            },
            Track {
                name: "0.15%".into(),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    percent(dec!(0.15), Some(usd(dec!(9)))),
                )],
            },
        ],
        standing_orders: vec![],
        standing_order_conversion: None,
        // A year's rate, as the Tel Aviv Stock Exchange lists it.
        custody: vec![custody(
            dec!(0.15),
            Period::Year,
            Period::Month,
            Some(ils(dec!(75))),
        )],
        conversion: markup_only(Markup::UpTo(Percent(dec!(0.7)))),
        // Part 9: "periodic management fees, up to ₪80 a month".
        handling: Some(handling(dec!(80), 0)),
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![
            Caveat::at_most(
                "A \"periodic management fee\" of up to ₪80 a month is listed, without saying \
                 when it applies; the full ₪80 is assumed.",
            )
            .about_fee(FeeKind::Handling),
            Caveat::reading(
                "Custody is 0.15% a year, charged monthly.",
                "the row doesn't say what period the 0.15% is for; the exchange's actual \
                 averages for June 2026 (0.15% a year for foreign holdings) confirm a year",
            )
            .about_fee(FeeKind::Custody),
        ],
    };

    // The offer for new customers (trading_benefits page and its January 2026
    // rules). Anything it doesn't mention is priced as the full tariff,
    // which its fine print refers to.
    let new_customers = Plan {
        name: "New customers".into(),
        description: "Altshuler's offer for opening a new account with at least ₪5,000: no \
                      custody or management fees, with no end date; Tel Aviv stocks, ETFs \
                      and index funds for 0.07% (at least ₪2.90); and US stocks and ETFs for \
                      1¢ a share (at least $6)."
            .into(),
        trading: [
            vec![
                // The offer names continuously-traded ETFs (במסלול רציף), which
                // is how Tel Aviv ETFs trade.
                trade(
                    &[Stock, Etf, MutualFund],
                    &[Tlv],
                    percent(dec!(0.07), Some(ils(dec!(2.9)))),
                ),
                trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(6))),
                ),
            ],
            full_tariff.trading.clone(),
        ]
        .concat(),
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        conversion: full_tariff.conversion.clone(),
        handling: None,
        fractions_on: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            Caveat::published(
                "The 0.07% is for index funds (קרנות\u{a0}מחקות); active and money-market \
                 funds cost nothing to trade.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[MutualFund])
            .on(&[Tlv]),
            not_in_the_offer(
                "The offer doesn't mention bonds, so the full tariff's prices are assumed.",
            )
            .about(&[Bond]),
            not_in_the_offer(
                "The offer doesn't mention mutual funds abroad, so the full tariff's prices \
                 are assumed.",
            )
            .about(&[MutualFund])
            .on(&[Usa, Europe]),
            Caveat::reading(
                "\"Management fees\" are waived with no end date: taken to cover both \
                 custody and the monthly management fee.",
                "the tariff calls both of them management fees (custody is \
                 \u{201c}דמי\u{a0}ניהול/טיפול\u{a0}פקדון\u{201d}, the monthly fee \
                 \u{201c}דמי\u{a0}ניהול\u{a0}תקופתיים\u{201d}), and the offer's rules waive \
                 \u{201c}account management fees\u{201d} without distinguishing",
            )
            .about_fee(FeeKind::Custody),
            Caveat::published(
                "The offer needs an active account with at least ₪5,000 in it; after a year \
                 without activity its benefits may be withdrawn.",
            ),
            Caveat::not_counted("The ₪200 gift for opening an account isn't included."),
        ],
    };

    Broker {
        name: "Altshuler Shaham Trade".into(),
        short_name: "Altshuler".into(),
        new_customer_plan: 1, // New customers
        description: "An investment house's trading platform, not a bank. Its full tariff \
                      has three tracks for US stocks and ETFs; new customers get a cheaper \
                      offer instead."
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
            markup_up_to(dec!(0.7)),
            Caveat::published("No price is listed for European stocks or ETFs.")
                .about_fee(FeeKind::Trade)
                .about(&[Stock, Etf])
                .on(&[Europe]),
        ],
        plans: vec![full_tariff, new_customers],
    }
}

// ─────────────────────────── Bank Leumi ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn leumi() -> Broker {
    let broker_caveats = vec![
        Caveat::may_cost_more(
            "The bank doesn't publish its conversion markup, so it's counted as 0: \
             conversions may cost more than shown.",
            "conversion markup not published",
        )
        .about_fee(FeeKind::Markup)
        .on(&[Usa, Europe]),
        Caveat::published("Active (non-index) mutual funds on Tel Aviv have no trade fee.")
            .about_fee(FeeKind::Trade)
            .about(&[MutualFund])
            .on(&[Tlv]),
    ];
    let quarterly = |exchanges: &[Exchange], rate: Decimal| CustodyFee {
        exchanges: exchanges.to_vec(),
        ..custody(rate, Period::Quarter, Period::Quarter, None)
    };

    // Online prices (Nispach Heh): Leumi Trade, the website and the app. The
    // branch prices in the main tables are higher.
    let online_conversion = PercentFee {
        percent: Percent(dec!(0.16)),
        min: Some(usd(dec!(5.76))),
        max: Some(usd(dec!(2400))),
    };
    let online = Plan {
        name: "Online".into(),
        description: "Leumi's standard prices for trading yourself in Leumi Trade, on the \
                      website or in the app (the tariff's appendix on direct channels). \
                      Trading through a banker at a branch costs more."
            .into(),
        trading: vec![
            // Tariff 4(a)(1) excludes mutual funds, but its footnote puts
            // index-tracking funds (Keren Mechaka) and ETFs back in. Active
            // funds carry no trade fee, at most a distribution fee in some
            // cases (4(b)(2)).
            trade(
                &[],
                &[Tlv],
                Price::Percent {
                    percent: Percent(dec!(0.4)),
                    min: Some(ils(dec!(26))),
                    max: Some(ils(dec!(6300))),
                },
            ),
            trade(
                &[],
                &[Usa, Europe],
                Price::Percent {
                    percent: Percent(dec!(0.3)),
                    min: Some(usd(dec!(24))),
                    max: Some(usd(dec!(6750))),
                },
            ),
        ],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![
            quarterly(&[Tlv], dec!(0.15)),
            quarterly(&[Usa, Europe], dec!(0.2)),
        ],
        conversion: ConversionFee {
            fee: online_conversion,
            or_if_less: None,
            markup: Markup::NotPublished,
        },
        handling: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![],
    };

    // Customer group from Nispach Alef. Only the parts that differ from Online.
    let plus18 = Plan {
        name: "Online, 'Leumi 18+'".into(),
        description: "Online prices with the 'Leumi 18+' customer-group discount: half \
                      the custody fee, and half the branch's conversion fee where that's \
                      less than the online one. It's a group for young customers; check your \
                      eligibility with the bank."
            .into(),
        custody: vec![
            quarterly(&[Tlv], dec!(0.075)),
            quarterly(&[Usa, Europe], dec!(0.1)),
        ],
        // 50% off the branch fee, but not below its minimum. Benefits don't
        // add up: each fee is the better of the group's and the online one
        // (the tariff's first page), and online is less below $4,500.
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: Some(usd(dec!(7.2))),
                max: Some(usd(dec!(3000))),
            },
            or_if_less: Some(online_conversion),
            markup: Markup::NotPublished,
        },
        caveats: vec![
            Caveat::at_most(
                "Conversion is 50% off the branch fee. Whether its $3,000 maximum is \
                 halved too isn't stated; it's assumed not.",
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            // The maximum only binds above $3M; halved, above $1.5M.
            .when_above(usd(dec!(1_500_000))),
        ],
        ..online.clone()
    };

    // Buying an index fund on Tel Aviv by standing order (tariff 4(a)(1), note
    // 10). Any Online customer can set one up; it's a plan here so it can be
    // compared with buying the same funds by one-off orders. The note is
    // only about buying: selling costs what it does on Online.
    let standing_order = Plan {
        name: "Online, monthly standing order".into(),
        description: "Not a separate account: an Online customer buying Tel Aviv index \
                      funds by monthly standing order, which costs 0.225% (at least ₪5) \
                      instead of 0.4% (at least ₪26). Everything else is priced as Online."
            .into(),
        standing_orders: vec![trade(
            &[MutualFund],
            &[Tlv],
            Price::Percent {
                percent: Percent(dec!(0.225)),
                min: Some(ils(dec!(5))),
                max: Some(ils(dec!(6300))),
            },
        )],
        caveats: vec![
            Caveat::published(
                "Only buying index funds on Tel Aviv by standing order is cheaper; \
                 everything else costs the same as Online.",
            )
            .about_fee(FeeKind::StandingOrder),
        ],
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
            trade(&[], &[Tlv], Price::Flat(ils(dec!(4)))),
            trade(&[], &[Usa, Europe], Price::Flat(usd(dec!(4)))),
        ],
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![
            online.custody[0].clone(),
            quarterly(&[Usa, Europe], dec!(0.15)),
        ],
        // 50% off the branch fee (0.2%, maximum $3,000), minimum $3. Less
        // than the online fee on any amount, so it needs no `or_if_less`.
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: Some(usd(dec!(3))),
                max: Some(usd(dec!(1500))),
            },
            or_if_less: None,
            markup: Markup::NotPublished,
        },
        handling: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![
            Caveat::may_cost_more(
                "₪4 is only stated for orders up to ₪30,000. Larger orders are assumed to \
                 cost the same.",
                "₪4 is stated only for orders up to ₪30,000",
            )
            .about_fee(FeeKind::Trade)
            .on(&[Tlv])
            .when_above(ils(dec!(30_000))),
            Caveat::may_cost_more(
                "$4 is only stated for orders up to $8,000. Larger orders are assumed to \
                 cost the same.",
                "$4 is stated only for orders up to $8,000",
            )
            .about_fee(FeeKind::Trade)
            .on(&[Usa, Europe])
            .when_above(usd(dec!(8000))),
            Caveat::at_most(
                "Pepper's ₪4 covers stocks, T-bills and bonds. Index funds on Tel Aviv \
                 aren't mentioned, so they're given the Online price.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[MutualFund])
            .on(&[Tlv]),
            Caveat::may_cost_more(
                "Conversion is 50% off the branch fee with a $3 minimum. Its $1,500 \
                 maximum (half the branch's) is assumed.",
                "the conversion maximum is assumed halved",
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            // The halved maximum only binds above $1.5M.
            .when_above(usd(dec!(1_500_000))),
        ],
    };

    Broker {
        name: "Bank Leumi".into(),
        short_name: "Leumi".into(),
        new_customer_plan: 0, // Online
        description: "One of Israel's largest banks. Its securities prices depend on how \
                      you trade (online in Leumi Trade, or at a branch), on customer groups, \
                      and on the Pepper app, so it has several plans here."
            .into(),
        tariff_date: Some(TariffDate::Day(date!(2026 - 06 - 29))),
        source_url: Some(
            "https://www.bankleumi.co.il/static-files/Commissions_Leumi/AmlotYechidimL.pdf".into(),
        ),
        caveats: broker_caveats,
        plans: vec![online, plus18, standing_order, pepper],
    }
}

// ─────────────────────────── Excellence Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn excellence() -> Broker {
    // Row 6, US securities, priced by the trading system the customer uses.
    let us_tracks = |securities: &[Security]| {
        vec![
            Track {
                name: "3¢ a share".into(),
                trading: vec![trade(
                    securities,
                    &[Usa],
                    per_share(usd(dec!(0.03)), usd(dec!(8))),
                )],
            },
            Track {
                name: "$11 per order".into(),
                trading: vec![trade(securities, &[Usa], Price::Flat(usd(dec!(11))))],
            },
            Track {
                name: "0.3% plus the broker's fee".into(),
                trading: vec![trade(securities, &[Usa], percent(dec!(0.3), None))],
            },
        ]
    };
    // Row 1 covers every Tel Aviv security but traditional mutual funds,
    // ETFs and index funds (קרנות מחקות) included; row 7 everything outside
    // the US, its minimum in the trade's currency.
    let tel_aviv = |securities: &[Security]| {
        trade(securities, &[Tlv], percent(dec!(0.4), Some(ils(dec!(10)))))
    };
    let outside_us = trade(&[], &[Europe], percent(dec!(0.4), Some(eur(dec!(35)))));

    let full_tariff = Plan {
        name: "Full tariff".into(),
        description: "Excellence's published maximum prices, including custody of 0.6% a \
                      quarter and a handling fee of ₪99 a month. US trades are priced by \
                      the trading system you choose; the app uses the cheapest for your \
                      inputs."
            .into(),
        trading: vec![tel_aviv(&[]), outside_us.clone()],
        tracks: us_tracks(&[]),
        standing_orders: vec![],
        standing_order_conversion: None,
        // Row 8: 0.6% a quarter, at least ₪40 a month.
        custody: vec![custody(
            dec!(0.6),
            Period::Quarter,
            Period::Month,
            Some(ils(dec!(40))),
        )],
        // 0.1%, plus a markup set in the customer's agreement.
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: None,
                max: None,
            },
            or_if_less: None,
            markup: Markup::NotPublished,
        },
        handling: Some(handling(dec!(99), 0)),
        fractions_on: vec![Usa],
        min_first_deposit: None,
        caveats: vec![
            Caveat::not_counted(
                "US trades cost at most 2% of their value; the 0.3% track adds the broker's \
                 fee, which isn't included.",
            )
            .about_fee(FeeKind::Trade)
            .on(&[Usa]),
            Caveat::at_most(
                "Accounts that hold assets rather than trade actively may be charged 0.1% a \
                 quarter for custody instead; the general 0.6% is used.",
            )
            .about_fee(FeeKind::Custody),
            Caveat::may_cost_more(
                "The conversion markup is set in each customer's agreement and isn't \
                 published, so it's counted as 0.",
                "conversion markup not published",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe]),
        ],
    };

    let typical_offer = Plan {
        name: "Typical offer".into(),
        description: "What new customers are usually offered: 0.07% in Tel Aviv (at least \
                      ₪3), US stocks and ETFs for 1¢ a share (at least $6), no custody, and \
                      ₪15 a month after two free years; converting costs 2 agorot a dollar."
            .into(),
        trading: vec![
            trade(
                &[Stock, Etf],
                &[Tlv],
                percent(dec!(0.07), Some(ils(dec!(3)))),
            ),
            tel_aviv(&[Bond, MutualFund]),
            trade(
                &[Stock, Etf],
                &[Usa],
                per_share(usd(dec!(0.01)), usd(dec!(6))),
            ),
            outside_us,
        ],
        // The offer doesn't price US bonds and funds: the full tariff's tracks.
        tracks: us_tracks(&[Bond, MutualFund]),
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        // "2 agorot a dollar", as its site states the cost of converting.
        conversion: markup_only(Markup::PerDollar(ils(dec!(0.02)))),
        handling: Some(handling(dec!(15), 24)),
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(10000))),
        caveats: vec![
            typical_offer_caveat("Excellence", "gemeltop.co.il, tradingil.co.il"),
            not_in_the_offer(
                "The offer prices only stocks and ETFs here, so the full tariff's prices \
                 are used.",
            )
            .about(&[Bond, MutualFund])
            .on(&[Tlv]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .about(&[Bond, MutualFund])
                .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .on(&[Europe]),
            Caveat::published("Some of its trading systems charge at least $5 instead of $6.")
                .about_fee(FeeKind::Trade)
                .about(&[Stock, Etf])
                .on(&[Usa]),
            no_custody_in_the_offer(
                "No custody: the offer's only holding cost is the monthly handling fee, as \
                 comparison sites list it. Its site states the two free years.",
            ),
            Caveat::not_counted(
                "Some sign-up links offer three free years and a refund of commissions; not \
                 included.",
            )
            .about_fee(FeeKind::Handling),
        ],
    };

    Broker {
        name: "Excellence Trade".into(),
        short_name: "Excellence".into(),
        new_customer_plan: 1, // Typical offer
        description: "The trading arm of the Phoenix investment house. It publishes only a \
                      full tariff of maximum prices; new customers are usually offered far \
                      less."
            .into(),
        tariff_date: None, // not stated in the PDF
        source_url: Some("https://www.xnes.co.il/media/nwogkoos/taarifon.pdf".into()),
        caveats: vec![],
        plans: vec![full_tariff, typical_offer],
    }
}

// ─────────────────────────── IBI ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn ibi() -> Broker {
    // Everything but Tel Aviv stocks and ETFs, as the full tariff prices it.
    let abroad = vec![
        // Foreign funds: "0.275% plus the broker's cost".
        trade(&[MutualFund], &[Usa, Europe], percent(dec!(0.275), None)),
        // Minimums "in the trade's currency".
        trade(&[Bond], &[Usa], percent(dec!(0.225), Some(usd(dec!(20))))),
        trade(
            &[Bond],
            &[Europe],
            percent(dec!(0.225), Some(eur(dec!(20)))),
        ),
        trade(
            &[Stock, Etf],
            &[Europe],
            percent(dec!(0.3), Some(eur(dec!(30)))),
        ),
    ];
    let tel_aviv_funds = trade(
        &[MutualFund],
        &[Tlv],
        percent(dec!(0.08), Some(ils(dec!(5)))),
    );
    let tel_aviv_bonds = trade(&[Bond], &[Tlv], percent(dec!(0.15), Some(ils(dec!(3.5)))));
    let conversion = markup_only(Markup::UpTo(Percent(dec!(0.7))));

    let full_tariff = Plan {
        name: "Full tariff".into(),
        description: "IBI's published maximum prices, including a handling fee of ₪50 a \
                      month and custody of 0.1% a quarter. For US stocks and ETFs you choose \
                      one of four tracks; the app uses the cheapest for your inputs."
            .into(),
        trading: [
            vec![
                trade(
                    &[Stock, Etf],
                    &[Tlv],
                    percent(dec!(0.15), Some(ils(dec!(3.5)))),
                ),
                tel_aviv_bonds.clone(),
                tel_aviv_funds.clone(),
            ],
            abroad.clone(),
        ]
        .concat(),
        // US stocks and ETFs, on one of four tracks.
        tracks: vec![
            Track {
                name: "1¢ a share".into(),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(10))),
                )],
            },
            Track {
                name: "$14 per order".into(),
                trading: vec![trade(&[Stock, Etf], &[Usa], Price::Flat(usd(dec!(14))))],
            },
            Track {
                name: "0.15%".into(),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    percent(dec!(0.15), Some(usd(dec!(10)))),
                )],
            },
            Track {
                name: "0.15% + 1¢ a share".into(),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    Price::PercentPlusPerShare {
                        percent: Percent(dec!(0.15)),
                        per_share: usd(dec!(0.01)),
                        min: Some(usd(dec!(6))),
                        max: None,
                    },
                )],
            },
        ],
        standing_orders: vec![],
        standing_order_conversion: None,
        // "0.1% of the account's value a quarter", but its site promises its
        // trading customers no custody "on any fund in the system, index
        // funds included".
        custody: vec![
            CustodyFee {
                securities: vec![MutualFund],
                exchanges: vec![Tlv],
                ..custody(dec!(0), Period::Quarter, Period::Quarter, None)
            },
            custody(dec!(0.1), Period::Quarter, Period::Quarter, None),
        ],
        conversion: conversion.clone(),
        handling: Some(handling(dec!(50), 0)),
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(15000))),
        caveats: vec![
            Caveat::at_most(
                "The tariff's prices are maximums (\"up to\"); the handling fee is taken at \
                 its ₪50.",
            )
            .about_fee(FeeKind::Handling),
            Caveat::published(
                "No custody, as IBI's site promises for every fund; the tariff itself would \
                 charge 0.1% a quarter.",
            )
            .about_fee(FeeKind::Custody)
            .about(&[MutualFund])
            .on(&[Tlv]),
        ],
    };

    let typical_offer = Plan {
        name: "Typical offer".into(),
        description: "What new customers are usually offered: 0.08% in Tel Aviv (at least \
                      ₪2.35), US stocks and ETFs from $7.50 a trade, no custody, and ₪15 a \
                      month."
            .into(),
        trading: [
            vec![
                trade(
                    &[Stock, Etf],
                    &[Tlv],
                    percent(dec!(0.08), Some(ils(dec!(2.35)))),
                ),
                tel_aviv_bonds,
                tel_aviv_funds,
                trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(7.5))),
                ),
            ],
            abroad,
        ]
        .concat(),
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        conversion,
        handling: Some(handling(dec!(15), 0)),
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(15000))),
        caveats: vec![
            typical_offer_caveat("IBI", "gemeltop.co.il"),
            not_in_the_offer(
                "The offer prices only stocks and ETFs here, so the full tariff's prices \
                 are used.",
            )
            .about(&[Bond, MutualFund])
            .on(&[Tlv]),
            Caveat::reading(
                "The offer states only the $7.50 minimum; 1¢ a share, as the tariff's first \
                 track, is assumed.",
                "the tariff's first US track is 1¢ a share with a minimum, and the offer \
                 gives its $7.50 as a minimum",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Stock, Etf])
            .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .about(&[Bond, MutualFund])
                .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .on(&[Europe]),
            no_custody_in_the_offer(
                "No custody: the offer's only holding cost is the monthly handling fee, as \
                 comparison sites list it. IBI's site states no custody on any fund.",
            ),
            Caveat::not_counted("The ₪300 gift for opening an account isn't included."),
        ],
    };

    Broker {
        name: "IBI".into(),
        short_name: "IBI".into(),
        new_customer_plan: 1, // Typical offer
        description: "An investment house's trading platform (IBI TRADE, with the IBI SMART \
                      app). It publishes only a full tariff of maximum prices; new customers \
                      are usually offered far less. Opening an account takes at least \
                      ₪15,000."
            .into(),
        tariff_date: Some(TariffDate::Day(date!(2026 - 07 - 01))),
        source_url: Some("https://campaign.ibi.co.il/PDF/TRADE/TAARIFON_IBI.pdf".into()),
        caveats: vec![
            markup_up_to(dec!(0.7)),
            Caveat::published(
                "Managed (active) funds of 12 fund managers cost nothing to trade; the app's \
                 mutual funds are index funds, which aren't included.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[MutualFund])
            .on(&[Tlv]),
            Caveat::not_counted(
                "Foreign funds cost 0.275% plus the foreign broker's cost, which isn't \
                 included.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[MutualFund])
            .on(&[Usa, Europe]),
        ],
        plans: vec![full_tariff, typical_offer],
    }
}

// ─────────────────────────── Interactive Israel ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn interactive() -> Broker {
    let us_shares = trade(
        &[Stock, Etf],
        &[Usa],
        per_share(usd(dec!(0.01)), usd(dec!(2.5))),
    );
    // Converting shekels: 0.002%, at least ₪10, which covers up to ₪500,000,
    // at the live market rate, as its site says.
    let conversion = ConversionFee {
        fee: PercentFee {
            percent: Percent(dec!(0.002)),
            min: Some(ils(dec!(10))),
            max: None,
        },
        or_if_less: None,
        markup: Markup::MarketRate,
    };
    let standard = Plan {
        name: "Standard".into(),
        description: "Its one price list: US stocks and ETFs for 1¢ a share (at least \
                      $2.50), European ones for 0.15% (at least €2.50), converting shekels \
                      for ₪10, and no custody, handling fee or minimum deposit."
            .into(),
        trading: vec![
            us_shares.clone(),
            trade(
                &[Stock, Etf],
                &[Europe],
                percent(dec!(0.15), Some(eur(dec!(2.5)))),
            ),
            trade(&[Bond], &[Usa], percent(dec!(0.2), Some(usd(dec!(10))))),
            trade(&[Bond], &[Europe], percent(dec!(0.2), Some(eur(dec!(10))))),
        ],
        tracks: vec![],
        // Its automatic investment plan buys at the usual price and converts
        // for free, at the same unpublished spread.
        standing_orders: vec![us_shares],
        standing_order_conversion: Some(ConversionFee {
            fee: PercentFee::FREE,
            ..conversion.clone()
        }),
        custody: vec![],
        conversion,
        handling: None,
        fractions_on: vec![Usa],
        min_first_deposit: None,
        caveats: vec![
            Caveat::reading(
                "Its automatic investment plan converts for free; taken as buying every \
                 month, for US stocks and ETFs.",
                "the tariff waives the conversion fee for conversions within a fixed \
                 automatic investment plan, which is what buying by standing order is",
            )
            .about_fee(FeeKind::StandingOrder)
            .about(&[Stock, Etf])
            .on(&[Usa]),
            Caveat::published(
                "Fractions of a share are charged as whole shares, as its tariff says.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Stock, Etf])
            .on(&[Usa]),
            Caveat::reading(
                "Bonds are charged 0.2% of their face value; the trade's value is used \
                 instead.",
                "bonds trade within a few percent of their face value, so the fee is nearly \
                 the same",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond]),
            Caveat::not_counted(
                "Its tariff's price for mutual funds is unclear, so it isn't included.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[MutualFund]),
            Caveat::published("Tel Aviv securities are traded for institutional clients only.")
                .about_fee(FeeKind::Trade)
                .on(&[Tlv]),
        ],
    };

    Broker {
        name: "Interactive Israel".into(),
        short_name: "Interactive".into(),
        new_customer_plan: 0, // Standard
        description: "The Israeli introducing broker for Interactive Brokers, run by MEXEM \
                      (regulated in Cyprus, not by the Israel Securities Authority). US and \
                      European exchanges; Tel Aviv only for institutional clients."
            .into(),
        tariff_date: Some(TariffDate::Day(date!(2026 - 09 - 23))),
        source_url: Some(
            "https://www.inter-il.com/wp-content/uploads/2026/09/\
             %D7%AA%D7%A2%D7%A8%D7%99%D7%A4%D7%95%D7%9F-%D7%A2%D7%9E%D7%9C%D7%95%D7%AA-\
             23.09.2026-1.pdf"
                .into(),
        ),
        caveats: vec![
            Caveat::published(
                "Converting is at the live market rate (שער\u{a0}רציף), as its site says, for \
                 up to ₪500,000: the fee is the whole cost the broker adds.",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe]),
            Caveat::not_counted(
                "The currency market's own bid-ask spread, a few hundredths of a percent: \
                 Interactive Brokers, where the account is held, passes market quotes through \
                 and charges the fee instead of a markup. Its automatic conversions may move \
                 the rate by up to 0.03% instead.",
            )
            .on(&[Usa, Europe]),
            Caveat::reading(
                "Converting back to shekels is taken to cost the same as converting them: \
                 0.002%, at least ₪10.",
                "the tariff prices conversions by the currency traded, and its only note \
                 is on converting shekels; nothing prices the way back differently",
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe]),
        ],
        plans: vec![standard],
    }
}

// ─────────────────────────── Meitav Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn meitav() -> Broker {
    // Row 1: every Tel Aviv security but TA options and futures.
    let tel_aviv = |securities: &[Security]| {
        trade(securities, &[Tlv], percent(dec!(0.3), Some(ils(dec!(10)))))
    };
    let abroad = vec![
        trade(
            &[Bond],
            &[Usa, Europe],
            percent(dec!(0.3), Some(usd(dec!(25)))),
        ),
        // Foreign funds, not counting clearing and correspondent fees.
        trade(
            &[MutualFund],
            &[Usa, Europe],
            percent(dec!(0.2), Some(usd(dec!(20)))),
        ),
        // "The world's other exchanges: 0.25%, min 25 in the trade's currency".
        trade(
            &[Stock, Etf],
            &[Europe],
            percent(dec!(0.25), Some(eur(dec!(25)))),
        ),
    ];
    let markup = Markup::UpTo(Percent(dec!(0.7)));

    let full_tariff = Plan {
        name: "Full tariff".into(),
        description: "Meitav Trade's published maximum prices, including custody of at \
                      least ₪270 a quarter and a handling fee of ₪90 a month. For US stocks \
                      you choose one of two tracks; the app uses the cheapest for your \
                      inputs."
            .into(),
        trading: [vec![tel_aviv(&[])], abroad.clone()].concat(),
        tracks: vec![
            Track {
                name: "1¢ a share".into(),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(12))),
                )],
            },
            Track {
                name: "0.7%".into(),
                trading: vec![trade(
                    &[Stock, Etf],
                    &[Usa],
                    percent(dec!(0.7), Some(usd(dec!(12)))),
                )],
            },
        ],
        standing_orders: vec![],
        standing_order_conversion: None,
        // "0.15%, at least ₪270 a quarter": a quarter's rate, as the Tel Aviv
        // Stock Exchange lists it (0.6% a year).
        custody: vec![custody(
            dec!(0.15),
            Period::Quarter,
            Period::Quarter,
            Some(ils(dec!(270))),
        )],
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.13)),
                min: None,
                max: None,
            },
            or_if_less: None,
            markup,
        },
        handling: Some(handling(dec!(90), 0)),
        fractions_on: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            Caveat::reading(
                "Custody is 0.15% a quarter (0.6% a year).",
                "the row doesn't say what period the 0.15% is for; the exchange's actual \
                 averages for June 2026 (0.6% a year) confirm a quarter",
            )
            .about_fee(FeeKind::Custody),
            Caveat::not_counted(
                "Foreign funds add clearing and correspondent fees, which aren't included.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[MutualFund])
            .on(&[Usa, Europe]),
            Caveat::at_most(
                "The handling fee is listed as up to ₪90 a month; the full ₪90 is assumed.",
            )
            .about_fee(FeeKind::Handling),
        ],
    };

    let typical_offer = Plan {
        name: "Typical offer".into(),
        description: "What new customers are usually offered: 0.07% for Tel Aviv ETFs and \
                      0.08% for stocks (at least ₪4.65), US stocks and ETFs for 1¢ a share \
                      (at least $5), no custody or conversion fee, and ₪15 a month after \
                      two free years."
            .into(),
        trading: [
            vec![
                trade(&[Etf], &[Tlv], percent(dec!(0.07), Some(ils(dec!(4.65))))),
                trade(&[Stock], &[Tlv], percent(dec!(0.08), Some(ils(dec!(4.65))))),
                tel_aviv(&[Bond, MutualFund]),
                trade(
                    &[Stock, Etf],
                    &[Usa],
                    per_share(usd(dec!(0.01)), usd(dec!(5))),
                ),
            ],
            abroad,
        ]
        .concat(),
        tracks: vec![],
        standing_orders: vec![],
        standing_order_conversion: None,
        // Its site: no conversion fee, and no custody.
        custody: vec![],
        conversion: markup_only(markup),
        handling: Some(handling(dec!(15), 24)),
        fractions_on: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            typical_offer_caveat("Meitav", "gemeltop.co.il, tradingil.co.il"),
            Caveat::published("No custody and no conversion fee, as Meitav's site states.")
                .about_fee(FeeKind::Custody),
            not_in_the_offer(
                "The offer prices only stocks and ETFs here, so the full tariff's prices \
                 are used.",
            )
            .about(&[Bond, MutualFund])
            .on(&[Tlv]),
            Caveat::published("Some of its trading systems charge at least $7.50 instead of $5.")
                .about_fee(FeeKind::Trade)
                .about(&[Stock, Etf])
                .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .about(&[Bond, MutualFund])
                .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .on(&[Europe]),
            Caveat::not_counted("The ₪100 gift for opening an account isn't included."),
        ],
    };

    Broker {
        name: "Meitav Trade".into(),
        short_name: "Meitav".into(),
        new_customer_plan: 1, // Typical offer
        description: "The largest exchange member that isn't a bank, part of the Meitav \
                      investment house. It publishes only a full tariff of maximum prices; \
                      new customers are usually offered far less."
            .into(),
        tariff_date: Some(TariffDate::Month(date!(2025 - 01 - 01))),
        source_url: Some("https://www.meitav.co.il/media/z2hkkhku/taarifon.pdf".into()),
        caveats: vec![markup_up_to(dec!(0.7))],
        plans: vec![full_tariff, typical_offer],
    }
}
