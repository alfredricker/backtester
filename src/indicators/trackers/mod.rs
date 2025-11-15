use super::window::Window;

// Module declarations
pub mod extremum;
pub mod sum;
pub mod variance;
pub mod change;
pub mod history;

// Re-exports
pub use extremum::ExtremumTracker;
pub use sum::SumTracker;
pub use variance::VarianceTracker;
pub use change::ChangeTracker;
pub use history::HistoryTracker;

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

    fn in_window(&self, window: &Window, current_timestamp: i64, check_timestamp: i64) -> bool {
        match window {
            // bar based methods are handled by count not time, .push method handles automatically
            Window::Bars(_) => true,
            _ => window.contains(current_timestamp, check_timestamp)
        }
    }
}