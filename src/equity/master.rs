use std::collections::HashMap;
use crate::types::ohlcv::Row;
use crate::indicators::indicator::Indicator;
use crate::config::IndicatorConfig;

/// Errors that can occur during equity tracking
#[derive(Debug, thiserror::Error)]
pub enum EquityTrackerError {
    #[error("Timestamp order violation for ticker {ticker}: current={current}, previous={previous}")]
    TimestampOrderViolation {
        ticker: String,
        current: i64,
        previous: i64,
    },
}

/// Data tracked for a single ticker
#[derive(Debug)]
struct TickerData {
    /// All indicators being tracked for this ticker
    indicators: Vec<Indicator>,
    /// Last timestamp processed (for validation)
    last_timestamp: Option<i64>,
}

impl TickerData {
    fn new(indicators: Vec<Indicator>) -> Self {
        Self {
            indicators,
            last_timestamp: None,
        }
    }
    
    /// Update all indicators with a new row, validating timestamp order
    fn update(&mut self, row: &Row) -> Result<(), EquityTrackerError> {
        // Validate timestamp is monotonically increasing
        if let Some(prev_ts) = self.last_timestamp {
            if row.timestamp <= prev_ts {
                return Err(EquityTrackerError::TimestampOrderViolation {
                    ticker: row.ticker.clone(),
                    current: row.timestamp,
                    previous: prev_ts,
                });
            }
        }
        
        // Update all indicators
        for indicator in &mut self.indicators {
            indicator.update(row);
        }
        
        // Store this timestamp for next validation
        self.last_timestamp = Some(row.timestamp);
        
        Ok(())
    }
    
    /// Get a reference to all indicators
    fn indicators(&self) -> &[Indicator] {
        &self.indicators
    }
    
    /// Reset all indicators and timestamp tracking
    fn reset(&mut self) {
        for indicator in &mut self.indicators {
            indicator.reset();
        }
        self.last_timestamp = None;
    }
}

/// The master data storage tracking indicators for all tickers
/// 
/// This struct maintains a HashMap where:
/// - Key: Ticker symbol (String)
/// - Value: TickerData containing indicators and metadata
/// 
/// # Features
/// - Automatic ticker initialization on first row
/// - Timestamp validation (must be monotonically increasing per ticker)
/// - Indicator updates are applied to all configured indicators
/// 
/// # Example
/// ```rust
/// let config = Config::default();
/// let mut tracker = EquityTracker::new(&config.indicator_config);
/// 
/// for row in data_rows {
///     tracker.process_row(&row)?;
///     
///     if let Some(indicators) = tracker.get_indicators(&row.ticker) {
///         for indicator in indicators {
///             if let Some(value) = indicator.get() {
///                 println!("{}: {}", indicator.name(), value);
///             }
///         }
///     }
/// }
/// ```
pub struct EquityTracker {
    /// Map of ticker symbol to ticker data
    tickers: HashMap<String, TickerData>,
    /// Configuration for creating new indicators
    indicator_config: IndicatorConfig,
}

impl EquityTracker {
    /// Create a new equity tracker with the given indicator configuration
    pub fn new(indicator_config: &IndicatorConfig) -> Self {
        Self {
            tickers: HashMap::new(),
            indicator_config: indicator_config.clone(),
        }
    }
    
    /// Process a single row of data
    /// 
    /// Automatically initializes ticker data if this is the first row for the ticker.
    /// Validates timestamp ordering and updates all indicators.
    /// 
    /// # Errors
    /// Returns `EquityTrackerError::TimestampOrderViolation` if the timestamp is not
    /// greater than the previous timestamp for this ticker.
    pub fn process_row(&mut self, row: &Row) -> Result<(), EquityTrackerError> {
        // Get or create ticker data
        let ticker_data = self.tickers
            .entry(row.ticker.clone())
            .or_insert_with(|| {
                let indicators = self.indicator_config.create_indicators();
                TickerData::new(indicators)
            });
        
        // Update with validation
        ticker_data.update(row)
    }
    
