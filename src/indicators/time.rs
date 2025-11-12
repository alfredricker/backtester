// Arguments into indicators are usually time based.
// This file uses chrono to create enums and structs
// to pass to your indicators to determine time ranges in trading times
// that matches the data
// example: sma(TimeWindow::Minutes(5)) simple moving average of last 5 minutes

use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use crate::config::{get_config, MarketHours};

/// Represents different time windows for indicators
#[derive(Debug, Clone, Copy)]
pub enum TimeWindow {
    /// Number of minutes of data to look back
    Minutes(i64),
    /// Number of hours of data to look back
    Hours(i64),
    /// Number of trading days to look back
    Days(i64),
    /// Raw number of bars/candles to look back
    Bars(usize),
}

impl TimeWindow {
    /// Convert the time window to a Duration (for time-based windows)
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            TimeWindow::Minutes(m) => Some(Duration::minutes(*m)),
            TimeWindow::Hours(h) => Some(Duration::hours(*h)),
            TimeWindow::Days(d) => Some(Duration::days(*d)),
            TimeWindow::Bars(_) => None, // Bars are count-based, not time-based
        }
    }

    /// Get the number of bars if this is a Bars window
    pub fn to_bars(&self) -> Option<usize> {
        match self {
            TimeWindow::Bars(n) => Some(*n),
            _ => None,
        }
    }

    /// Check if a timestamp falls within this window from a given reference time
    pub fn contains(&self, reference_timestamp: i64, check_timestamp: i64) -> bool {
        if let Some(duration) = self.to_duration() {
            let reference = DateTime::<Utc>::from_timestamp_nanos(reference_timestamp);
            let check = DateTime::<Utc>::from_timestamp_nanos(check_timestamp);
            let cutoff = reference - duration;
            check >= cutoff && check <= reference
        } else {
            // For Bars-based windows, we can't determine this without the full dataset
            false
        }
    }

    /// Check if this window type supports rounding
    /// Rounding doesn't apply to Bars or time windows less than 10 minutes
    fn supports_rounding(&self) -> bool {
        match self {
            TimeWindow::Minutes(m) => *m >= 10,
            TimeWindow::Hours(_) | TimeWindow::Days(_) => true,
            TimeWindow::Bars(_) => false,
        }
    }

    /// Get the start time for this window from a given current time
    /// 
    /// # Arguments
    /// * `current_time` - The reference time (typically "now")
    /// * `round` - Whether to round the start time to clean boundaries
    /// * `market_hours` - Optional market hours config (uses global config if None)
    /// 
    /// # Examples
    /// ```
    /// // Window: 1 day, current: 2025-08-08 14:26:00
    /// // Without rounding: 2025-08-07 14:26:00
    /// // With rounding: 2025-08-08 00:00:00 (or 09:30:00 if no premarket)
    /// 
    /// // Window: 1 hour, current: 2025-08-08 14:26:00
    /// // Without rounding: 2025-08-08 13:26:00
    /// // With rounding: 2025-08-08 14:00:00
    /// ```
    pub fn get_start_time(
        &self,
        current_time: DateTime<Utc>,
        round: bool,
        market_hours: Option<&MarketHours>,
    ) -> DateTime<Utc> {
        // Get market hours from global config if not provided
        let config = get_config();
        let mh = market_hours.unwrap_or(&config.market_hours);

        match self {
            TimeWindow::Bars(_) => {
                // Bars don't have a time-based start, return current time
                current_time
            }
            TimeWindow::Minutes(m) => {
                if round && *m >= 10 {
                    // Round to the start of the current minute interval
                    self.round_to_minute_interval(current_time, *m)
                } else {
                    // No rounding: just go back m minutes
                    current_time - Duration::minutes(*m)
                }
            }
            TimeWindow::Hours(h) => {
                if round {
                    // Round to the start of the current hour
                    let rounded = current_time
                        .with_minute(0)
                        .and_then(|dt| dt.with_second(0))
                        .and_then(|dt| dt.with_nanosecond(0))
                        .unwrap_or(current_time);
                    rounded - Duration::hours(*h - 1) // Subtract (h-1) because we're already at the start of current hour
                } else {
                    // No rounding: just go back h hours
                    current_time - Duration::hours(*h)
                }
            }
            TimeWindow::Days(d) => {
                if round {
                    // Round to the start of the day, respecting market hours
                    self.round_to_day_start(current_time, *d, mh)
                } else {
                    // No rounding: just go back d days
                    current_time - Duration::days(*d)
                }
            }
        }
    }

    /// Round to the start of a minute interval (for windows >= 10 minutes)
    fn round_to_minute_interval(&self, current_time: DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
        let current_minute = current_time.minute() as i64;
        let rounded_minute = (current_minute / minutes) * minutes;
        
        current_time
            .with_minute(rounded_minute as u32)
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
            .unwrap_or(current_time)
    }

    /// Round to the start of the day, respecting market hours
    fn round_to_day_start(
        &self,
        current_time: DateTime<Utc>,
        days: i64,
        market_hours: &MarketHours,
    ) -> DateTime<Utc> {
        // Start at the beginning of the current day
        let start_of_day = current_time
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
            .unwrap_or(current_time);

        // If premarket is not included, adjust to market open time
        let adjusted_start = if !market_hours.include_premarket {
            let market_open = market_hours.market_open;
            start_of_day
                .with_hour(market_open.hour())
                .and_then(|dt| dt.with_minute(market_open.minute()))
                .and_then(|dt| dt.with_second(market_open.second()))
                .unwrap_or(start_of_day)
        } else {
            // If premarket is included, use premarket open time
            let premarket_open = market_hours.premarket_open;
            start_of_day
                .with_hour(premarket_open.hour())
                .and_then(|dt| dt.with_minute(premarket_open.minute()))
                .and_then(|dt| dt.with_second(premarket_open.second()))
                .unwrap_or(start_of_day)
        };

        // Go back (days - 1) because we're already at the start of the current day
        adjusted_start - Duration::days(days - 1)
    }
}

