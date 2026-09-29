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
use crate::Security::{self, Bond, Etf, IndexFund, Stock};
use crate::{
    Broker, BrokerKind, Caveat, ConversionFee, CustodyFee, FeeKind, HandlingFee, Markup, Money,
    Percent, PercentFee, Period, Plan, Price, Source, TariffDate, Track, TradeFee, ils, iso, usd,
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
    date!(2026 - 09 - 29)
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

/// The same, taken off by the month's trade fees: the typical offers.
fn handling_less_trade_fees(per_month: Decimal, free_months: u32) -> HandlingFee {
    HandlingFee {
        less_trade_fees: true,
        ..handling(per_month, free_months)
    }
}

fn eur(amount: Decimal) -> Money {
    Money::from_decimal(amount, iso::EUR)
}

/// A page a number comes from, to link beside its caveat.
fn source(name: &str, url: &str) -> Source {
    Source {
        name: name.to_owned(),
        url: url.to_owned(),
    }
}

/// The exchange's calculator of its members' tariffs and actual average fees.
#[must_use]
pub fn exchange_calculator() -> Source {
    source(
        "The Tel Aviv Stock Exchange's fee calculator",
        "https://market.tase.co.il/he/market_data/trading_fees",
    )
}

/// gemeltop.co.il's comparison of the investment houses' offers.
#[must_use]
pub fn gemeltop_comparison() -> Source {
    source(
        "gemeltop.co.il's comparison of trading accounts",
        "https://gemeltop.co.il/hashvaat-batei-hashkaot-mschar-atzmai/",
    )
}

/// A broker's offer on gemeltop.co.il, at `slug`.
fn gemeltop(slug: &str) -> Source {
    source(
        "The offer on gemeltop.co.il",
        &format!("https://gemeltop.co.il/{slug}/"),
    )
}

/// tradingil.co.il's comparison of trading accounts.
#[must_use]
pub fn tradingil_comparison() -> Source {
    source(
        "tradingil.co.il's comparison of trading accounts",
        "https://tradingil.co.il/%D7%97%D7%A9%D7%91%D7%95%D7%9F-%D7%9E%D7%A1%D7%97%D7%A8-\
         %D7%A2%D7%A6%D7%9E%D7%90%D7%99/",
    )
}

/// A broker's offer on tradingil.co.il, at `slug`.
fn tradingil(slug: &str) -> Source {
    source(
        "The offer on tradingil.co.il",
        &format!("https://tradingil.co.il/{slug}/"),
    )
}

/// tradingil.co.il's comparison of what converting currency costs at each.
#[must_use]
pub fn tradingil_conversions() -> Source {
    source(
        "tradingil.co.il on conversion costs",
        "https://tradingil.co.il/%D7%94%D7%9E%D7%A8%D7%AA-%D7%9E%D7%98%D7%97/",
    )
}

/// broker.co.il's comparison of Meitav Trade and IBI Trade.
#[must_use]
pub fn broker_co_il() -> Source {
    source(
        "broker.co.il's comparison of Meitav and IBI",
        "https://www.broker.co.il/blog/?ContentID=66316",
    )
}

/// No conversion fee, only a markup.
fn markup_only(markup: Markup) -> ConversionFee {
    ConversionFee {
        fee: PercentFee::FREE,
        or_if_less: None,
        markup,
    }
}

/// What a typical-offer plan's caveats begin with: where its numbers come
/// from (`offer_pages`), and why they can be trusted.
fn typical_offer_caveat(broker: &str, sites: &str, offer_pages: &[&Source]) -> Caveat {
    Caveat::reading(
        &format!(
            "Not published by {broker}: the terms comparison sites list for joining \
             ({sites}, September 2026). Ask for them when you open the account."
        ),
        "the Tel Aviv Stock Exchange's actual average fees for June 2026 match them: \
         0.07%–0.085% on Tel Aviv stocks, against full tariffs of 0.15%–0.4%",
    )
    .sources(offer_pages)
    .source(&exchange_calculator())
}

/// Why a typical offer prices some securities as the full tariff does.
fn not_in_the_offer(text: &str) -> Caveat {
    Caveat::at_most(text).about_fee(FeeKind::Trade)
}

/// Why a typical offer's handling fee is taken off by the month's trade fees,
/// and `support`: which site says so, with the pages.
fn handling_less_trade_fees_caveat(support: &str, sources: &[&Source]) -> Caveat {
    Caveat::reading(
        "The month's trade fees are taken off the handling fee, so a month that pays ₪15 \
         or more in them pays no handling fee.",
        support,
    )
    .about_fee(FeeKind::Handling)
    .sources(sources)
}

/// Why a typical offer charges no custody, on what the comparison sites say
/// (`sources`: their pages, and the broker's if it says so too).
fn no_custody_in_the_offer(text: &str, sources: &[&Source]) -> Caveat {
    Caveat::reading(
        text,
        "the comparison sites list a holding cost wherever there is one (\u{201c}plus \
         custody\u{201d} for the banks), and the handling fee is the only one they list \
         for the offer",
    )
    .about_fee(FeeKind::Custody)
    .sources(sources)
}

// ─────────────────────────── Altshuler Shaham Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn altshuler() -> Broker {
    let offer = source(
        "Altshuler's joining offer",
        "https://www.as-invest.co.il/trade/trading_benefits/",
    );
    let rules = source(
        "The offer's rules (PDF)",
        "https://www.as-invest.co.il/media/p1pb1i2x/\
         %D7%AA%D7%A7%D7%A0%D7%95%D7%9F-%D7%90%D7%9C%D7%98%D7%A9%D7%95%D7%9C%D7%A8-\
         %D7%A9%D7%97%D7%9D-%D7%98%D7%A8%D7%99%D7%99%D7%93.pdf",
    );
    let faq = source("Altshuler's FAQ", "https://www.as-invest.co.il/trade/faq/");
    let currency_page = source(
        "Altshuler's currency page",
        "https://www.as-invest.co.il/trade/currency_exchange/",
    );
    let full_tariff = Plan {
        name: "Full tariff".into(),
        description: "Altshuler's published price list. For US stocks and ETFs you choose \
                      one of three tracks when opening the account; the app uses the \
                      cheapest for your inputs."
            .into(),
        trading: vec![
            // ETFs and funds share the tariff's funds row, but ETFs have a lower minimum.
            trade(&[Etf], &[Tlv], percent(dec!(0.15), Some(ils(dec!(3.5))))),
            trade(
                &[IndexFund],
                &[Tlv],
                percent(dec!(0.15), Some(ils(dec!(16)))),
            ),
            trade(
                &[Stock, Bond],
                &[Tlv],
                percent(dec!(0.15), Some(ils(dec!(3.5)))),
            ),
            // The tariff's "foreign bonds and funds" row; only the US, since
            // its site says it trades on the Israeli and US exchanges only.
            trade(
                &[Bond, IndexFund],
                &[Usa],
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
            .about_fee(FeeKind::Custody)
            .source(&exchange_calculator()),
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
                    &[Stock, Etf, IndexFund],
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
            .about(&[IndexFund])
            .on(&[Tlv])
            .source(&offer),
            not_in_the_offer(
                "The offer doesn't mention bonds, so the full tariff's prices are assumed.",
            )
            .about(&[Bond]),
            not_in_the_offer(
                "The offer doesn't mention funds abroad, so the full tariff's prices are \
                 assumed.",
            )
            .about(&[IndexFund])
            .on(&[Usa]),
            Caveat::reading(
                "\"Management fees\" are waived with no end date: taken to cover both \
                 custody and the monthly management fee.",
                "the tariff calls both of them management fees (custody is \
                 \u{201c}דמי\u{a0}ניהול/טיפול\u{a0}פקדון\u{201d}, the monthly fee \
                 \u{201c}דמי\u{a0}ניהול\u{a0}תקופתיים\u{201d}), the offer's rules waive \
                 \u{201c}account management fees\u{201d} without distinguishing, and the \
                 exchange's actual averages for June 2026 show Altshuler's customers paying \
                 0% custody",
            )
            .about_fee(FeeKind::Custody)
            .source(&offer)
            .source(&rules)
            .source(&exchange_calculator()),
            Caveat::published(
                "The offer needs an active account with at least ₪5,000 in it; after a year \
                 without activity its benefits may be withdrawn.",
            )
            .source(&rules),
            Caveat::not_counted("The ₪200 gift for opening an account isn't included.")
                .source(&offer),
        ],
    };

    Broker {
        name: "Altshuler Shaham Trade".into(),
        short_name: "Altshuler".into(),
        kind: BrokerKind::InvestmentHouse,
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
            Caveat::reading(
                "The tariff says the conversion markup is up to 0.7%, and 0.7% is what it \
                 charges: no fee, and the rate shown when converting includes the markup, \
                 as its site says.",
                "tradingil.co.il's comparison of conversion costs lists Altshuler's markup \
                 as 0.7% (September 2026), and Altshuler's currency page says the shown \
                 rate is final, the market's rate plus the markup, with no fee",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&tradingil_conversions())
            .source(&currency_page),
            Caveat::published(
                "Altshuler trades on the Israeli and US exchanges only, as its site says: \
                 nothing in Europe is offered.",
            )
            .about_fee(FeeKind::Trade)
            .on(&[Europe])
            .source(&faq),
        ],
        plans: vec![full_tariff, new_customers],
    }
}

