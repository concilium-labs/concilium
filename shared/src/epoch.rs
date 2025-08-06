use chrono::{DateTime, NaiveDateTime, Utc};

pub fn current_epoch_number() -> i64 {
    let start_naive = NaiveDateTime::parse_from_str("2009-01-03 21:45:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let start_timestamp = DateTime::<Utc>::from_naive_utc_and_offset(start_naive, Utc).timestamp();

    ((Utc::now().timestamp() - start_timestamp) / 12) + 1
}

pub fn timestamp_to_epoch_number(timestamp: i64) -> i64 {
    let start_naive = NaiveDateTime::parse_from_str("2009-01-03 21:45:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let start_timestamp = DateTime::<Utc>::from_naive_utc_and_offset(start_naive, Utc).timestamp();

    ((timestamp - start_timestamp) / 12) + 1
} 