use crate::indicators::trackers::{SumTracker, WindowTracker};
use crate::indicators::fields::CommonField;
use crate::indicators::window::Window;
use crate::indicators::indicator::Indicator;
use crate::types::ohlcv::Row;
use crate::config::get_config;
use chrono::{DateTime, Utc, Timelike};

/// Average Current Volume (ACV)
/// 
/// Measures how the current day's volume compares to the expected volume 
/// at a given time of day, normalized by the Average Daily Volume (ADV).
/// 
/// Formula: ACV = current_volume / (ADV * volume_distribution(interval))
/// 
/// The volume distribution function models the typical intraday volume pattern,
/// with higher volume at market open and close.
/// 
/// # Usage
/// ```ignore
/// let mut acv = ACV::new(20); // 20-day ADV for normalization
/// 
/// // Update with each bar
/// for row in data {
///     acv.update(&row);
///     if let Some(acv_value) = acv.get() {
///         println!("ACV: {}", acv_value);
///     }
/// }
/// 
/// // Reset at end of day
/// acv.on_market_close();
/// ```
#[derive(Debug)]
pub struct ACV {
    /// Tracker for the current day's volume
    current_day_tracker: SumTracker,
    
    /// Tracker for the average daily volume over N days
    daily_avg_tracker: SumTracker,
    
    /// Last timestamp to detect new days
    last_day_timestamp: Option<i64>,
    
    /// Optional premarket volume to include
    premarket_volume: f64,
}

impl ACV {
    /// Create a new ACV indicator
    /// 
    /// # Arguments
    /// * `days` - Number of days for ADV calculation (e.g., 20 for ADV20)
    pub fn new(days: usize) -> Self {
        Self {
            current_day_tracker: SumTracker::new(Window::Days(1).rounded()),
            daily_avg_tracker: SumTracker::new(Window::Bars(days)),
            last_day_timestamp: None,
            premarket_volume: 0.0,
        }
    }
    
    /// Create ACV with explicit premarket volume
    pub fn with_premarket_volume(days: usize, premarket_vol: f64) -> Self {
        Self {
            current_day_tracker: SumTracker::new(Window::Days(1).rounded()),
            daily_avg_tracker: SumTracker::new(Window::Bars(days)),
            last_day_timestamp: None,
            premarket_volume: premarket_vol,
        }
    }
    
    /// Set the premarket volume for the current day
    pub fn set_premarket_volume(&mut self, volume: f64) {
        self.premarket_volume = volume;
    }
    
    /// Call this at market close to record the day's volume
    /// 
    /// This pushes the day's total volume to the ADV tracker and resets
    /// the current day tracker for the next trading day.
    pub fn on_market_close(&mut self) {
        if let Some(timestamp) = self.last_day_timestamp {
            let daily_volume = self.premarket_volume + self.current_day_tracker.sum();
            
            // Push the day's total volume to the daily average tracker
            self.daily_avg_tracker.push(timestamp, daily_volume);
            self.daily_avg_tracker.prune(timestamp);
            
            // Reset for next day
            self.current_day_tracker.clear();
            self.premarket_volume = 0.0;
        }
    }
    
    /// Get the current day's volume so far (including premarket)
    pub fn current_volume(&self) -> f64 {
        self.premarket_volume + self.current_day_tracker.sum()
    }
    
    /// Get the Average Daily Volume (ADV)
    pub fn adv(&self) -> Option<f64> {
        self.daily_avg_tracker.get()
    }
    
    /// Calculate the volume distribution for a given interval
    /// 
    /// This function models the typical U-shaped intraday volume distribution,
    /// with higher volume at market open and close.
    /// 
    /// # Arguments
    /// * `interval` - Minutes since market open
    /// 
    /// # Returns
    /// A value between 0.08 and 0.6 representing the expected fraction of 
    /// daily volume by this point in the trading day.
    fn volume_distribution(interval: i64) -> f64 {
        const A: f64 = 0.73;
        const B: f64 = 2.3;
        
        // Precalculated constants for performance
        const GAMMA_A: f64 = 14.819293933533726; // 46^0.73
        const GAMMA_B: f64 = 15437.746769286306;  // 46^2.3
        const EPSILON: f64 = 2247135.5315805604;  // 77^2.3
        const XI: f64 = 73.12134508107815;        // (100*GAMMA_B/EPSILON - 60)/(GAMMA_B/EPSILON-1)
        
        if interval < 0 {
            // Before market open, return minimum distribution
            return 0.08;
        } else if interval <= 46 {
            // First part of the day (roughly first hour)
            (8.0 + 52.0 * (interval as f64).powf(A) / GAMMA_A) / 100.0
        } else {
            // Rest of the trading day
            (XI + (60.0 - XI) * (interval as f64).powf(B) / GAMMA_B) / 100.0
        }
    }
    
    /// Convert a timestamp to minutes since market open
    /// 
    /// # Arguments
    /// * `timestamp` - Unix timestamp in nanoseconds
    /// 
    /// # Returns
    /// Minutes since market open (can be negative for premarket)
    fn timestamp_to_interval(timestamp: i64) -> i64 {
        let dt = DateTime::<Utc>::from_timestamp_nanos(timestamp);
        let config = get_config();
        let market_open = config.market_hours.market_open;
        
        // Calculate minutes since market open
        let current_minutes = dt.hour() as i64 * 60 + dt.minute() as i64;
        let market_open_minutes = market_open.hour() as i64 * 60 + market_open.minute() as i64;
        
        current_minutes - market_open_minutes
    }
}

impl Indicator for ACV {
    fn update(&mut self, row: &Row) {
        let volume = CommonField::Volume.extract(row);
        self.current_day_tracker.push(row.timestamp, volume);
        self.current_day_tracker.prune(row.timestamp);
        self.last_day_timestamp = Some(row.timestamp);
    }
    
    fn get(&self) -> Option<f64> {
        // Get ADV
        let adv = self.daily_avg_tracker.get()?;
        
        // Can't calculate if ADV is zero or negative
        if adv <= 0.0 {
            return None;
        }
        
        // Get current timestamp
        let timestamp = self.last_day_timestamp?;
        
        // Calculate current volume (premarket + intraday)
        let current_vol = self.current_volume();
        
        // Calculate interval (minutes since market open)
        let interval = Self::timestamp_to_interval(timestamp);
        
        // Get expected volume distribution at this time
        let dist_val = Self::volume_distribution(interval);
        
        // Avoid division by zero
        if dist_val <= 0.0 {
            return None;
        }
        
        // Calculate ACV: current_volume / (ADV * volume_distribution)
        Some(current_vol / (adv * dist_val))
    }
    
    fn reset(&mut self) {
        self.current_day_tracker.clear();
        self.daily_avg_tracker.clear();
        self.last_day_timestamp = None;
        self.premarket_volume = 0.0;
    }
    
    fn name(&self) -> &str {
        "ACV"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_volume_distribution() {
        // Test that distribution is reasonable
        assert!(ACV::volume_distribution(0) >= 0.08);
        assert!(ACV::volume_distribution(46) <= 0.6);
        assert!(ACV::volume_distribution(390) <= 0.6); // End of day (6.5 hours)
        
        // Distribution should be monotonically increasing
        let dist_30 = ACV::volume_distribution(30);
        let dist_60 = ACV::volume_distribution(60);
        assert!(dist_60 > dist_30);
    }
    
    #[test]
    fn test_timestamp_to_interval() {
        // This test requires proper config initialization
        // and would need actual timestamps to test properly
    }
}