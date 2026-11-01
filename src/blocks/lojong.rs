//! Lojong: one of Atisha's 59 mind-training slogans, with its explanation and the point (of seven)
//! it belongs to. Offline: the slogans are compiled into the binary from `lojong.json`
//! (source: https://dx1.dev/lojong/).

use std::sync::OnceLock;

use anyhow::{Result, bail};
use chrono::{Datelike, NaiveDate};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::Deserialize;

use crate::doc::{Align, Doc, Style};

#[derive(Deserialize)]
struct Data {
    points: Vec<String>,
    slogans: Vec<Slogan>,
}

#[derive(Deserialize)]
struct Slogan {
    number: u8,
    point: u8,
    title: String,
    explanation: String,
}

fn data() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| serde_json::from_str(include_str!("lojong.json")).expect("lojong.json is valid"))
}

/// Slogans short enough to print at double size without becoming a tower of words.
const BIG_TITLE_MAX: usize = 48;

/// Which slogan to print.
/// - `number` (1 to 59) fixes it;
/// - `daily` gives the same slogan all day, a different one each day;
/// - otherwise random, or reproducible with `seed`.
pub fn build(today: NaiveDate, number: Option<u8>, daily: bool, seed: Option<u64>, explanation: bool) -> Result<Doc> {
    let data = data();
    let count = data.slogans.len();
    let index = match number {
        Some(n) if (1..=count).contains(&(n as usize)) => n as usize - 1,
        Some(n) => bail!("lojong: `number` must be between 1 and {count} (got {n})"),
        None => {
            let seed = if daily {
                today.num_days_from_ce() as u64
            } else {
                seed.unwrap_or_else(|| rand::rng().random_range(0..u64::MAX))
            };
            StdRng::seed_from_u64(seed).random_range(0..count)
        }
    };
    let slogan = &data.slogans[index];
    let point = data.points.get(slogan.point as usize - 1).map(String::as_str).unwrap_or("");

    let mut doc = Doc::new();
    doc.header("Lojong");
    doc.text(&format!("Point {} of 7: {point}", slogan.point), Style::default().small().center());
    doc.feed(1);
    let size = if slogan.title.chars().count() <= BIG_TITLE_MAX { 2 } else { 1 };
    doc.text(&slogan.title, Style::default().bold().center().size(size));
    if explanation {
        doc.feed(1);
        doc.rule('-');
        doc.text(&slogan.explanation, Style::default());
    }
    doc.feed(1);
    doc.text(&format!("Slogan {} of {count}", slogan.number), Style::default().small().align(Align::Right));
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 31).unwrap()
    }

    #[test]
    fn data_is_complete_and_in_order() {
        let d = data();
        assert_eq!(d.points.len(), 7);
        assert_eq!(d.slogans.len(), 59);
        for (i, s) in d.slogans.iter().enumerate() {
            assert_eq!(s.number as usize, i + 1);
            assert!((1..=7).contains(&s.point), "slogan {} has point {}", s.number, s.point);
            assert!(!s.title.is_empty() && !s.explanation.is_empty());
        }
        // Points never go backwards.
        assert!(d.slogans.windows(2).all(|w| w[0].point <= w[1].point));
    }

    #[test]
    fn fixed_number_prints_that_slogan() {
        let text = build(day(), Some(13), false, None, true).unwrap().preview(false);
        // Double-size text is previewed with spaced letters.
        assert!(text.contains("g r a t e f u l"));
        assert!(text.contains("Transforming Adversity"));
        assert!(text.contains("Slogan 13 of 59"));
    }

    #[test]
    fn out_of_range_number_is_an_error() {
        assert!(build(day(), Some(0), false, None, true).is_err());
        assert!(build(day(), Some(60), false, None, true).is_err());
    }

    #[test]
    fn same_seed_same_slogan_and_daily_is_stable() {
        let p = |doc: Doc| doc.preview(false);
        assert_eq!(
            p(build(day(), None, false, Some(7), true).unwrap()),
            p(build(day(), None, false, Some(7), true).unwrap())
        );
        assert_eq!(
            p(build(day(), None, true, None, true).unwrap()),
            p(build(day(), None, true, Some(1), true).unwrap())
        );
    }

    #[test]
    fn explanation_can_be_left_out() {
        let short = build(day(), Some(2), false, None, false).unwrap().preview(false);
        assert!(!short.contains("dreamlike"));
    }
}
