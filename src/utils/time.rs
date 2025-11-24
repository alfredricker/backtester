use crate::config::get_config;
use chrono::DateTime;

fn apply_time_to_timestamp(timestamp: i64, target_time: chrono::NaiveTime) -> i64 {
    let dt = DateTime::from_timestamp(timestamp, 0).unwrap();
    let date = dt.date_naive();
    
    date.and_time(target_time)
        .and_utc()
        .timestamp()
}

/// Get end of day timestamp (market close)
pub fn get_mc_timestamp(timestamp: i64) -> i64 {
    let config = get_config();
    apply_time_to_timestamp(timestamp, config.market_hours.market_close)
}

/// Get start of day timestamp (market open)
pub fn get_mo_timestamp(timestamp: i64) -> i64 {
    let config = get_config();
    apply_time_to_timestamp(timestamp, config.market_hours.market_open)
}

/// Get start premarket timestamp of trading day
pub fn get_pmo_timestamp(timestamp: i64) -> i64 {
    let config = get_config();
    apply_time_to_timestamp(timestamp, config.market_hours.premarket_open)
}

/// Get end postmarket timestamp of trading day
pub fn get_pmc_timestamp(timestamp: i64) -> i64 {
    let config = get_config();
    apply_time_to_timestamp(timestamp, config.market_hours.postmarket_close)
}