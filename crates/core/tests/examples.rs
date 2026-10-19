//! What to search for in the app (`examples.rs`): that every entry can be
//! found and traced to its source, that the bond held to a date is repaid by
//! it, and that the site's own words keep CLAUDE.md's Hebrew rules. The
//! names and numbers themselves come from the script; nothing here fixes one.

use std::collections::HashSet;

use broker_fees::examples::{
    self, BondSeries, Example, FindBy, NOT_A_PICK, Source, Venue, nearest_bond, nearest_makam,
};
use broker_fees::short_term::Pays;
use broker_fees::short_term::government_bonds::Linkage;
use broker_fees::short_term::money_market;
use broker_fees::ways::Way;
use broker_fees::{IntoEnumIterator, Lang, Named, Page, Product, Text};
use rust_decimal_macros::dec;
use time::{Date, Duration, Month};

fn first_of(year: i32, month: u8) -> Date {
    Date::from_calendar_date(year, Month::try_from(month).unwrap(), 1).unwrap()
}

/// The first of every month from the data's day to 2060: every visitor's
/// date the guide could ask about, and some past the last series.
fn every_month() -> impl Iterator<Item = Date> {
    let start = examples::checked();
    (start.year()..=2060)
        .flat_map(|year| (1..=12).map(move |month| first_of(year, month)))
        .filter(move |day| *day >= start)
}

fn every_page() -> Vec<Page> {
    let bonds = examples::government_bonds().iter().chain(examples::makam());
    bonds
        .map(BondSeries::page)
        .chain(examples::funds().iter().map(Example::page))
        .chain(
            examples::money_market_funds()
                .iter()
                .map(broker_fees::examples::MoneyMarketFund::page),
        )
        .collect()
}

// ─────────────────────────── Every entry can be found ───────────────────────────

#[test]
fn every_entry_has_a_number_or_a_ticker() {
    for bond in examples::government_bonds().iter().chain(examples::makam()) {
        assert!(bond.number.0 > 0, "{}", bond.name);
        assert!(!bond.name.trim().is_empty(), "{}", bond.number);
    }
    for fund in examples::funds() {
        assert_ne!(fund.name.trim(), "");
        match fund.find_by {
            FindBy::Number(number) => assert!(number.0 > 0, "{}", fund.name),
            FindBy::Ticker(ticker) => assert!(
                !ticker.is_empty() && ticker.chars().all(|c| c.is_ascii_uppercase()),
                "{}: {ticker}",
                fund.name
            ),
        }
    }
    for fund in examples::money_market_funds() {
        assert!(fund.number.0 > 0 && !fund.name.trim().is_empty());
    }
}

#[test]
fn every_entry_has_a_source_in_both_languages() {
    let pages = every_page();
    assert_ne!(pages, [] as [broker_fees::Page; 0]);
    for page in pages {
        assert!(page.url.starts_with("https://"), "{}", page.url);
        assert!(
            !page.name.en.is_empty() && !page.name.he.is_empty(),
            "{}",
            page.url
        );
    }
}

#[test]
fn a_source_names_the_entry_it_is_about() {
    for fund in examples::funds() {
        let url = fund.page().url;
        if let FindBy::Number(number) = fund.find_by {
            assert!(url.contains(&number.to_string()), "{}: {url}", fund.name);
        }
    }
    for bond in examples::government_bonds().iter().chain(examples::makam()) {
        assert!(bond.page().url.contains(&bond.number.to_string()));
    }
}

#[test]
fn numbers_are_found_in_tel_aviv_and_tickers_abroad() {
    for fund in examples::funds() {
        let in_tel_aviv = fund.venue == Venue::TelAviv;
        assert_eq!(
            in_tel_aviv,
            matches!(fund.find_by, FindBy::Number(_)),
            "{}",
            fund.name
        );
        let tel_aviv_source = matches!(
            fund.source,
            Source::Maya | Source::Funder | Source::TelAvivExchange
        );
        assert_eq!(in_tel_aviv, tel_aviv_source, "{}", fund.name);
    }
}

