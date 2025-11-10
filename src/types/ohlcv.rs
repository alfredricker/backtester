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

impl Row{
    pub fn timestamp_to_datetime(timestamp: i64)->DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_nanos(i64_value);
    }

    pub fn get_field(&self, field: OHLCV) -> f64 {
        match field {
            OHLCV::Open => self.open,
            OHLCV::High => self.high,
            OHLCV::Low => self.low,
            OHLCV::Close => self.close,
            OHLCV::Volume => self.volume as f64,
        }
    }
}

impl OHLCV {
    // const &'static references the same piece of memory on the stackt throughout program lifetime
    // useful for looping over values:
    // for field in OHLCV::ALL
    pub const ALL: &'static [OHLCV] = &[OHLCV::Open, OHLCV::High, OHLCV::Low, OHLCV::Close, OHLCV::Volume];
}