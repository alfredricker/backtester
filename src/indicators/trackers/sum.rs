use std::collections::VecDeque;
use super::super::window::Window;
use super::WindowTracker;

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
    window: Window,
}

impl SumTracker {
    /// Create a new SumTracker
    /// 
    /// # Examples
    /// ```
    /// // Without rounding
    /// let tracker = SumTracker::new(Window::Days(1));
    /// 
    /// // With rounding (aligns to market open)
    /// let tracker = SumTracker::new(Window::Days(1).rounded());
    /// ```
    pub fn new(window: Window) -> Self {
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
        if let Window::Bars(n) = self.window {
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
