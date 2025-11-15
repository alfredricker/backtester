use std::collections::VecDeque;
use super::super::window::{Window, WindowConfig};
use super::WindowTracker;

// ============================================================================
// CHANGE TRACKER - For tracking changes between consecutive values
// ============================================================================

/// Tracks changes between consecutive values over a sliding window
///
/// This is a flexible tracker that stores the change (delta) between each value
/// and the previous value. Useful for building momentum indicators like RSI, ROC, etc.
///
/// # Use Cases
/// - RSI: Track gains and losses
/// - Rate of Change (ROC): Track percentage changes
/// - Momentum indicators: Track value deltas
///
#[derive(Debug, Clone)]
pub struct ChangeTracker {
    /// Deque of (timestamp, change) pairs
    changes: VecDeque<(i64, f64)>,
    
    /// Previous value for calculating change
    prev_value: Option<f64>,
    
    /// The time window to track
    window: Window,
    
    /// Whether to track absolute or percentage change
    use_percentage: bool,
}

impl ChangeTracker {
    /// Create a new change tracker
    ///
    /// Accepts either `Window` or `WindowConfig` (from `.rounded()`)
    ///
    /// # Arguments
    /// * `window` - The time window to track
    /// * `use_percentage` - If true, calculates percentage change: (new - old) / old * 100
    ///                       If false, calculates absolute change: (new - old)
    pub fn new(window: impl Into<WindowConfig>, use_percentage: bool) -> Self {
        let config: WindowConfig = window.into();
        Self {
            changes: VecDeque::new(),
            prev_value: None,
            window: config.window,
            use_percentage,
        }
    }
    
    /// Create a tracker for absolute changes
    pub fn absolute(window: impl Into<WindowConfig>) -> Self {
        Self::new(window, false)
    }
    
    /// Create a tracker for percentage changes
    pub fn percentage(window: impl Into<WindowConfig>) -> Self {
        Self::new(window, true)
    }
    
    /// Get all changes in the current window
    pub fn changes(&self) -> &VecDeque<(i64, f64)> {
        &self.changes
    }
    
    /// Get the sum of all changes
    pub fn sum(&self) -> f64 {
        self.changes.iter().map(|(_, change)| change).sum()
    }
    
    /// Get the average change
    pub fn average(&self) -> Option<f64> {
        if self.changes.is_empty() {
            None
        } else {
            Some(self.sum() / self.changes.len() as f64)
        }
    }
    
    /// Get the sum of positive changes (gains)
    pub fn sum_gains(&self) -> f64 {
        self.changes
            .iter()
            .map(|(_, change)| if *change > 0.0 { *change } else { 0.0 })
            .sum()
    }
    
    /// Get the sum of negative changes (losses, as positive value)
    pub fn sum_losses(&self) -> f64 {
        self.changes
            .iter()
            .map(|(_, change)| if *change < 0.0 { -change } else { 0.0 })
            .sum()
    }
    
    /// Get the average gain
    pub fn average_gain(&self) -> f64 {
        if self.changes.is_empty() {
            0.0
        } else {
            self.sum_gains() / self.changes.len() as f64
        }
    }
    
    /// Get the average loss
    pub fn average_loss(&self) -> f64 {
        if self.changes.is_empty() {
            0.0
        } else {
            self.sum_losses() / self.changes.len() as f64
        }
    }
    
    fn in_window(&self, current_timestamp: i64, check_timestamp: i64) -> bool {
        match self.window {
            Window::Bars(_) => true,
            _ => self.window.contains(current_timestamp, check_timestamp)
        }
    }
}

impl WindowTracker for ChangeTracker {
    fn push(&mut self, timestamp: i64, value: f64) {
        // Calculate change from previous value
        if let Some(prev) = self.prev_value {
            let change = if self.use_percentage {
                if prev == 0.0 {
                    0.0  // Avoid division by zero
                } else {
                    ((value - prev) / prev) * 100.0
                }
            } else {
                value - prev
            };
            
            self.changes.push_back((timestamp, change));
            
            // For bar-based windows, limit the size
            if let Window::Bars(n) = self.window {
                while self.changes.len() > n {
                    self.changes.pop_front();
                }
            }
        }
        
        self.prev_value = Some(value);
    }
    
    fn get(&self) -> Option<f64> {
        // Returns the most recent change
        self.changes.back().map(|(_, change)| *change)
    }
    
    fn prune(&mut self, current_timestamp: i64) {
        while let Some(&(timestamp, _)) = self.changes.front() {
            if !self.in_window(current_timestamp, timestamp) {
                self.changes.pop_front();
            } else {
                break;
            }
        }
    }
    
    fn clear(&mut self) {
        self.changes.clear();
        self.prev_value = None;
    }
}
