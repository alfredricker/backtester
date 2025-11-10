// Arguments into indicators are usually time based.
// This file uses chrono to create enums and structs
// to pass to your indicators to determine time ranges in trading times
// that matches the data
// example: sma(TimeWindow::Minutes(5)) simple moving average of last 5 minutes

use chrono::{DateTime, Duration, Utc};

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

    #[test]
    fn test_time_window_duration() {
        let window = TimeWindow::Minutes(5);
        assert_eq!(window.to_duration(), Some(Duration::minutes(5)));
        
        let window = TimeWindow::Bars(10);
        assert_eq!(window.to_duration(), None);
    }
}
