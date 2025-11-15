use std::collections::VecDeque;
use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use crate::config::{get_config, MarketHours};

/// Represents different time windows for indicators
#[derive(Debug, Clone, Copy)]
pub enum Window {
    /// Number of minutes of data to look back
    Minutes(i64),
    /// Number of hours of data to look back
    Hours(i64),
    /// Number of hours, rounded to start of hour
    HoursRounded(i64),
    /// Number of trading days to look back
    Days(i64),
    /// Number of trading days, rounded to market open
    DaysRounded(i64),
    /// Raw number of bars/candles to look back
    Bars(usize),
}

impl Window {
    /// Convert the time window to a Duration (for time-based windows)
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            Window::Minutes(m) => Some(Duration::minutes(*m)),
            Window::Hours(h) | Window::HoursRounded(h) => Some(Duration::hours(*h)),
            Window::Days(d) | Window::DaysRounded(d) => Some(Duration::days(*d)),
            Window::Bars(_) => None, // Bars are count-based, not time-based
        }
    }

    /// Get the number of bars if this is a Bars window
    pub fn to_bars(&self) -> Option<usize> {
        match self {
            Window::Bars(n) => Some(*n),
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

    /// Create a rounded version of this window
    /// 
    /// Rounding behavior:
    /// - Hours: Round to the start of the current hour
    /// - Days: Round to the start of the day (respecting market hours)
    /// - Minutes/Bars: Returns self unchanged (rounding not applicable)
    /// 
    /// # Examples
    /// ```
    /// let window = Window::Days(1).rounded();      // Returns Window::DaysRounded(1)
    /// let window = Window::Hours(3).rounded();     // Returns Window::HoursRounded(3)
    /// let window = Window::Minutes(15).rounded();  // Returns Window::Minutes(15) unchanged
    /// ```
    pub fn rounded(self) -> Window {
        match self {
            Window::Hours(h) => Window::HoursRounded(h),
            Window::Days(d) => Window::DaysRounded(d),
            other => other, // Minutes, Bars, and already-rounded windows stay the same
        }
    }

    /// Get the start time for this window from a given current time
    /// 
    /// Handles rounding automatically for HoursRounded and DaysRounded variants
    pub fn get_start_time(&self, current_time: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Window::Bars(_) => current_time,
            Window::Minutes(m) => current_time - Duration::minutes(*m),
            Window::Hours(h) => current_time - Duration::hours(*h),
            Window::Days(d) => current_time - Duration::days(*d),
            Window::HoursRounded(h) => {
                // Round to the start of the current hour
                let rounded = current_time
                    .with_minute(0)
                    .and_then(|dt| dt.with_second(0))
                    .and_then(|dt| dt.with_nanosecond(0))
                    .unwrap_or(current_time);
                rounded - Duration::hours(*h - 1)
            }
            Window::DaysRounded(d) => {
                let config = get_config();
                Self::round_to_day_start(current_time, *d, &config.market_hours)
            }
        }
    }

    /// Round to the start of the day, respecting market hours
    fn round_to_day_start(
        current_time: DateTime<Utc>,
        days: i64,
        market_hours: &MarketHours,
    ) -> DateTime<Utc> {
        let start_of_day = current_time
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
            .unwrap_or(current_time);

        let adjusted_start = if !market_hours.include_premarket {
            let market_open = market_hours.market_open;
            start_of_day
                .with_hour(market_open.hour())
                .and_then(|dt| dt.with_minute(market_open.minute()))
                .and_then(|dt| dt.with_second(market_open.second()))
                .unwrap_or(start_of_day)
        } else {
            let premarket_open = market_hours.premarket_open;
            start_of_day
                .with_hour(premarket_open.hour())
                .and_then(|dt| dt.with_minute(premarket_open.minute()))
                .and_then(|dt| dt.with_second(premarket_open.second()))
                .unwrap_or(start_of_day)
        };

        adjusted_start - Duration::days(days - 1)
    }
}