#[test]
fn no_entry_is_listed_twice() {
    let mut seen = HashSet::new();
    let numbers = examples::government_bonds()
        .iter()
        .chain(examples::makam())
        .map(|bond| bond.number)
        .chain(
            examples::money_market_funds()
                .iter()
                .map(|fund| fund.number),
        );
    for number in numbers {
        assert!(seen.insert(number), "{number} twice");
    }
    let mut finds = Vec::new();
    for fund in examples::funds() {
        assert!(!finds.contains(&fund.find_by), "{} twice", fund.name);
        finds.push(fund.find_by);
    }
}

#[test]
fn fees_are_plausible_shares_of_a_percent() {
    // A fee read into the wrong column (a 30% "up to" for a 0.3%) shows here.
    let fees = examples::funds()
        .iter()
        .flat_map(|fund| [fund.fee, fund.variable_fee_up_to])
        .flatten()
        .chain(examples::money_market_funds().iter().map(|fund| fund.fee));
    for fee in fees {
        assert!(fee.0 >= dec!(0) && fee.0 < dec!(1), "{fee:?}");
    }
}

// ─────────────────────────── The funds and the ways ───────────────────────────

#[test]
fn every_way_bought_on_an_exchange_has_an_example_there() {
    for way in Way::for_index() {
        let found: Vec<_> = examples::for_way(way).collect();
        match way.exchange {
            Some(exchange) => {
                assert!(!found.is_empty(), "{way:?} has no example");
                for fund in found {
                    assert_eq!(fund.product, way.product, "{}", fund.name);
                    assert_eq!(fund.venue.exchange(), exchange, "{}", fund.name);
                }
            }
            // A fund's track is chosen inside the fund, not searched for.
            None => assert!(found.is_empty(), "{way:?}"),
        }
    }
}

#[test]
fn the_irish_etfs_listed_in_tel_aviv_are_examples_of_their_own_product() {
    assert!(examples::of_product(Product::ForeignEtfInTelAviv).count() > 0);
    for fund in examples::of_product(Product::ForeignEtfInTelAviv) {
        assert_eq!(fund.venue, Venue::TelAviv);
    }
}

#[test]
fn the_money_market_funds_the_kind_names_are_listed_at_their_fees() {
    // `money_market.rs` names its cheapest and dearest fund in Hebrew, after
    // a colon; each must be a real fund on Maya at the fee the kind charges.
    let mut named = 0;
    for place in money_market::kind().places {
        let Some((_, name)) = place.name.he.split_once(": ") else {
            continue;
        };
        let name = name.replace('\u{a0}', " ");
        let fund = examples::money_market_fund(&name)
            .unwrap_or_else(|| panic!("{name} isn't on Maya's list"));
        let Pays::TheRateLess(fee) = place.pays else {
            panic!("{name} isn't paid as a fund");
        };
        assert_eq!(fund.fee, fee, "{name}");
        named += 1;
    }
    assert!(named > 0, "the kind names no fund");
}

// ─────────────────────────── The bond to hold to a date ───────────────────────────

#[test]
fn the_series_are_in_the_order_they_are_repaid() {
    for list in [examples::government_bonds(), examples::makam()] {
        assert!(list.windows(2).all(|pair| pair[0].repaid <= pair[1].repaid));
    }
}

#[test]
fn both_kinds_of_government_bond_are_listed() {
    for linkage in [Linkage::CpiLinked, Linkage::Unlinked] {
        assert!(
            examples::government_bonds()
                .iter()
                .any(|bond| bond.linkage == linkage)
        );
    }
    assert!(
        examples::makam()
            .iter()
            .all(|bill| bill.linkage == Linkage::Unlinked)
    );
}

#[test]
fn the_nearest_bond_is_repaid_by_the_date_and_is_the_last_that_is() {
    let today = examples::checked();
    for linkage in [Linkage::CpiLinked, Linkage::Unlinked] {
        let candidates: Vec<_> = examples::government_bonds()
            .iter()
            .filter(|bond| bond.linkage == linkage)
            .collect();
        for by in every_month() {
            let in_time = candidates
                .iter()
                .filter(|bond| today < bond.repaid && bond.repaid <= by);
            let latest = in_time.map(|bond| bond.repaid).max();
            match nearest_bond(linkage, today, by) {
                Some(bond) => {
                    assert_eq!(bond.linkage, linkage);
                    assert!(bond.repaid <= by, "{} is repaid after {by}", bond.name);
                    assert!(bond.repaid > today, "{} is already repaid", bond.name);
                    assert_eq!(
                        Some(bond.repaid),
                        latest,
                        "{by}: {} isn't the last in time",
                        bond.name
                    );
                }
                None => assert_eq!(latest, None, "{by}: a series is repaid in time"),
            }
        }
    }
}

