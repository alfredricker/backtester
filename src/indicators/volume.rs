// VOLUME BASED INDICATORS
use super::tracker::{SumTracker, WindowTracker};
use super::enums::{PriceField, CommonField};
use super::time::TimeWindow;
use crate::types::ohlcv::Row;

pub struct VWAP {
    pv_tracker: SumTracker,     // PRICE × VOLUME TRACKER
    volume_tracker: SumTracker,  // VOLUME TRACKER
    price_field: PriceField,
}

impl VWAP {
    pub fn new(window: TimeWindow, price_field: Option<PriceField>) -> Self {
        let pf = price_field.unwrap_or(PriceField::Typical);
        Self {
            pv_tracker: SumTracker::new(window),
            volume_tracker: SumTracker::new(window),
            price_field: pf,
        }
    }

    pub fn update(&mut self, row: &Row) {
        let price = self.price_field.extract(row);
        let volume: f64 = CommonField::Volume.extract(row);

        self.pv_tracker.push(row.timestamp, price * volume);
        self.pv_tracker.prune(row.timestamp);
        
        self.volume_tracker.push(row.timestamp, volume);
        self.volume_tracker.prune(row.timestamp);
    }

    pub fn get(&self) -> Option<f64> {
        let total_volume = self.volume_tracker.sum();
        if total_volume == 0.0 {
            None
        } else {
            Some(self.pv_tracker.sum() / total_volume)
        }
    }

    pub fn reset(&mut self) {
        self.pv_tracker.clear();
        self.volume_tracker.clear();
    }
}
