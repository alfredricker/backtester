use crate::indicators::window::Window;
use crate::indicators::trackers::{WindowTracker, ExtremumTracker};
use crate::indicators::fields::CommonField;
use crate::indicators::indicator::Indicator;
use crate::types::ohlcv::Row;

/// INDICATORS ARE SIMPLE STRUCTS THAT TRACK THE MINIMUM AMOUNT OF DATA

#[derive(Debug)]
pub struct HighOfPeriod {
    tracker: ExtremumTracker,
    field: CommonField,
}

impl HighOfPeriod {
    pub fn new(window: Window, field: CommonField) -> Self {
        Self {
            tracker: ExtremumTracker::new_max(window),
            field,
        }
    }
}

impl Indicator for HighOfPeriod {
    fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    fn get(&self) -> Option<f64> {
        self.tracker.get()
    }
    
    fn reset(&mut self) {
        self.tracker.clear();
    }
    
    fn name(&self) -> &str {
        "High of Period"
    }
}

/// Low of Day (LOD) - Tracks the lowest price over a time window
#[derive(Debug)]
pub struct LowOfPeriod {
    tracker: ExtremumTracker,
    field: CommonField,
}

impl LowOfPeriod {
    pub fn new(window: Window, field: CommonField) -> Self {
        Self {
            tracker: ExtremumTracker::new_min(window),
            field,
        }
    }
}

impl Indicator for LowOfPeriod {
    fn update(&mut self, row: &Row) {
        let value = self.field.extract(row);
        self.tracker.push(row.timestamp, value);
        self.tracker.prune(row.timestamp);
    }
    
    fn get(&self) -> Option<f64> {
        self.tracker.get()
    }
    
    fn reset(&mut self) {
        self.tracker.clear();
    }
    
    fn name(&self) -> &str {
        "Low of Period"
    }
}