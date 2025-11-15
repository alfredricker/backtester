use std::collections::VecDeque;
use super::super::window::{Window, WindowConfig};
use super::WindowTracker;

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
    window: Window,
}

impl VarianceTracker {
    /// Create a new VarianceTracker
    /// 
    /// Accepts either `Window` or `WindowConfig` (from `.rounded()`)
    pub fn new(window: impl Into<WindowConfig>) -> Self {
        let config: WindowConfig = window.into();
        Self {
            values: VecDeque::new(),
            sum: 0.0,
            sum_sq_diff: 0.0,
            window: config.window,
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
        if let Window::Bars(n) = self.window {
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