/// Central function to get start time for any window
/// This is a convenience function that uses the global config
pub fn get_start_time(
    window: TimeWindow,
    current_time: DateTime<Utc>,
    round: bool,
) -> DateTime<Utc> {
    window.get_start_time(current_time, round, None)
}

/// Get start time with custom market hours configuration
pub fn get_start_time_with_config(
    window: TimeWindow,
    current_time: DateTime<Utc>,
    round: bool,
    market_hours: &MarketHours,
) -> DateTime<Utc> {
    window.get_start_time(current_time, round, Some(market_hours))
}

/// Represents specific times of day for auction-based exits/entries
#[derive(Debug, Clone, Copy)]
pub struct TimeOfDay {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl TimeOfDay {
    pub fn new(hour: u32, minute: u32, second: u32) -> Self {
        TimeOfDay { hour, minute, second }
    }

    /// Market open (9:30:00 ET)
    pub const MARKET_OPEN: TimeOfDay = TimeOfDay { hour: 9, minute: 30, second: 0 };
    
    /// Market close (16:00:00 ET)
    pub const MARKET_CLOSE: TimeOfDay = TimeOfDay { hour: 16, minute: 0, second: 0 };
    
    /// Pre-market close (9:29:59 ET)
    pub const PRE_MARKET_CLOSE: TimeOfDay = TimeOfDay { hour: 9, minute: 29, second: 59 };
    
    /// Last minute of trading (15:59:59 ET)
    pub const LAST_MINUTE: TimeOfDay = TimeOfDay { hour: 15, minute: 59, second: 59 };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, MarketHours, init_config};
    use chrono::NaiveTime;

    fn setup_test_config() {
        let config = Config {
            market_hours: MarketHours {
                include_premarket: false,
                include_postmarket: false,
                market_open: NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                market_close: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                premarket_open: NaiveTime::from_hms_opt(4, 0, 0).unwrap(),
                postmarket_close: NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
            },
        };
        init_config(config);
    }

    #[test]
    fn test_time_window_duration() {
        let window = TimeWindow::Minutes(5);
        assert_eq!(window.to_duration(), Some(Duration::minutes(5)));
        
        let window = TimeWindow::Bars(10);
        assert_eq!(window.to_duration(), None);
    }

