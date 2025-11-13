//
use super::time::TimeWindow;
use super::tracker::{WindowTracker, ExtremumTracker, SumTracker, ChangeTracker};
use super::enums::CommonField;
use crate::types::ohlcv::{Row};

/// INDICATORS ARE SIMPLE STRUCTS THAT TRACK THE MINIMUM AMOUNT OF DATA

pub struct HighOfPeriod {
    tracker: ExtremumTracker,
    field: CommonField,
}

impl HighOfPeriod {
    pub fn new(window: TimeWindow, field: CommonField) -> Self {
        Self {
            tracker: ExtremumTracker::new_max(window),
            field,
        }
    }
    
    /// Update with a new data point
    pub fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    /// Get the current maximum value
    pub fn get(&self) -> Option<f64> {
        self.tracker.get()
    }
    
    /// Reset the indicator
    pub fn reset(&mut self) {
        self.tracker.clear();
    }
}

/// Low of Day (LOD) - Tracks the lowest price over a time window
pub struct LowOfPeriod {
    tracker: ExtremumTracker,
    field: CommonField,
}

impl LowOfPeriod {
    pub fn new(window: TimeWindow, field: CommonField) -> Self {
        Self {
            tracker: ExtremumTracker::new_min(window),
            field,
        }
    }
    
    pub fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    pub fn get(&self) -> Option<f64> {
        self.tracker.get()
    }
    
    pub fn reset(&mut self) {
        self.tracker.clear();
    }
}

/// Moving Average using stateful tracking
pub struct MovingAverage {
    tracker: SumTracker,
    field: CommonField,
}

impl MovingAverage {
    pub fn new(window: TimeWindow, field: CommonField) -> Self {
        Self {
            tracker: SumTracker::new(window),
            field,
        }
    }
    
    pub fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    pub fn get(&self) -> Option<f64> {
        self.tracker.get()
    }
    
    pub fn reset(&mut self) {
        self.tracker.clear();
    }
}

/// Relative Strength Index (RSI)
/// Momentum indicator comparing magnitude of recent gains to recent losses
///
/// RSI = 100 - (100 / (1 + RS))
/// where RS = Average Gain / Average Loss
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