//! Scratch: prints answers' Hebrew (not committed).
use broker_fees::ExchangeRates;
use broker_fees::guide::*;
use rust_decimal_macros::dec;

fn main() {
    let mut settings = Settings::new(ExchangeRates::new(dec!(3.7), dec!(4.625)).unwrap());
    settings.today = Some(time::macros::date!(2026 - 10 - 01));
    let data = Data::listed();
    let which = std::env::args().nth(1).unwrap_or_default();
    let base = |now, monthly, months| Answers {
        situation: None,
        now,
        monthly,
        horizon: Horizon::Months(months),
        flexibility: None,
        cushion: None,
        needs_any_day: false,
        bank: Some(Bank::Hapoalim),
    };
    let mut cases: Vec<(String, Answers, Choices)> = vec![
        ("apartment".into(), base(dec!(100_000), dec!(2000), 36), Choices::default()),
        ("bonus".into(), base(dec!(15_000), dec!(0), 60), Choices::default()),
        ("salary".into(), base(dec!(0), dec!(500), 360), Choices::default()),
        (
            "salary-etf".into(),
            base(dec!(0), dec!(500), 360),
            Choices {
                wrapper: Some(Wrapper::Anyone),
                ..Choices::default()
            },
        ),
        (
            "bonus-deposit".into(),
            Answers {
                flexibility: Some(Flexibility::Fixed),
                ..base(dec!(15_000), dec!(0), 60)
            },
            Choices {
                safe_way: Some(SafeWay::Deposit),
                ..Choices::default()
            },
        ),
    ];
    for preset in broker_fees::situations::situations() {
        cases.push((preset.id.to_owned(), preset.answers.clone(), Choices::default()));
    }
    for (label, answers, choices) in cases {
        if !which.is_empty() && !label.starts_with(&which) {
            continue;
        }
        let answer = answer(&answers, &choices, &settings, &data).unwrap();
        let value = serde_json::to_value(&answer).unwrap();
        println!("=== {label}");
        if std::env::var("FULL").is_ok() {
            println!("{}", serde_json::to_string_pretty(&value).unwrap());
        } else {
            let mut out = Vec::new();
            he(&value, "", &mut out);
            for line in out {
                println!("{line}");
            }
        }
    }
}

fn he(value: &serde_json::Value, path: &str, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(text)) = map.get("he") {
                out.push(format!("{path}: {text}"));
                return;
            }
            for (key, field) in map {
                he(field, &format!("{path}.{key}"), out);
            }
        }
        serde_json::Value::Array(items) => {
            for (at, item) in items.iter().enumerate() {
                he(item, &format!("{path}[{at}]"), out);
            }
        }
        _ => {}
    }
}
