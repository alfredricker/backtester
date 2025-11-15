use crate::indicators::trackers::{SumTracker, WindowTracker};
use crate::indicators::fields::CommonField;
use crate::indicators::window::Window;
use crate::types::ohlcv::Row;

/// Average Daily Volume (ADV)
/// 
/// Tracks the average daily volume over a specified number of days.
/// 
/// # Usage
/// Call `update()` with each bar during the day, then call `on_market_close()` 
/// at the end of each trading day to record that day's total volume.
#[derive(Debug)]
pub struct ADV {
    /// Tracker for the average of daily volumes (each "bar" is one day's total)
    daily_avg_tracker: SumTracker,
    
    /// Tracker for accumulating volume during the current day
    current_day_tracker: SumTracker,
    
    /// Last timestamp to detect new days
    last_day_timestamp: Option<i64>,
}

impl ADV {
    /// Create a new ADV indicator
    /// 
    /// # Arguments
    /// * `days` - Number of days to average over (e.g., 20 for ADV20)
    pub fn new(days: usize) -> Self {
        Self {
            daily_avg_tracker: SumTracker::new(Window::Bars(days)),
            // Use .rounded() to align the day tracker to market open
            current_day_tracker: SumTracker::new(Window::Days(1).rounded()),
            last_day_timestamp: None,
        }
    }

    /// Update with a new bar of data
    pub fn update(&mut self, row: &Row) {
        let volume = CommonField::Volume.extract(row);
        self.current_day_tracker.push(row.timestamp, volume);
        self.current_day_tracker.prune(row.timestamp);
        self.last_day_timestamp = Some(row.timestamp);
    }
    
    /// Call this at market close to record the day's volume
    /// 
    /// This pushes the day's total volume to the multi-day tracker.
    /// You should call this once per day after the market closes.
    pub fn on_market_close(&mut self) {
        if let Some(timestamp) = self.last_day_timestamp {
            let daily_volume = self.current_day_tracker.sum();
            
            // Push the day's total volume to the daily average tracker
            self.daily_avg_tracker.push(timestamp, daily_volume);
            self.daily_avg_tracker.prune(timestamp);
            
            // Reset the current day tracker for tomorrow
            self.current_day_tracker.clear();
        }
    }

    /// Get the average daily volume
    /// 
    /// Returns None if there aren't enough days of data yet
    pub fn get(&self) -> Option<f64> {
        self.daily_avg_tracker.get()
    }
    
    /// Get the current day's volume so far (not the average)
    pub fn current_day_volume(&self) -> f64 {
        self.current_day_tracker.sum()
    }
    
    /// Reset the indicator
    pub fn reset(&mut self) {
        self.daily_avg_tracker.clear();
        self.current_day_tracker.clear();
        self.last_day_timestamp = None;
    }
}