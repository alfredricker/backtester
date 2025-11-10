use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Row {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub ticker: String,
}

pub enum OHLCV {
    Open,
    High,
    Low,
    Close,
    Volume,
}

impl Row {
    /// Convert a timestamp (nanoseconds) to a DateTime<Utc>
    pub fn timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_nanos(timestamp)
    }

    /// Get this row's timestamp as a DateTime
    pub fn datetime(&self) -> DateTime<Utc> {
        Self::timestamp_to_datetime(self.timestamp)
    }

    /// Get a specific OHLCV field value
    pub fn get_field(&self, field: OHLCV) -> f64 {
        match field {
            OHLCV::Open => self.open,
            OHLCV::High => self.high,
            OHLCV::Low => self.low,
            OHLCV::Close => self.close,
            OHLCV::Volume => self.volume as f64,
        }
    }

    /// Get typical price: (High + Low + Close) / 3
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Get weighted close: (High + Low + Close + Close) / 4
    pub fn weighted_close(&self) -> f64 {
        (self.high + self.low + self.close + self.close) / 4.0
    }

    /// Get median price: (High + Low) / 2
    pub fn median_price(&self) -> f64 {
        (self.high + self.low) / 2.0
    }

    /// Get the true range compared to a previous row
    /// Used in ATR calculations
    pub fn true_range(&self, previous: &Row) -> f64 {
        let h_l = self.high - self.low;
        let h_pc = (self.high - previous.close).abs();
        let l_pc = (self.low - previous.close).abs();
        h_l.max(h_pc).max(l_pc)
    }
}

/// Helper methods for working with slices of Row data
pub trait DataWindow {
    /// Filter rows within a time window from a reference timestamp
    fn filter_by_time_window(&self, reference_timestamp: i64, window_nanos: i64) -> Vec<&Row>;
    
    /// Get the last N bars
    fn last_n_bars(&self, n: usize) -> &[Row];
    
    /// Check if data has enough bars for a given window size
    fn has_enough_bars(&self, required: usize) -> bool;
}

impl DataWindow for [Row] {
    fn filter_by_time_window(&self, reference_timestamp: i64, window_nanos: i64) -> Vec<&Row> {
        let cutoff = reference_timestamp - window_nanos;
        self.iter()
            .filter(|row| row.timestamp >= cutoff && row.timestamp <= reference_timestamp)
            .collect()
    }
    
    fn last_n_bars(&self, n: usize) -> &[Row] {
        let start = if self.len() > n {
            self.len() - n
        } else {
            0
        };
        &self[start..]
    }
    
    fn has_enough_bars(&self, required: usize) -> bool {
        self.len() >= required
    }
}

impl OHLCV {
    // const &'static references the same piece of memory on the stackt throughout program lifetime
    // useful for looping over values:
    // for field in OHLCV::ALL
    pub const ALL: &'static [OHLCV] = &[OHLCV::Open, OHLCV::High, OHLCV::Low, OHLCV::Close, OHLCV::Volume];
}