    #[test]
    fn test_supports_rounding() {
        assert!(!TimeWindow::Minutes(5).supports_rounding());
        assert!(TimeWindow::Minutes(10).supports_rounding());
        assert!(TimeWindow::Minutes(30).supports_rounding());
        assert!(TimeWindow::Hours(1).supports_rounding());
        assert!(TimeWindow::Days(1).supports_rounding());
        assert!(!TimeWindow::Bars(20).supports_rounding());
    }

    #[test]
    fn test_hour_rounding() {
        setup_test_config();
        
        // Current time: 2025-08-08 14:26:35
        let current = DateTime::parse_from_rfc3339("2025-08-08T14:26:35Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let window = TimeWindow::Hours(1);
        
        // Without rounding: should be 13:26:35
        let start_no_round = window.get_start_time(current, false, None);
        assert_eq!(start_no_round.hour(), 13);
        assert_eq!(start_no_round.minute(), 26);
        assert_eq!(start_no_round.second(), 35);
        
        // With rounding: should be 14:00:00
        let start_round = window.get_start_time(current, true, None);
        assert_eq!(start_round.hour(), 14);
        assert_eq!(start_round.minute(), 0);
        assert_eq!(start_round.second(), 0);
    }

    #[test]
    fn test_day_rounding_without_premarket() {
        setup_test_config();
        
        // Current time: 2025-08-08 14:26:00
        let current = DateTime::parse_from_rfc3339("2025-08-08T14:26:00Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let window = TimeWindow::Days(1);
        
        // Without rounding: should be 2025-08-07 14:26:00
        let start_no_round = window.get_start_time(current, false, None);
        assert_eq!(start_no_round.day(), 7);
        assert_eq!(start_no_round.hour(), 14);
        assert_eq!(start_no_round.minute(), 26);
        
        // With rounding (no premarket): should be 2025-08-08 09:30:00
        let start_round = window.get_start_time(current, true, None);
        assert_eq!(start_round.day(), 8);
        assert_eq!(start_round.hour(), 9);
        assert_eq!(start_round.minute(), 30);
        assert_eq!(start_round.second(), 0);
    }

    #[test]
    fn test_day_rounding_with_premarket() {
        // Setup config with premarket
        let config = Config {
            market_hours: MarketHours {
                include_premarket: true,
                include_postmarket: false,
                market_open: NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                market_close: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                premarket_open: NaiveTime::from_hms_opt(4, 0, 0).unwrap(),
                postmarket_close: NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
            },
        };
        
        // Current time: 2025-08-08 14:26:00
        let current = DateTime::parse_from_rfc3339("2025-08-08T14:26:00Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let window = TimeWindow::Days(1);
        
        // With rounding (with premarket): should be 2025-08-08 04:00:00
        let start_round = window.get_start_time(current, true, Some(&config.market_hours));
        assert_eq!(start_round.day(), 8);
        assert_eq!(start_round.hour(), 4);
        assert_eq!(start_round.minute(), 0);
        assert_eq!(start_round.second(), 0);
    }

    #[test]
    fn test_minute_rounding() {
        setup_test_config();
        
        // Current time: 2025-08-08 14:26:35
        let current = DateTime::parse_from_rfc3339("2025-08-08T14:26:35Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let window = TimeWindow::Minutes(15);
        
        // Without rounding: should be 14:11:35 (15 minutes back)
        let start_no_round = window.get_start_time(current, false, None);
        assert_eq!(start_no_round.hour(), 14);
        assert_eq!(start_no_round.minute(), 11);
        assert_eq!(start_no_round.second(), 35);
        
        // With rounding: should be 14:15:00 (rounded to 15-minute interval)
        let start_round = window.get_start_time(current, true, None);
        assert_eq!(start_round.hour(), 14);
        assert_eq!(start_round.minute(), 15);
        assert_eq!(start_round.second(), 0);
    }

    #[test]
    fn test_central_get_start_time_function() {
        setup_test_config();
        
        let current = DateTime::parse_from_rfc3339("2025-08-08T14:26:00Z")
            .unwrap()
            .with_timezone(&Utc);
        
        // Test using the central function
        let start = get_start_time(TimeWindow::Hours(2), current, true);
        assert_eq!(start.hour(), 14);
        assert_eq!(start.minute(), 0);
    }
}