    /// Get the indicators for a specific ticker
    pub fn get_indicators(&self, ticker: &str) -> Option<&[Indicator]> {
        self.tickers.get(ticker).map(|data| data.indicators())
    }
    
    /// Get mutable access to indicators for a specific ticker
    pub fn get_indicators_mut(&mut self, ticker: &str) -> Option<&mut Vec<Indicator>> {
        self.tickers.get_mut(ticker).map(|data| &mut data.indicators)
    }
    
    /// Check if a ticker is being tracked
    pub fn has_ticker(&self, ticker: &str) -> bool {
        self.tickers.contains_key(ticker)
    }
    
    /// Get the number of tickers being tracked
    pub fn ticker_count(&self) -> usize {
        self.tickers.len()
    }
    
    /// Get all ticker symbols being tracked
    pub fn tickers(&self) -> Vec<String> {
        self.tickers.keys().cloned().collect()
    }
    
    /// Reset all indicators for a specific ticker
    pub fn reset_ticker(&mut self, ticker: &str) {
        if let Some(data) = self.tickers.get_mut(ticker) {
            data.reset();
        }
    }
    
    /// Reset all indicators for all tickers
    pub fn reset_all(&mut self) {
        for data in self.tickers.values_mut() {
            data.reset();
        }
    }
    
    /// Clear all ticker data
    pub fn clear(&mut self) {
        self.tickers.clear();
    }
    
    /// Get the last processed timestamp for a ticker
    pub fn last_timestamp(&self, ticker: &str) -> Option<i64> {
        self.tickers.get(ticker).and_then(|data| data.last_timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndicatorSpec;
    use crate::indicators::{time::TimeWindow, fields::CommonField};

    fn create_test_row(ticker: &str, timestamp: i64, close: f64) -> Row {
        Row {
            timestamp,
            ticker: ticker.to_string(),
            open: close - 1.0,
            high: close + 1.0,
            low: close - 2.0,
            close,
            volume: 1000000,
        }
    }

    #[test]
    fn test_new_tracker() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![
                IndicatorSpec::MovingAverage {
                    window: TimeWindow::Bars(3),
                    field: CommonField::Close,
                },
            ],
        };
        let tracker = EquityTracker::new(&config);
        assert_eq!(tracker.ticker_count(), 0);
    }

