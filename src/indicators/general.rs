//
use super::time::TimeWindow;
use super::tracker::{WindowTracker, ExtremumTracker, SumTracker};
use crate::types::ohlcv::{Row};

/// INDICATORS ARE SIMPLE STRUCTS THAT TRACK THE MINIMUM AMOUNT OF DATA

// GENERAL FIELDS THAT CAN BE USED FOR THE FOLLOWING INDICATORS
pub enum CommonField{
    Open,
    High,
    Low,
    Close,
    // as f64
    Volume,
    Median,
    Typical,
    WeightedClose,
}

impl CommonField {
    pub fn extract(&self, row: &Row) -> f64 {
        match self {
            CommonField::Open => row.open,
            CommonField::High => row.high,
            CommonField::Low => row.low,
            CommonField::Close => row.close,
            CommonField::Typical => (row.high + row.low + row.close) / 3.0,
            CommonField::WeightedClose => (row.high + row.low + row.close + row.close) / 4.0,
            CommonField::Median => (row.high + row.low) / 2.0,
            CommonField::Volume => row.volume as f64,
        }
    }
}

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