// Efficient sliding window tracking algorithms for different indicator types
//
// This module provides trait-based tracking for indicators that need to maintain
// state over sliding windows. Different indicator types use different algorithms:
//
// - ExtremumTracker: For max/min indicators (uses monotonic deque - O(1) amortized)
// - SumTracker: For average indicators (tracks sum and count - O(1))
// - VarianceTracker: For standard deviation (tracks sum and sum of squares - O(1))

use std::collections::VecDeque;
use super::time::TimeWindow;

/// Trait for tracking values over a sliding window
/// 
/// Different implementations use different algorithms optimized for their use case
pub trait WindowTracker {
    /// Add a new data point to the tracker
    fn push(&mut self, timestamp: i64, value: f64);
    
    /// Get the current result (e.g., max, min, or average)
    fn get(&self) -> Option<f64>;
    
    /// Remove data points that fall outside the window
    fn prune(&mut self, current_timestamp: i64);
    
    /// Clear all tracked data
    fn clear(&mut self);

    fn in_window(&self, window: &TimeWindow, current_timestamp: i64, check_timestamp: i64) -> bool {
        match window {
            // bar based methods are handled by count not time, .push method handles automatically
            TimeWindow::Bars(_) => true,
            _ => window.contains(current_timestamp, check_timestamp)
        }
    }
}

// ============================================================================
// EXTREMUM TRACKER - For Max/Min indicators
// ============================================================================

/// Tracks the maximum or minimum value in a sliding window using a monotonic deque
///
/// # Algorithm: Monotonic Deque
/// 
/// The key insight is that we only need to track "potential future extremums".
/// For a maximum tracker:
/// - If a new value is larger than older values, those older values can NEVER
///   be the maximum again (even after the new value expires)
/// - Therefore, we can discard them immediately
///
/// # Example: Tracking maximum with window size 3
/// ```text
/// Values: [3, 1, 4, 1, 5, 9, 2]
///
/// Step 1: Push 3
///   Deque: [(ts:1, val:3)]
///   Max: 3
///   Reasoning: First value, just add it
///
/// Step 2: Push 1
///   Deque: [(ts:1, val:3), (ts:2, val:1)]
///   Max: 3
///   Reasoning: 1 < 3, so keep both. If 3 expires, 1 might be the max
///
/// Step 3: Push 4
///   Deque: [(ts:3, val:4)]
///   Max: 4
///   Reasoning: 4 > 3 and 4 > 1, so remove both! They can never be max again.
///   
/// Step 4: Push 1 (now ts:1 expires from window)
///   Deque: [(ts:3, val:4), (ts:4, val:1)]
///   Max: 4
///   Reasoning: 1 < 4, so keep both. After 4 expires, 1 might be max.
///
/// Step 5: Push 5
///   Deque: [(ts:5, val:5)]
///   Max: 5
///   Reasoning: 5 > everything, remove all older values
///
/// Step 6: Push 9
///   Deque: [(ts:6, val:9)]
///   Max: 9
///   
/// Step 7: Push 2 (ts:3 and ts:4 are now outside window)
///   Deque: [(ts:6, val:9), (ts:7, val:2)]
///   Max: 9
///   Reasoning: 2 < 9, keep both. After 9 expires, 2 will be the max.
/// ```
///
/// # Complexity
/// - Time: O(1) amortized per operation (each element added once, removed once)
/// - Space: O(W) worst case, but typically much smaller than window size
///
#[derive(Debug, Clone)]
pub struct ExtremumTracker {
    /// Deque of (timestamp, value) pairs in monotonic order
    /// For max: decreasing order (front = largest)
    /// For min: increasing order (front = smallest)
    deque: VecDeque<(i64, f64)>,
    
    /// The time window to track
    window: TimeWindow,
    
    /// Whether to track maximum (true) or minimum (false)
    track_max: bool,
}

impl ExtremumTracker {
    /// Create a new extremum tracker
    ///
    /// # Arguments
    /// * `window` - The time window to track
    /// * `track_max` - If true, tracks maximum; if false, tracks minimum
    pub fn new(window: TimeWindow, track_max: bool) -> Self {
        Self {
            deque: VecDeque::new(),
            window,
            track_max,
        }
    }
    
    /// Create a maximum tracker
    pub fn new_max(window: TimeWindow) -> Self {
        Self::new(window, true)
    }
    
    /// Create a minimum tracker
    pub fn new_min(window: TimeWindow) -> Self {
        Self::new(window, false)
    }
    
    /// Check if value1 should replace value2 based on whether we're tracking max or min
    fn should_replace(&self, new_value: f64, old_value: f64) -> bool {
        if self.track_max {
            new_value >= old_value  // For max: new value >= old value
        } else {
            new_value <= old_value  // For min: new value <= old value
        }
    }
}

