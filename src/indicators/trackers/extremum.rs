use std::collections::VecDeque;
use super::super::window::{Window, WindowConfig};
use super::WindowTracker;

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
    window: Window,
    
    /// Whether to track maximum (true) or minimum (false)
    track_max: bool,
}

impl ExtremumTracker {
    /// Create a new extremum tracker
    ///
    /// Accepts either `Window` or `WindowConfig` (from `.rounded()`)
    ///
    /// # Arguments
    /// * `window` - The time window to track
    /// * `track_max` - If true, tracks maximum; if false, tracks minimum
    pub fn new(window: impl Into<WindowConfig>, track_max: bool) -> Self {
        let config: WindowConfig = window.into();
        Self {
            deque: VecDeque::new(),
            window: config.window,
            track_max,
        }
    }
    
    /// Create a maximum tracker
    pub fn new_max(window: impl Into<WindowConfig>) -> Self {
        Self::new(window, true)
    }
    
    /// Create a minimum tracker
    pub fn new_min(window: impl Into<WindowConfig>) -> Self {
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
        if let Window::Bars(n) = self.window {
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