// ─────────────────────────── Bank Leumi ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn leumi() -> Broker {
    let tariff_url =
        "https://www.bankleumi.co.il/static-files/Commissions_Leumi/AmlotYechidimL.pdf";
    let tariff = source("Tariff (PDF)", tariff_url);
    let rates_page = source(
        "Leumi's exchange rates",
        "https://www.bankleumi.co.il/vgnprod/shearim.asp",
    );
    let pepper_site = source("Pepper's site", "https://www.pepper.co.il/");
    let pepper_package = source(
        "Pepper's package, as Leumi announced it",
        "https://www.leumi.co.il/he/node/2577",
    );
    let broker_caveats = vec![
        Caveat::reading(
            "Leumi converts at its published transfers-and-checks rate, about 0.9% from \
             the representative rate each way: a conversion markup of up to 0.9%, on top \
             of the fee. A rate agreed live in the app during trading hours may be a \
             little better.",
            "Leumi's exchange rates for 28 September 2026: it bought dollars at ₪3.0395 \
             and sold them at ₪3.0960 against a representative ₪3.0660 (0.86% under and \
             0.98% over), and euros at ₪3.4575 and ₪3.5218 against ₪3.4877",
        )
        .about_fee(FeeKind::Markup)
        .on(&[Usa, Europe])
        .source(&rates_page),
        Caveat::published("Active (non-index) mutual funds on Tel Aviv have no trade fee.")
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Tlv]),
    ];
    let quarterly = |exchanges: &[Exchange], rate: Decimal| CustodyFee {
        exchanges: exchanges.to_vec(),
        ..custody(rate, Period::Quarter, Period::Quarter, None)
    };

    // Measured from the bank's published rates (the caveat above).
    let markup = Markup::UpTo(Percent(dec!(0.9)));
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
            markup,
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
                      the custody fee, Tel Aviv bonds for 0.35% instead of 0.4%, and half \
                      the branch's conversion fee where that's less than the online one. \
                      It's a group for young customers; check your eligibility with the \
                      bank."
            .into(),
        // The group's 0.35% on bonds beats online's 0.4%, but comes with the
        // branch's minimum and maximum (₪27, ₪7,000), and benefits don't
        // stack: each fee is the better of the two. The group's rate within
        // online's bounds is never more than ₪1 off that.
        trading: [
            vec![trade(
                &[Bond],
                &[Tlv],
                Price::Percent {
                    percent: Percent(dec!(0.35)),
                    min: Some(ils(dec!(26))),
                    max: Some(ils(dec!(6300))),
                },
            )],
            online.trading.clone(),
        ]
        .concat(),
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
            markup,
        },
        caveats: vec![
            Caveat::reading(
                "Bonds on Tel Aviv cost the group's 0.35%, with Online's ₪26 minimum and \
                 ₪6,300 maximum.",
                "the group's price is 0.35% with the branch's bounds (at least ₪27, at most \
                 ₪7,000), and the tariff's first page gives each fee the better of the \
                 group's and the online price; the better of those two is within ₪1 of \
                 this on any order",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond])
            .on(&[Tlv])
            .source(&tariff),
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
            &[IndexFund],
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
                securities: vec![IndexFund],
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
            markup,
        },
        handling: None,
        fractions_on: vec![],
        min_first_deposit: None,
        caveats: vec![
            Caveat::may_cost_more(
                "₪4 is only stated for orders up to ₪30,000; Pepper's site says larger \
                 orders are priced by the tariff, without saying which row. They're \
                 assumed to cost ₪4 too.",
                "₪4 is stated only for orders up to ₪30,000",
            )
            .about_fee(FeeKind::Trade)
            .on(&[Tlv])
            .when_above(ils(dec!(30_000)))
            .source(&pepper_site),
            Caveat::may_cost_more(
                "$4 is only stated for orders up to $8,000; Pepper's site says larger \
                 orders are priced by the tariff, without saying which row. They're \
                 assumed to cost $4 too.",
                "$4 is stated only for orders up to $8,000",
            )
            .about_fee(FeeKind::Trade)
            .on(&[Usa, Europe])
            .when_above(usd(dec!(8000)))
            .source(&pepper_site),
            Caveat::reading(
                "Pepper's ₪4 row names stocks, T-bills and bonds; ETFs are taken to be \
                 included.",
                "the tariff's own \u{201c}stocks and bonds\u{201d} row counts ETFs and index \
                 funds in (part 4, footnote 4), and Pepper's row uses the same words",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Etf])
            .on(&[Tlv])
            .source(&tariff),
            Caveat::at_most(
                "Pepper's ₪4 covers stocks, T-bills and bonds. Index funds on Tel Aviv \
                 aren't mentioned, so they're given the Online price.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Tlv]),
            Caveat::published(
                "Custody is 0.15% a quarter on everything: the tariff's Pepper rate for \
                 foreign holdings, and Online's for Tel Aviv ones. Pepper's site says the \
                 same.",
            )
            .about_fee(FeeKind::Custody)
            .source(&pepper_site),
            Caveat::not_counted(
                "Pepper's package of March 2026, for customers who move their salary to \
                 it: a year without trade fees, minimums or currency fees, and a grant of \
                 up to ₪1,500. One year only.",
            )
            .source(&pepper_package),
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
        kind: BrokerKind::Bank,
        new_customer_plan: 0, // Online
        description: "One of Israel's largest banks. Its securities prices depend on how \
                      you trade (online in Leumi Trade, or at a branch), on customer groups, \
                      and on the Pepper app, so it has several plans here."
            .into(),
        tariff_date: Some(TariffDate::Day(date!(2026 - 06 - 29))),
        source_url: Some(tariff_url.into()),
        caveats: broker_caveats,
        plans: vec![online, plus18, standing_order, pepper],
    }
}

