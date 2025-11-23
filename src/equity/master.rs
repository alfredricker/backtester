use std::collections::HashMap;
use crate::types::ohlcv::Row;
use crate::strategy::Strategy;
use crate::position::condition::Conditionable;

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

pub trait StrategyRunner {
    fn update(&mut self, row: &Row);
}

impl<L: Conditionable, R: Conditionable> StrategyRunner for Strategy<L, R> {
    fn update(&mut self, row: &Row) {
        self.update(row);
    }
}

/// Data tracked for a single ticker
struct TickerData {
    /// The strategy instance being tracked for this ticker
    strategy: Box<dyn StrategyRunner>,
    /// Last timestamp processed (for validation)
    last_timestamp: Option<i64>,
}

impl TickerData {
    fn new(strategy: Box<dyn StrategyRunner>) -> Self {
        Self {
            strategy,
            last_timestamp: None,
        }
    }
    
    /// Update strategy with a new row, validating timestamp order
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
        
        // Update strategy (which updates indicators)
        self.strategy.update(row);
        
        // Store this timestamp for next validation
        self.last_timestamp = Some(row.timestamp);
        
        Ok(())
    }
}

/// The master data storage tracking indicators for all tickers
pub struct EquityTracker {
    /// Map of ticker symbol to ticker data
    tickers: HashMap<String, TickerData>,
    /// Factory to create new strategy instances for new tickers
    factory: Box<dyn Fn() -> Box<dyn StrategyRunner>>,
}

impl EquityTracker {
    /// Create a new equity tracker with the given strategy factory
    pub fn new(factory: Box<dyn Fn() -> Box<dyn StrategyRunner>>) -> Self {
        Self {
            tickers: HashMap::new(),
            factory,
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
                let strategy = (self.factory)();
                TickerData::new(strategy)
            });
        
        // Update with validation
        ticker_data.update(row)
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
    
    /// Get the last processed timestamp for a ticker
    pub fn last_timestamp(&self, ticker: &str) -> Option<i64> {
        self.tickers.get(ticker).and_then(|data| data.last_timestamp)
    }
}
