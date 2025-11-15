use crate::indicators::indicator::Indicator;
use crate::types::ohlcv::Row;
use super::event::{Event, Threshold};

/// Direction of the cross event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossDirection {
    /// Crosses from at-or-below to above
    Above,
    /// Crosses from at-or-above to below
    Below,
}

/// Event that triggers when an indicator crosses a threshold
/// 
/// Detects when an indicator value transitions across a threshold in the specified direction.
/// The threshold can be:
/// - A fixed numeric value
/// - Another indicator's value
/// - A field from the current row (Close, High, Low, Open, Volume)
/// 
/// # Examples
/// 
/// ```rust
/// use strategy_tester::events::cross::{Cross, CrossDirection};
/// use strategy_tester::events::event::{Event, Threshold};
/// use strategy_tester::indicators::fields::CommonField;
/// 
/// // Indicator crosses above fixed value (e.g., RSI crosses above 70)
/// let mut event = Cross::new(0, Threshold::Fixed(70.0), CrossDirection::Above);
/// 
/// // Fast MA crosses above slow MA
/// let mut event = Cross::new(0, Threshold::Indicator(1), CrossDirection::Above);
/// 
/// // Indicator crosses below the Close price
/// let mut event = Cross::new(0, Threshold::Field(CommonField::Close), CrossDirection::Below);
/// ```
#[derive(Debug)]
pub struct Cross {
    /// Index of the indicator being monitored
    indicator_idx: usize,
    /// The threshold to cross
    threshold: Threshold,
    /// Direction of the cross to detect
    direction: CrossDirection,
    /// Previous indicator value (for detecting the cross)
    prev_indicator: Option<f64>,
    /// Previous threshold value (for detecting the cross)
    prev_threshold: Option<f64>,
}

impl Cross {
    /// Create a new Cross event
    /// 
    /// # Arguments
    /// * `indicator_idx` - Index of the indicator to monitor
    /// * `threshold` - The threshold to detect crossing
    /// * `direction` - Direction of cross (Above or Below)
    pub fn new(indicator_idx: usize, threshold: Threshold, direction: CrossDirection) -> Self {
        Self {
            indicator_idx,
            threshold,
            direction,
            prev_indicator: None,
            prev_threshold: None,
        }
    }
    
    /// Convenience constructor for crossing above
    pub fn above(indicator_idx: usize, threshold: Threshold) -> Self {
        Self::new(indicator_idx, threshold, CrossDirection::Above)
    }
    
    /// Convenience constructor for crossing below
    pub fn below(indicator_idx: usize, threshold: Threshold) -> Self {
        Self::new(indicator_idx, threshold, CrossDirection::Below)
    }
}

impl Event for Cross {
    fn update(&mut self, indicator, row: &Row) -> bool {
    }

    /// Check if a cross occurred
    /// 
    /// Returns true if a cross in the specified direction occurred
    fn check(&self, current_indicator: f64, current_threshold: f64) -> bool {
        if let (Some(prev_ind), Some(prev_thresh)) = (self.prev_indicator, self.prev_threshold) {
            match self.direction {
                CrossDirection::Above => {
                    // Cross above: was at-or-below, now above
                    prev_ind <= prev_thresh && current_indicator > current_threshold
                }
                CrossDirection::Below => {
                    // Cross below: was at-or-above, now below
                    prev_ind >= prev_thresh && current_indicator < current_threshold
                }
            }
        } else {
            // Not enough history to detect a cross
            false
        }
    }
    
    fn reset(&mut self) {
        self.prev_indicator = None;
        self.prev_threshold = None;
    }
    