#[test]
fn a_series_repaid_on_the_date_itself_is_in_time() {
    let today = examples::checked();
    let bond = examples::government_bonds().last().unwrap();
    assert_eq!(nearest_bond(bond.linkage, today, bond.repaid), Some(bond));
    let day_before = bond.repaid - Duration::days(1);
    assert_ne!(nearest_bond(bond.linkage, today, day_before), Some(bond));
}

#[test]
fn no_bond_before_the_first_is_repaid() {
    let today = examples::checked();
    for linkage in [Linkage::CpiLinked, Linkage::Unlinked] {
        let first = examples::government_bonds()
            .iter()
            .filter(|bond| bond.linkage == linkage && bond.repaid > today)
            .map(|bond| bond.repaid)
            .min()
            .unwrap();
        assert_eq!(
            nearest_bond(linkage, today, first - Duration::days(1)),
            None
        );
        assert_eq!(nearest_bond(linkage, today, today), None);
    }
}

#[test]
fn a_series_already_repaid_is_never_offered() {
    // Run as if it were the day after a series was repaid: the data ages.
    for bond in examples::government_bonds() {
        let found = nearest_bond(bond.linkage, bond.repaid, bond.repaid + Duration::days(1));
        assert!(
            found.is_none_or(|found| found.repaid > bond.repaid),
            "{}",
            bond.name
        );
    }
}

#[test]
fn the_nearest_bill_is_repaid_by_the_date() {
    let today = examples::checked();
    for by in every_month().take(24) {
        if let Some(bill) = nearest_makam(today, by) {
            assert!(today < bill.repaid && bill.repaid <= by, "{}", bill.name);
            let later_in_time = examples::makam()
                .iter()
                .any(|other| other.repaid > bill.repaid && other.repaid <= by);
            assert!(!later_in_time, "{by}: {} isn't the last in time", bill.name);
        }
    }
}

// ─────────────────────────── The site's own words ───────────────────────────

/// "מדד" followed by what names it, as CLAUDE.md's Hebrew rules ask.
fn names_its_index(text: &str) -> bool {
    text.match_indices("מדד").all(|(at, word)| {
        let rest = text[at + word.len()..].trim_start_matches([' ', '\u{a0}']);
        rest.starts_with("S&P")
            || rest.starts_with("המחירים לצרכן")
            || rest.starts_with("המחירים\u{a0}לצרכן")
    })
}

fn keeps_the_hebrew_rules(he: &str) {
    assert!(!he.contains('"'), "{he}: write ״, not \"");
    assert!(names_its_index(he), "{he}: a bare מדד");
}

#[test]
fn the_rule_catches_a_bare_index() {
    assert!(!names_its_index("קרן מחקה מדד מניות רחב"));
    assert!(names_its_index("מדד S&P 500, צמודה למדד המחירים לצרכן"));
}

#[test]
fn the_sites_own_words_keep_the_hebrew_rules() {
    // Trade names (the exchange's, Maya's) are the sources' and exempt: the
    // site writes them only as what to search for.
    let words: Vec<Text> = std::iter::once(NOT_A_PICK)
        .chain(every_page().into_iter().map(|page| page.name))
        .collect();
    for text in &words {
        keeps_the_hebrew_rules(&text.he);
        assert!(!text.en.is_empty());
    }
    for venue in Venue::iter() {
        keeps_the_hebrew_rules(venue.name(Lang::He));
        assert_ne!(venue.name(Lang::En), "");
    }
}

#[test]
fn the_example_mark_says_it_is_no_pick() {
    assert_eq!(NOT_A_PICK[Lang::He], *"דוגמה, לא המלצה");
}