// ─────────────────────────── Excellence Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn excellence() -> Broker {
    let article = source(
        "Excellence's article on conversion costs",
        "https://www.xnes.co.il/academy/trading/account-fees/",
    );
    let on_gemeltop = gemeltop("excellence-trade-amlot");
    let on_tradingil = tradingil(
        "%D7%91%D7%A8%D7%95%D7%A7%D7%A8-%D7%9C%D7%9E%D7%A1%D7%97%D7%A8-%D7%91%D7%99%D7%A9%D7%A8%D7%90%D7%9C",
    );
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
        // 0.1%, plus a markup the tariff leaves to the customer's agreement
        // and the site puts at 2 agorot a dollar.
        conversion: ConversionFee {
            fee: PercentFee {
                percent: Percent(dec!(0.1)),
                min: None,
                max: None,
            },
            or_if_less: None,
            markup: Markup::PerDollar(ils(dec!(0.02))),
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
            Caveat::published(
                "The tariff leaves the conversion markup to each customer's agreement; \
                 Excellence's site puts it at 2 agorot a dollar, at most ₪200 on $10,000, \
                 which is used.",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&article),
            Caveat::at_most(
                "The tariff's 0.1% conversion fee is charged on top of the 2 agorot; \
                 Excellence's site describes the 2 agorot as the whole cost, so the fee \
                 may not apply.",
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            .source(&article),
        ],
    };

    let typical_offer = Plan {
        name: "Typical offer".into(),
        description: "What new customers are usually offered: 0.07% for Tel Aviv stocks, \
                      ETFs and index funds and 0.06% for bonds (at least ₪3), US stocks and \
                      ETFs for 1¢ a share (at least $6), no custody, and ₪15 a month after \
                      two free years, less that month's trade fees; converting costs 2 \
                      agorot a dollar."
            .into(),
        trading: vec![
            trade(
                &[Stock, Etf, IndexFund],
                &[Tlv],
                percent(dec!(0.07), Some(ils(dec!(3)))),
            ),
            trade(&[Bond], &[Tlv], percent(dec!(0.06), Some(ils(dec!(3))))),
            trade(
                &[Stock, Etf],
                &[Usa],
                per_share(usd(dec!(0.01)), usd(dec!(6))),
            ),
            outside_us,
        ],
        // The offer doesn't price US bonds and funds: the full tariff's tracks.
        tracks: us_tracks(&[Bond, IndexFund]),
        standing_orders: vec![],
        standing_order_conversion: None,
        custody: vec![],
        // "2 agorot a dollar", as its site states the cost of converting.
        conversion: markup_only(Markup::PerDollar(ils(dec!(0.02)))),
        handling: Some(handling_less_trade_fees(dec!(15), 24)),
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(10000))),
        caveats: vec![
            typical_offer_caveat(
                "Excellence",
                "gemeltop.co.il, tradingil.co.il",
                &[&on_gemeltop, &on_tradingil],
            ),
            Caveat::reading(
                "Index funds cost the offer's 0.07% and bonds 0.06%, at least ₪3 each; \
                 gemeltop.co.il lists only stocks and ETFs.",
                "tradingil.co.il lists index funds (קרן\u{a0}מחקה) with stocks and ETFs at \
                 0.07%, and bonds at 0.06%, for its joining offer (September 2026)",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond, IndexFund])
            .on(&[Tlv])
            .source(&on_tradingil),
            Caveat::published(
                "Converting costs 2 agorot a dollar and nothing else, as Excellence's site \
                 states: at most ₪200 on $10,000.",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&article),
            handling_less_trade_fees_caveat(
                "tradingil.co.il says the handling fee \u{201c}can be offset\u{201d} by trade \
                 fees in its joining offer (September 2026)",
                &[&on_tradingil],
            ),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .about(&[Bond, IndexFund])
                .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .on(&[Europe]),
            Caveat::published("Some of its trading systems charge at least $5 instead of $6.")
                .about_fee(FeeKind::Trade)
                .about(&[Stock, Etf])
                .on(&[Usa])
                .source(&on_tradingil),
            no_custody_in_the_offer(
                "No custody: the offer's only holding cost is the monthly handling fee, as \
                 comparison sites list it. Its site states the two free years.",
                &[&on_gemeltop, &on_tradingil],
            ),
            Caveat::not_counted(
                "Some sign-up links offer three free years and a refund of commissions; not \
                 included.",
            )
            .about_fee(FeeKind::Handling)
            .source(&on_gemeltop)
            .source(&on_tradingil),
        ],
    };

    Broker {
        name: "Excellence Trade".into(),
        short_name: "Excellence".into(),
        kind: BrokerKind::InvestmentHouse,
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
    let fund_page = source(
        "IBI's page on funds",
        "https://www.ibi.co.il/solutions/zero-balance-managed-funds/",
    );
    let currency_faq = source(
        "IBI's currency FAQ",
        "https://www.ibi.co.il/solutions/trading/forms-important-information/foreign-currency-faq/",
    );
    let on_gemeltop = gemeltop("ibi-trade-amlot");
    let on_tradingil = tradingil(
        "%D7%97%D7%A9%D7%91%D7%95%D7%9F-%D7%9E%D7%A1%D7%97%D7%A8-%D7%A2%D7%A6%D7%9E%D7%90%D7%99-ibi",
    );
    // Everything but Tel Aviv stocks and ETFs, as the full tariff prices it.
    let abroad = vec![
        // Foreign funds: "0.275% plus the broker's cost".
        trade(&[IndexFund], &[Usa, Europe], percent(dec!(0.275), None)),
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
        &[IndexFund],
        &[Tlv],
        percent(dec!(0.08), Some(ils(dec!(5)))),
    );
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
                    &[Stock, Etf, Bond],
                    &[Tlv],
                    percent(dec!(0.15), Some(ils(dec!(3.5)))),
                ),
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
                securities: vec![IndexFund],
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
            .about(&[IndexFund])
            .on(&[Tlv])
            .source(&fund_page),
        ],
    };

    let typical_offer = Plan {
        name: "Typical offer".into(),
        description: "What new customers are usually offered: 0.08% for Tel Aviv stocks, \
                      ETFs and bonds (at least ₪2.35) and index funds (at least ₪5), US \
                      stocks and ETFs for 1¢ a share (at least $7.50), no custody, and ₪15 \
                      a month, less that month's trade fees."
            .into(),
        trading: [
            vec![
                trade(
                    &[Stock, Etf, Bond],
                    &[Tlv],
                    percent(dec!(0.08), Some(ils(dec!(2.35)))),
                ),
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
        handling: Some(handling_less_trade_fees(dec!(15), 0)),
        fractions_on: vec![Usa],
        min_first_deposit: Some(ils(dec!(15000))),
        caveats: vec![
            typical_offer_caveat(
                "IBI",
                "gemeltop.co.il, tradingil.co.il",
                &[&on_gemeltop, &on_tradingil],
            ),
            Caveat::reading(
                "Bonds cost the offer's 0.08% (at least ₪2.35), and index funds 0.08% (at \
                 least ₪5), which is also the full tariff's price for them; gemeltop.co.il \
                 lists only stocks and ETFs.",
                "tradingil.co.il lists both for its joining offer (September 2026)",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond, IndexFund])
            .on(&[Tlv])
            .source(&on_tradingil),
            Caveat::reading(
                "1¢ a share, at least $7.50: gemeltop.co.il states only the minimum.",
                "the tariff's first US track is 1¢ a share with a minimum, and \
                 tradingil.co.il spells out 1¢ a share with a $7.50 minimum for its joining \
                 offer (September 2026)",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Stock, Etf])
            .on(&[Usa])
            .source(&on_gemeltop)
            .source(&on_tradingil),
            handling_less_trade_fees_caveat(
                "tradingil.co.il and broker.co.il say so for IBI's joining offer (2026)",
                &[&on_tradingil, &broker_co_il()],
            ),
            Caveat::at_most(
                "tradingil.co.il's joining offer adds two free years of the handling fee; \
                 gemeltop.co.il's doesn't, so it's charged from the first month.",
            )
            .about_fee(FeeKind::Handling)
            .source(&on_tradingil)
            .source(&on_gemeltop),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .about(&[Bond, IndexFund])
                .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .on(&[Europe]),
            no_custody_in_the_offer(
                "No custody: the offer's only holding cost is the monthly handling fee, as \
                 comparison sites list it. IBI's site states no custody on any fund.",
                &[&on_gemeltop, &on_tradingil, &fund_page],
            ),
            Caveat::not_counted("The ₪300 gift for opening an account isn't included.")
                .source(&on_gemeltop),
        ],
    };

    Broker {
        name: "IBI".into(),
        short_name: "IBI".into(),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 1, // Typical offer
        description: "An investment house's trading platform (IBI TRADE, with the IBI SMART \
                      app). It publishes only a full tariff of maximum prices; new customers \
                      are usually offered far less. Opening an account takes at least \
                      ₪15,000."
            .into(),
        tariff_date: Some(TariffDate::Day(date!(2026 - 07 - 01))),
        source_url: Some("https://campaign.ibi.co.il/PDF/TRADE/TAARIFON_IBI.pdf".into()),
        caveats: vec![
            Caveat::reading(
                "The conversion markup is 0.7%: IBI's site says it charges no fee, only a \
                 spread written in each customer's fee appendix, and works its example at \
                 0.7%.",
                "IBI's currency FAQ (2026), and tradingil.co.il's comparison of conversion \
                 costs (September 2026), which lists 0.7% below $15,000",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&currency_faq)
            .source(&tradingil_conversions()),
            Caveat::at_most(
                "Conversions of $15,000 or more get a 0.5% markup instead of 0.7%, per \
                 tradingil.co.il; 0.7% is used for all.",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .when_above(usd(dec!(15_000)))
            .source(&tradingil_conversions()),
            Caveat::published(
                "Managed (active) funds of 12 fund managers cost nothing to trade; index \
                 funds aren't among them.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Tlv])
            .source(&fund_page),
            Caveat::not_counted(
                "Foreign funds cost 0.275% plus the foreign broker's cost, which isn't \
                 included.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
            .on(&[Usa, Europe]),
        ],
        plans: vec![full_tariff, typical_offer],
    }
}

// ─────────────────────────── Interactive Israel ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn interactive() -> Broker {
    let tariff_url = "https://www.inter-il.com/wp-content/uploads/2026/09/\
                      %D7%AA%D7%A2%D7%A8%D7%99%D7%A4%D7%95%D7%9F-%D7%A2%D7%9E%D7%9C%D7%95%D7%AA-\
                      23.09.2026-1.pdf";
    let tariff = source("Tariff (PDF)", tariff_url);
    let commission_page = source(
        "Interactive's commission page",
        "https://www.inter-il.com/commission/",
    );
    let tel_aviv_page = source(
        "Interactive's Tel Aviv page",
        "https://www.inter-il.com/israeli-stock-market/",
    );
    let ibkr = source(
        "Interactive Brokers on spot currencies",
        "https://www.interactivebrokers.com/en/pricing/commissions-spot-currencies.php",
    );
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
            .on(&[Usa])
            .source(&tariff),
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
            .about(&[Bond])
            .source(&tariff),
            Caveat::not_counted(
                "Its tariff's price for mutual funds is unclear, so it isn't included.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund]),
            Caveat::published("Tel Aviv securities are traded for institutional clients only.")
                .about_fee(FeeKind::Trade)
                .on(&[Tlv])
                .source(&tel_aviv_page),
        ],
    };

    Broker {
        name: "Interactive Israel".into(),
        short_name: "Interactive".into(),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 0, // Standard
        description: "The Israeli introducing broker for Interactive Brokers, run by MEXEM \
                      (regulated in Cyprus, not by the Israel Securities Authority). US and \
                      European exchanges; Tel Aviv only for institutional clients."
            .into(),
        tariff_date: Some(TariffDate::Day(date!(2026 - 09 - 23))),
        source_url: Some(tariff_url.into()),
        caveats: vec![
            Caveat::published(
                "Converting is at the live market rate (שער\u{a0}רציף), as its site says, for \
                 up to ₪500,000: the fee is the whole cost the broker adds.",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&commission_page),
            Caveat::not_counted(
                "The currency market's own bid-ask spread, a few hundredths of a percent: \
                 Interactive Brokers, where the account is held, passes market quotes through \
                 and charges the fee instead of a markup. Its automatic conversions may move \
                 the rate by up to 0.03% instead.",
            )
            .on(&[Usa, Europe])
            .source(&ibkr),
            Caveat::reading(
                "Converting back to shekels is taken to cost the same as converting them: \
                 0.002%, at least ₪10.",
                "the tariff prices conversions by the currency traded, and its only note \
                 is on converting shekels; nothing prices the way back differently",
            )
            .about_fee(FeeKind::Conversion)
            .on(&[Usa, Europe])
            .source(&tariff),
        ],
        plans: vec![standard],
    }
}

