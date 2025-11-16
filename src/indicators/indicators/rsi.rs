use crate::indicators::trackers::{ChangeTracker, WindowTracker};
use crate::indicators::fields::CommonField;
use crate::indicators::window::Window;
use crate::indicators::indicator::Indicator;
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
    pub fn new(window: Window, field: CommonField) -> Self {
        Self {
            tracker: ChangeTracker::absolute(window),
            field,
        }
    }
    
    /// Convenience constructor for close price RSI (most common)
    pub fn close(window: Window) -> Self {
        Self::new(window, CommonField::Close)
    }
}

impl Indicator for RSI {
    fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    /// Get the RSI value (0-100 scale)
    fn get(&self) -> Option<f64> {
        let avg_gain = self.tracker.average_gain();
        let avg_loss = self.tracker.average_loss();
        
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        
        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
    
    fn reset(&mut self) {
        self.tracker.clear();
    }
    
    fn name(&self) -> &str {
        "RSI"
    }
}