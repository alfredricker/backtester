use std::collections::VecDeque;
use super::super::window::Window;
use super::WindowTracker;

// ============================================================================
// HISTORY TRACKER - For indicators that need full value history
// ============================================================================

/// Generic tracker that stores all values in a sliding window
/// Use this for complex indicators that need access to the full history
///
/// # Use Cases
/// - Pattern recognition
/// - Custom calculations that don't fit other tracker types
/// - Indicators that need to scan multiple values
///
#[derive(Debug, Clone)]
pub struct HistoryTracker {
    /// Deque of (timestamp, value) pairs
    values: VecDeque<(i64, f64)>,
    
    /// The time window to track
    window: Window,
}

impl HistoryTracker {
    /// Create a new HistoryTracker
    pub fn new(window: Window) -> Self {
        Self {
            values: VecDeque::new(),
            window,
        }
    }
    
    /// Get all values in the current window
    pub fn values(&self) -> &VecDeque<(i64, f64)> {
        &self.values
    }
    
    /// Get the number of values in the window
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    /// Check if the tracker is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    fn in_window(&self, current_timestamp: i64, check_timestamp: i64) -> bool {
        match self.window {
            Window::Bars(_) => true,
            _ => self.window.contains(current_timestamp, check_timestamp)
        }
    }
}

impl WindowTracker for HistoryTracker {
    fn push(&mut self, timestamp: i64, value: f64) {
        self.values.push_back((timestamp, value));
        
        if let Window::Bars(n) = self.window {
            while self.values.len() > n {
                self.values.pop_front();
            }
        }
    }
    
    fn get(&self) -> Option<f64> {
        // Returns the most recent value
        self.values.back().map(|(_, v)| *v)
    }
    
    fn prune(&mut self, current_timestamp: i64) {
        while let Some(&(timestamp, _)) = self.values.front() {
            if !self.in_window(current_timestamp, timestamp) {
                self.values.pop_front();
            } else {
                break;
            }
        }
    }
    
    fn clear(&mut self) {
        self.values.clear();
    }
}
