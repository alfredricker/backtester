
use super::trackers::{ChangeTracker, WindowTracker};
use super::fields::CommonField;
use super::time::TimeWindow;
use crate::types::ohlcv::Row;

/// Relative Strength Index (RSI)
/// Momentum indicator comparing magnitude of recent gains to recent losses
///
/// RSI = 100 - (100 / (1 + RS))
/// where RS = Average Gain / Average Loss
#[derive(Debug)]
pub struct RSI {
    tracker: ChangeTracker,
    field: CommonField,
}

impl RSI {
    pub fn new(window: TimeWindow, field: CommonField) -> Self {
        Self {
            tracker: ChangeTracker::absolute(window),
            field,
        }
    }
    
    /// Convenience constructor for close price RSI (most common)
    pub fn close(window: TimeWindow) -> Self {
        Self::new(window, CommonField::Close)
    }
    
    pub fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    /// Get the RSI value (0-100 scale)
    pub fn get(&self) -> Option<f64> {
        let avg_gain = self.tracker.average_gain();
        let avg_loss = self.tracker.average_loss();
        
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        
        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
    
    pub fn reset(&mut self) {
        self.tracker.clear();
    }
}