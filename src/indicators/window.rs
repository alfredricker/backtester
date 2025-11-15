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
    /// Number of trading days to look back
    Days(i64),
    /// Raw number of bars/candles to look back
    Bars(usize),
}

/// Configuration for a time window with optional rounding
#[derive(Debug, Clone, Copy)]
pub struct WindowConfig {
    pub window: Window,
    pub round: bool,
}

impl From<Window> for WindowConfig {
    fn from(window: Window) -> Self {
        WindowConfig {
            window,
            round: false, // default: no rounding
        }
    }
}

impl Window {
    /// Convert the time window to a Duration (for time-based windows)
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            Window::Minutes(m) => Some(Duration::minutes(*m)),
            Window::Hours(h) => Some(Duration::hours(*h)),
            Window::Days(d) => Some(Duration::days(*d)),
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

    /// Create a config with rounding enabled
    /// 
    /// Rounding behavior:
    /// - Hours: Round to the start of the current hour
    /// - Days: Round to the start of the day (respecting market hours)
    /// - Minutes/Bars: Rounding has no effect
    pub fn rounded(self) -> WindowConfig {
        WindowConfig {
            window: self,
            round: true,
        }
    }
    
    /// Create a config with rounding disabled (default)
    pub fn unrounded(self) -> WindowConfig {
        WindowConfig {
            window: self,
            round: false,
        }
    }

    /// Get the start time for this window from a given current time (no rounding)
    /// 
    /// For rounding support, use `WindowConfig` with `.rounded()`:
    /// ```
    /// let config = Window::Days(1).rounded();
    /// let start = config.get_start_time(current_time, None);
    /// ```
    pub fn get_start_time(
        &self,
        current_time: DateTime<Utc>,
    ) -> DateTime<Utc> {
        match self {
            Window::Bars(_) => {
                // Bars don't have a time-based start, return current time
                current_time
            }
            Window::Minutes(m) => current_time - Duration::minutes(*m),
            Window::Hours(h) => current_time - Duration::hours(*h),
            Window::Days(d) => current_time - Duration::days(*d),
        }
    }
}

impl WindowConfig {
    /// Get the start time for this window config from a given current time
    /// 
    /// Uses the global market hours configuration by default.
    /// 
    /// # Examples
    /// ```
    /// // Window: 1 day, current: 2025-08-08 14:26:00
    /// // Without rounding: 2025-08-07 14:26:00
    /// let start = Window::Days(1).get_start_time(current);
    /// 
    /// // With rounding: 2025-08-08 09:30:00 (respects market hours)
    /// let start = Window::Days(1).rounded().get_start_time(current);
    /// 
    /// // Window: 1 hour, current: 2025-08-08 14:26:00
    /// // Without rounding: 2025-08-08 13:26:00
    /// let start = Window::Hours(1).get_start_time(current);
    /// 
    /// // With rounding: 2025-08-08 14:00:00
    /// let start = Window::Hours(1).rounded().get_start_time(current);
    /// ```
    pub fn get_start_time(&self, current_time: DateTime<Utc>) -> DateTime<Utc> {
        if !self.round {
            return self.window.get_start_time(current_time);
        }

        // Get market hours from global config
        let config = get_config();
        let mh = &config.market_hours;

        match self.window {
            Window::Bars(_) | Window::Minutes(_) => {
                // Bars and Minutes don't support rounding
                self.window.get_start_time(current_time)
            }
            Window::Hours(h) => {
                // Round to the start of the current hour
                let rounded = current_time
                    .with_minute(0)
                    .and_then(|dt| dt.with_second(0))
                    .and_then(|dt| dt.with_nanosecond(0))
                    .unwrap_or(current_time);
                rounded - Duration::hours(h - 1) // Subtract (h-1) because we're already at the start of current hour
            }
            Window::Days(d) => {
                // Round to the start of the day, respecting market hours
                Self::round_to_day_start(current_time, d, mh)
            }
        }
    }

    /// Round to the start of the day, respecting market hours
    fn round_to_day_start(
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