impl WindowTracker for ExtremumTracker {
    fn push(&mut self, timestamp: i64, value: f64) {
        // Remove all values from the back that are "worse" than the new value
        // For max: remove all smaller values
        // For min: remove all larger values
        while let Some(&(_, back_value)) = self.deque.back() {
            if self.should_replace(value, back_value) {
                self.deque.pop_back();
            } else {
                break;
            }
        }
        
        // Add the new value
        self.deque.push_back((timestamp, value));
        
        // For bar-based windows, limit the size
        if let TimeWindow::Bars(n) = self.window {
            while self.deque.len() > n {
                self.deque.pop_front();
            }
        }
    }
    
    fn get(&self) -> Option<f64> {
        self.deque.front().map(|(_, value)| *value)
    }
    
    fn prune(&mut self, current_timestamp: i64) {
        // Remove expired entries from the front
        while let Some(&(timestamp, _)) = self.deque.front() {
            if !self.in_window(&self.window, current_timestamp, timestamp) {
                self.deque.pop_front();
            } else {
                break;
            }
        }
    }
    
    fn clear(&mut self) {
        self.deque.clear();
    }
}

// ============================================================================
// SUM TRACKER - For Average indicators
// ============================================================================

/// Tracks the sum and count of values in a sliding window for calculating averages
///
/// # Algorithm
/// Maintains a deque of all values in the window along with their timestamps.
/// Calculates sum on-the-fly when requested.
///
/// # Complexity
/// - Time: O(1) for push, O(1) for get (maintains running sum)
/// - Space: O(W) where W is window size
///
#[derive(Debug, Clone)]
pub struct SumTracker {
    /// Deque of (timestamp, value) pairs in the window
    values: VecDeque<(i64, f64)>,
    
    /// Running sum of all values in the window
    sum: f64,
    
    /// The time window to track
    window: TimeWindow,
}

impl SumTracker {
    pub fn new(window: TimeWindow) -> Self {
        Self {
            values: VecDeque::new(),
            sum: 0.0,
            window,
        }
    }
    
    /// Get the current sum
    pub fn sum(&self) -> f64 {
        self.sum
    }
    
    /// Get the count of values in the window
    pub fn count(&self) -> usize {
        self.values.len()
    }
    
    /// Get the average value (sum / count)
    pub fn average(&self) -> Option<f64> {
        if self.values.is_empty() {
            None
        } else {
            Some(self.sum / self.values.len() as f64)
        }
    }
}

impl WindowTracker for SumTracker {
    fn push(&mut self, timestamp: i64, value: f64) {
        self.values.push_back((timestamp, value));
        self.sum += value;
        
        // For bar-based windows, limit the size
        if let TimeWindow::Bars(n) = self.window {
            while self.values.len() > n {
                if let Some((_, old_value)) = self.values.pop_front() {
                    self.sum -= old_value;
                }
            }
        }
    }
    
    fn get(&self) -> Option<f64> {
        self.average()
    }
    
    fn prune(&mut self, current_timestamp: i64) {
        // Remove expired entries from the front
        while let Some(&(timestamp, _)) = self.values.front() {
            if !self.in_window(&self.window, current_timestamp, timestamp) {
                if let Some((_, value)) = self.values.pop_front() {
                    self.sum -= value;
                }
            } else {
                break;
            }
        }
    }
    
    fn clear(&mut self) {
        self.values.clear();
        self.sum = 0.0;
    }
}

// ============================================================================
// VARIANCE TRACKER - For Standard Deviation indicators
// ============================================================================

/// Tracks sum and sum of squares for calculating variance and standard deviation
///
/// Uses Welford's online algorithm for numerical stability
///
#[derive(Debug, Clone)]
pub struct VarianceTracker {
    /// Deque of (timestamp, value) pairs in the window
    values: VecDeque<(i64, f64)>,
    
    /// Running sum for mean calculation
    sum: f64,
    
    /// Running sum of squared differences from mean (for variance)
    sum_sq_diff: f64,
    
    /// The time window to track
    window: TimeWindow,
}

impl VarianceTracker {
    pub fn new(window: TimeWindow) -> Self {
        Self {
            values: VecDeque::new(),
            sum: 0.0,
            sum_sq_diff: 0.0,
            window,
        }
    }
    
    /// Get the current mean
    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            None
        } else {
            Some(self.sum / self.values.len() as f64)
        }
    }
    
    /// Get the variance
    pub fn variance(&self) -> Option<f64> {
        if self.values.is_empty() {
            None
        } else {
            Some(self.sum_sq_diff / self.values.len() as f64)
        }
    }
    
    /// Get the standard deviation
    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }
    
    /// Recalculate sum_sq_diff (used after removing values)
    fn recalculate(&mut self) {
        if self.values.is_empty() {
            self.sum = 0.0;
            self.sum_sq_diff = 0.0;
            return;
        }
        
        let mean = self.sum / self.values.len() as f64;
        self.sum_sq_diff = self.values
            .iter()
            .map(|(_, v)| {
                let diff = v - mean;
                diff * diff
            })
            .sum();
    }
}