// ─────────────────────────── Meitav Trade ───────────────────────────

#[must_use]
#[allow(clippy::too_many_lines, reason = "a tariff's data")]
pub fn meitav() -> Broker {
    let site = source(
        "Meitav Trade's site",
        "https://www.meitav.co.il/trade/independent_trading/",
    );
    let on_gemeltop = gemeltop("meitav-trade-amlot");
    let on_tradingil = tradingil("%D7%9E%D7%99%D7%98%D7%91-%D7%98%D7%A8%D7%99%D7%99%D7%93");
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
            &[IndexFund],
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
            .about_fee(FeeKind::Custody)
            .source(&exchange_calculator()),
            Caveat::not_counted(
                "Foreign funds add clearing and correspondent fees, which aren't included.",
            )
            .about_fee(FeeKind::Trade)
            .about(&[IndexFund])
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
                      0.08% for stocks and bonds (at least ₪4.65), US stocks and ETFs for \
                      1¢ a share (at least $5), no custody or conversion fee, and ₪15 a \
                      month after two free years, less that month's trade fees."
            .into(),
        trading: [
            vec![
                trade(&[Etf], &[Tlv], percent(dec!(0.07), Some(ils(dec!(4.65))))),
                trade(
                    &[Stock, Bond],
                    &[Tlv],
                    percent(dec!(0.08), Some(ils(dec!(4.65)))),
                ),
                tel_aviv(&[IndexFund]),
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
        handling: Some(handling_less_trade_fees(dec!(15), 24)),
        fractions_on: vec![],
        min_first_deposit: Some(ils(dec!(5000))),
        caveats: vec![
            typical_offer_caveat(
                "Meitav",
                "gemeltop.co.il, tradingil.co.il",
                &[&on_gemeltop, &on_tradingil],
            ),
            Caveat::published("No custody and no conversion fee, as Meitav's site states.")
                .about_fee(FeeKind::Custody)
                .source(&site),
            Caveat::reading(
                "Bonds cost the offer's 0.08%, at least ₪4.65; gemeltop.co.il lists only \
                 ETFs and stocks.",
                "tradingil.co.il lists bonds at 0.08% for its joining offer (September 2026)",
            )
            .about_fee(FeeKind::Trade)
            .about(&[Bond])
            .on(&[Tlv])
            .source(&on_tradingil),
            not_in_the_offer(
                "The offer doesn't mention index funds, so the full tariff's price is used. \
                 Managed (active) and money-market funds cost nothing to trade in it.",
            )
            .about(&[IndexFund])
            .on(&[Tlv]),
            handling_less_trade_fees_caveat(
                "broker.co.il says so for Meitav's joining offer (2026)",
                &[&broker_co_il()],
            ),
            Caveat::published("Some of its trading systems charge at least $7.50 instead of $5.")
                .about_fee(FeeKind::Trade)
                .about(&[Stock, Etf])
                .on(&[Usa])
                .source(&on_tradingil),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .about(&[Bond, IndexFund])
                .on(&[Usa]),
            not_in_the_offer("Not in the offer, so the full tariff's prices are used.")
                .on(&[Europe]),
            Caveat::not_counted("The ₪100 gift for opening an account isn't included.")
                .source(&on_gemeltop),
        ],
    };

    Broker {
        name: "Meitav Trade".into(),
        short_name: "Meitav".into(),
        kind: BrokerKind::InvestmentHouse,
        new_customer_plan: 1, // Typical offer
        description: "The largest exchange member that isn't a bank, part of the Meitav \
                      investment house. It publishes only a full tariff of maximum prices; \
                      new customers are usually offered far less."
            .into(),
        tariff_date: Some(TariffDate::Month(date!(2025 - 01 - 01))),
        source_url: Some("https://www.meitav.co.il/media/z2hkkhku/taarifon.pdf".into()),
        caveats: vec![
            Caveat::reading(
                "The tariff says the conversion markup is up to 0.7%; 2.1 agorot a dollar \
                 is charged, about 0.7% at ₪3 a dollar, so the full 0.7% is used.",
                "tradingil.co.il's comparison of conversion costs, September 2026",
            )
            .about_fee(FeeKind::Markup)
            .on(&[Usa, Europe])
            .source(&tradingil_conversions()),
        ],
        plans: vec![full_tariff, typical_offer],
    }
}