    fn name(&self) -> &str {
        match self.direction {
            CrossDirection::Above => "Cross Above",
            CrossDirection::Below => "Cross Below",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::movingAverage::MovingAverage;
    use crate::indicators::time::Window;
    use crate::indicators::fields::CommonField;
    use crate::types::ohlcv::Row;

    fn create_test_row(timestamp: i64, close: f64) -> Row {
        Row {
            timestamp,
            ticker: "TEST".to_string(),
            open: close - 0.5,
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume: 1000,
        }
    }

    #[test]
    fn test_cross_above_fixed_value() {
        let mut indicators = vec![Indicator::MovingAverage(
            MovingAverage::new(Window::Bars(2), CommonField::Close)
        )];
        let mut event = Cross::above(0, Threshold::Fixed(100.0));

        // First update: MA = 90, no cross (no history)
        let row1 = create_test_row(1, 90.0);
        indicators[0].update(&row1);
        assert!(!event.update(&indicators, &row1));

        // Second update: MA = 95, no cross (still below)
        let row2 = create_test_row(2, 100.0);
        indicators[0].update(&row2);
        assert!(!event.update(&indicators, &row2));
        // MA should be (90 + 100) / 2 = 95

        // Third update: MA = 105, CROSS! (was 95, now 105)
        let row3 = create_test_row(3, 110.0);
        indicators[0].update(&row3);
        assert!(event.update(&indicators, &row3));
        // MA should be (100 + 110) / 2 = 105

        // Fourth update: MA = 110, no cross (already above)
        let row4 = create_test_row(4, 110.0);
        indicators[0].update(&row4);
        assert!(!event.update(&indicators, &row4));
    }

    #[test]
    fn test_cross_above_indicator() {
        // Fast MA (2-bar) and Slow MA (3-bar)
        let mut indicators = vec![
            Indicator::MovingAverage(MovingAverage::new(Window::Bars(2), CommonField::Close)),
            Indicator::MovingAverage(MovingAverage::new(Window::Bars(3), CommonField::Close)),
        ];
        let mut event = Cross::above(0, Threshold::Indicator(1));

        // Build up some initial data
        for (ts, price) in [(1, 100.0), (2, 100.0), (3, 100.0)] {
            let row = create_test_row(ts, price);
            indicators[0].update(&row);
            indicators[1].update(&row);
            event.update(&indicators, &row);
        }
        // Both MAs are at 100

        // Make fast MA drop below slow MA
        let row4 = create_test_row(4, 80.0);
        indicators[0].update(&row4);
        indicators[1].update(&row4);
        event.update(&indicators, &row4);
        // Fast: (100 + 80) / 2 = 90
        // Slow: (100 + 100 + 80) / 3 = 93.33

        // Now make fast MA cross above slow MA with a stronger signal
        let row5 = create_test_row(5, 130.0);
        indicators[0].update(&row5);
        indicators[1].update(&row5);
        let crossed = event.update(&indicators, &row5);
        // Fast: (80 + 130) / 2 = 105
        // Slow: (100 + 80 + 130) / 3 = 103.33
        // This should trigger a cross: fast was 90 (< 93.33), now 105 (> 103.33)
        assert!(crossed);
    }

    #[test]
    fn test_cross_below_fixed_value() {
        let mut indicators = vec![Indicator::MovingAverage(
            MovingAverage::new(Window::Bars(2), CommonField::Close)
        )];
        let mut event = Cross::below(0, Threshold::Fixed(100.0));

        // Start above threshold
        let row1 = create_test_row(1, 110.0);
        indicators[0].update(&row1);
        assert!(!event.update(&indicators, &row1));

        // Stay above
        let row2 = create_test_row(2, 105.0);
        indicators[0].update(&row2);
        assert!(!event.update(&indicators, &row2));
        // MA = (110 + 105) / 2 = 107.5

        // Cross below
        let row3 = create_test_row(3, 90.0);
        indicators[0].update(&row3);
        assert!(event.update(&indicators, &row3));
        // MA = (105 + 90) / 2 = 97.5 (crossed from 107.5 to 97.5)

        // Stay below
        let row4 = create_test_row(4, 85.0);
        indicators[0].update(&row4);
        assert!(!event.update(&indicators, &row4));
    }

    #[test]
    fn test_cross_above_field() {
        // Test indicator crossing above a row field (e.g., Close price)
        let mut indicators = vec![Indicator::MovingAverage(
            MovingAverage::new(Window::Bars(2), CommonField::High)
        )];
        let mut event = Cross::above(0, Threshold::Field(CommonField::Close));

        // MA of High starts below Close
        let row1 = create_test_row(1, 100.0); // High = 101.0
        indicators[0].update(&row1);
        assert!(!event.update(&indicators, &row1));

        let row2 = create_test_row(2, 100.0); // High = 101.0, MA = 101.0
        indicators[0].update(&row2);
        assert!(!event.update(&indicators, &row2));
        // MA of High = 101.0, Close = 100.0, MA is above Close (but no cross from below)

        // Drop MA below Close
        let row3 = create_test_row(3, 105.0); // High = 106.0, Close = 105.0
        indicators[0].update(&row3);
        assert!(!event.update(&indicators, &row3));
        // MA = (101 + 106) / 2 = 103.5, Close = 105.0, MA is below Close

        // Cross above Close
        let row4 = create_test_row(4, 100.0); // High = 101.0, Close = 100.0
        indicators[0].update(&row4);
        assert!(event.update(&indicators, &row4));
        // MA = (106 + 101) / 2 = 103.5, Close = 100.0
        // MA was 103.5 < 105.0, now 103.5 > 100.0 - CROSS!
    }

    #[test]
    fn test_reset() {
        let mut indicators = vec![Indicator::MovingAverage(
            MovingAverage::new(Window::Bars(2), CommonField::Close)
        )];
        let mut event = Cross::above(0, Threshold::Fixed(100.0));

        // Build up history
        let row1 = create_test_row(1, 90.0);
        indicators[0].update(&row1);
        event.update(&indicators, &row1);

        // Reset event
        event.reset();

        // After reset, next update should not trigger cross (no history)
        let row2 = create_test_row(2, 110.0);
        indicators[0].update(&row2);
        assert!(!event.update(&indicators, &row2));
    }
}