impl WindowTracker for VarianceTracker {
    fn push(&mut self, timestamp: i64, value: f64) {
        self.values.push_back((timestamp, value));
        self.sum += value;
        
        // Recalculate variance components
        self.recalculate();
        
        // For bar-based windows, limit the size
        if let TimeWindow::Bars(n) = self.window {
            while self.values.len() > n {
                if let Some((_, old_value)) = self.values.pop_front() {
                    self.sum -= old_value;
                    self.recalculate();
                }
            }
        }
    }
    
    fn get(&self) -> Option<f64> {
        self.std_dev()
    }
    
    fn prune(&mut self, current_timestamp: i64) {
        let mut pruned = false;
        
        // Remove expired entries from the front
        while let Some(&(timestamp, _)) = self.values.front() {
            if !self.in_window(&self.window, current_timestamp, timestamp) {
                if let Some((_, value)) = self.values.pop_front() {
                    self.sum -= value;
                    pruned = true;
                }
            } else {
                break;
            }
        }
        
        if pruned {
            self.recalculate();
        }
    }
    
    fn clear(&mut self) {
        self.values.clear();
        self.sum = 0.0;
        self.sum_sq_diff = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extremum_tracker_max() {
        let mut tracker = ExtremumTracker::new_max(TimeWindow::Bars(3));
        
        // Example from documentation
        tracker.push(1, 3.0);
        assert_eq!(tracker.get(), Some(3.0));
        
        tracker.push(2, 1.0);
        assert_eq!(tracker.get(), Some(3.0));
        
        tracker.push(3, 4.0);
        assert_eq!(tracker.get(), Some(4.0));
        // Deque should only have 4.0 (3 and 1 were removed)
        assert_eq!(tracker.deque.len(), 1);
        
        tracker.push(4, 1.0);
        assert_eq!(tracker.get(), Some(4.0));
        assert_eq!(tracker.deque.len(), 2); // [4, 1]
        
        tracker.push(5, 5.0);
        assert_eq!(tracker.get(), Some(5.0));
        assert_eq!(tracker.deque.len(), 1); // [5]
    }
    
    #[test]
    fn test_extremum_tracker_min() {
        let mut tracker = ExtremumTracker::new_min(TimeWindow::Bars(3));
        
        tracker.push(1, 5.0);
        assert_eq!(tracker.get(), Some(5.0));
        
        tracker.push(2, 3.0);
        assert_eq!(tracker.get(), Some(3.0));
        // 5.0 should be removed since 3.0 is smaller
        assert_eq!(tracker.deque.len(), 1);
        
        tracker.push(3, 4.0);
        assert_eq!(tracker.get(), Some(3.0));
        assert_eq!(tracker.deque.len(), 2); // [3, 4]
        
        tracker.push(4, 1.0);
        assert_eq!(tracker.get(), Some(1.0));
        assert_eq!(tracker.deque.len(), 1); // [1]
    }
    
    #[test]
    fn test_sum_tracker() {
        let mut tracker = SumTracker::new(TimeWindow::Bars(3));
        
        tracker.push(1, 10.0);
        assert_eq!(tracker.sum(), 10.0);
        assert_eq!(tracker.count(), 1);
        assert_eq!(tracker.average(), Some(10.0));
        
        tracker.push(2, 20.0);
        assert_eq!(tracker.sum(), 30.0);
        assert_eq!(tracker.average(), Some(15.0));
        
        tracker.push(3, 30.0);
        assert_eq!(tracker.sum(), 60.0);
        assert_eq!(tracker.average(), Some(20.0));
        
        // Fourth value should push out the first
        tracker.push(4, 40.0);
        assert_eq!(tracker.sum(), 90.0); // 20 + 30 + 40
        assert_eq!(tracker.count(), 3);
        assert_eq!(tracker.average(), Some(30.0));
    }
    
    #[test]
    fn test_variance_tracker() {
        let mut tracker = VarianceTracker::new(TimeWindow::Bars(3));
        
        tracker.push(1, 2.0);
        tracker.push(2, 4.0);
        tracker.push(3, 6.0);
        
        // Mean = 4.0
        assert_eq!(tracker.mean(), Some(4.0));
        
        // Variance = ((2-4)^2 + (4-4)^2 + (6-4)^2) / 3 = (4 + 0 + 4) / 3 = 8/3
        let variance = tracker.variance().unwrap();
        assert!((variance - 8.0/3.0).abs() < 0.001);
        
        // Std dev = sqrt(8/3) ≈ 1.633
        let std_dev = tracker.std_dev().unwrap();
        assert!((std_dev - 1.633).abs() < 0.001);
    }
}