    #[test]
    fn test_process_single_ticker() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![
                IndicatorSpec::MovingAverage {
                    window: TimeWindow::Bars(3),
                    field: CommonField::Close,
                },
            ],
        };
        let mut tracker = EquityTracker::new(&config);

        let row1 = create_test_row("AAPL", 1000, 100.0);
        tracker.process_row(&row1).unwrap();

        assert_eq!(tracker.ticker_count(), 1);
        assert!(tracker.has_ticker("AAPL"));
        assert_eq!(tracker.last_timestamp("AAPL"), Some(1000));
    }

    #[test]
    fn test_process_multiple_tickers() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![
                IndicatorSpec::RSI {
                    window: TimeWindow::Bars(2),
                    field: CommonField::Close,
                },
            ],
        };
        let mut tracker = EquityTracker::new(&config);

        tracker.process_row(&create_test_row("AAPL", 1000, 100.0)).unwrap();
        tracker.process_row(&create_test_row("TSLA", 1000, 200.0)).unwrap();
        tracker.process_row(&create_test_row("AAPL", 2000, 105.0)).unwrap();

        assert_eq!(tracker.ticker_count(), 2);
        assert_eq!(tracker.last_timestamp("AAPL"), Some(2000));
        assert_eq!(tracker.last_timestamp("TSLA"), Some(1000));
    }

    #[test]
    fn test_timestamp_validation() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![],
        };
        let mut tracker = EquityTracker::new(&config);

        // First row succeeds
        tracker.process_row(&create_test_row("AAPL", 1000, 100.0)).unwrap();

        // Second row with later timestamp succeeds
        tracker.process_row(&create_test_row("AAPL", 2000, 105.0)).unwrap();

        // Third row with earlier timestamp fails
        let result = tracker.process_row(&create_test_row("AAPL", 1500, 102.0));
        assert!(result.is_err());

        if let Err(EquityTrackerError::TimestampOrderViolation { ticker, current, previous }) = result {
            assert_eq!(ticker, "AAPL");
            assert_eq!(current, 1500);
            assert_eq!(previous, 2000);
        } else {
            panic!("Expected TimestampOrderViolation error");
        }
    }

    #[test]
    fn test_timestamp_validation_per_ticker() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![],
        };
        let mut tracker = EquityTracker::new(&config);

        // Each ticker has independent timestamp tracking
        tracker.process_row(&create_test_row("AAPL", 1000, 100.0)).unwrap();
        tracker.process_row(&create_test_row("TSLA", 500, 200.0)).unwrap();  // Earlier timestamp is OK
        tracker.process_row(&create_test_row("AAPL", 2000, 105.0)).unwrap();
        tracker.process_row(&create_test_row("TSLA", 1000, 205.0)).unwrap();

        assert_eq!(tracker.last_timestamp("AAPL"), Some(2000));
        assert_eq!(tracker.last_timestamp("TSLA"), Some(1000));
    }

    #[test]
    fn test_indicator_updates() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![
                IndicatorSpec::MovingAverage {
                    window: TimeWindow::Bars(2),
                    field: CommonField::Close,
                },
            ],
        };
        let mut tracker = EquityTracker::new(&config);

        tracker.process_row(&create_test_row("AAPL", 1000, 100.0)).unwrap();
        tracker.process_row(&create_test_row("AAPL", 2000, 200.0)).unwrap();

        let indicators = tracker.get_indicators("AAPL").unwrap();
        assert_eq!(indicators.len(), 1);
        
        // Moving average of 100 and 200 is 150
        let ma_value = indicators[0].get();
        assert!(ma_value.is_some());
        assert!((ma_value.unwrap() - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_reset_ticker() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![
                IndicatorSpec::MovingAverage {
                    window: TimeWindow::Bars(2),
                    field: CommonField::Close,
                },
            ],
        };
        let mut tracker = EquityTracker::new(&config);

        tracker.process_row(&create_test_row("AAPL", 1000, 100.0)).unwrap();
        tracker.process_row(&create_test_row("AAPL", 2000, 200.0)).unwrap();

        tracker.reset_ticker("AAPL");

        // After reset, should accept earlier timestamp
        tracker.process_row(&create_test_row("AAPL", 500, 50.0)).unwrap();
        assert_eq!(tracker.last_timestamp("AAPL"), Some(500));
    }

    #[test]
    fn test_clear() {
        let config = IndicatorConfig {
            enabled: true,
            specs: vec![],
        };
        let mut tracker = EquityTracker::new(&config);

        tracker.process_row(&create_test_row("AAPL", 1000, 100.0)).unwrap();
        tracker.process_row(&create_test_row("TSLA", 1000, 200.0)).unwrap();

        assert_eq!(tracker.ticker_count(), 2);

        tracker.clear();
        assert_eq!(tracker.ticker_count(), 0);
        assert!(!tracker.has_ticker("AAPL"));
        assert!(!tracker.has_ticker("TSLA"));
    }

    #[test]
    fn test_disabled_indicators() {
        let config = IndicatorConfig {
            enabled: false,
            specs: vec![
                IndicatorSpec::MovingAverage {
                    window: TimeWindow::Bars(2),
                    field: CommonField::Close,
                },
            ],
        };
        let mut tracker = EquityTracker::new(&config);

        tracker.process_row(&create_test_row("AAPL", 1000, 100.0)).unwrap();

        let indicators = tracker.get_indicators("AAPL").unwrap();
        assert_eq!(indicators.len(), 0);  // No indicators created when disabled
    }
}