// VOLUME BASED INDICATORS
use crate::indicators::trackers::{SumTracker, WindowTracker};
use crate::indicators::fields::{PriceField, CommonField};
use crate::indicators::window::Window;
use crate::indicators::indicator::Indicator;
use crate::types::ohlcv::Row;

#[derive(Debug)]
pub struct VWAP {
    pv_tracker: SumTracker,     // PRICE × VOLUME TRACKER
    volume_tracker: SumTracker,  // VOLUME TRACKER
    price_field: PriceField,
}

impl VWAP {
    pub fn new(window: Window, price_field: Option<PriceField>) -> Self {
        let pf = price_field.unwrap_or(PriceField::Typical);
        Self {
            pv_tracker: SumTracker::new(window),
            volume_tracker: SumTracker::new(window),
            price_field: pf,
        }
    }
}

impl Indicator for VWAP {
    fn update(&mut self, row: &Row) {
        let price = self.price_field.extract(row);
        let volume: f64 = CommonField::Volume.extract(row);

        self.pv_tracker.push(row.timestamp, price * volume);
        self.pv_tracker.prune(row.timestamp);
        
        self.volume_tracker.push(row.timestamp, volume);
        self.volume_tracker.prune(row.timestamp);
    }

    fn get(&self) -> Option<f64> {
        let total_volume = self.volume_tracker.sum();
        if total_volume == 0.0 {
            None
        } else {
            Some(self.pv_tracker.sum() / total_volume)
        }
    }

    fn reset(&mut self) {
        self.pv_tracker.clear();
        self.volume_tracker.clear();
    }
    
    fn name(&self) -> &str {
        "VWAP"
    }
}
