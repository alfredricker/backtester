//
use super::time::TimeWindow;
use super::trackers::{WindowTracker, ExtremumTracker, SumTracker, ChangeTracker};
use super::fields::CommonField;
use crate::types::ohlcv::{Row};

/// INDICATORS ARE SIMPLE STRUCTS THAT TRACK THE MINIMUM AMOUNT OF DATA

#[derive(Debug)]
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
#[derive(Debug)]
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