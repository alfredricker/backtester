use crate::equity::master::{EquityTracker, StrategyRunner};
use crate::indicators::{MovingAverage, Window, CommonField};
use crate::position::condition::{Condition, Conditionable};
use crate::position::strategy::{PositionStrategy, Action};
use crate::position::sizing::SizingStrategy;
use crate::position::order::OrderType;
use crate::strategy::Strategy;

/// Create the MA Momentum Strategy
pub fn create_ma_momentum() -> Box<dyn StrategyRunner> {
    // 1. Define Indicators
    // 30-minute Moving Average of Typical Price
    let ma30 = MovingAverage::new(
        Window::Minutes(30),
        CommonField::Typical
    );

    // 60-minute Moving Average of Typical Price
    let ma60 = MovingAverage::new(
        Window::Minutes(60),
        CommonField::Typical
    );

    // 2. Define Conditions
    // We want to track when MA30 crosses above or below MA60
    // Since MovingAverage implements Indicator, and Box<dyn Indicator> implements Conditionable,
    // we box them up.
    
    // Note: We need to clone or recreate indicators if we used them in multiple places, 
    // but here we are creating a single condition pair for entry/exit logic.
    // Actually, for a crossover strategy, we typically want:
    // Entry: CrossAbove(MA30, MA60)
    // Exit: CrossBelow(MA30, MA60)
    //
    // Our Condition struct holds the state (previous values). 
    // We need separate Condition instances if we want to track different states, 
    // but here the logic is symmetric. 
    // However, Condition<L, R> owns L and R. 
    // If we want to share the SAME indicator instance between Entry and Exit checks 
    // (so they update together), we have a bit of an ownership challenge with the current 
    // Condition structure which owns L and R.
    //
    // TODO: Refactor Condition to share underlying indicators (e.g., using Rc<RefCell<>> or similar),
    // or simply accept that we might calculate the same MA twice if we create two independent Conditions.
    //
    // For now, we will create a single PositionStrategy that handles Entry. 
    // To handle Exit, we might need another PositionStrategy or a more complex Condition.
    //
    // Let's assume we create two independent sets of indicators for now to avoid ownership issues,
    // even though it's inefficient.
    
    // Set 1 for Entry Condition
    let entry_ma30 = Box::new(MovingAverage::new(Window::Minutes(30), CommonField::Typical)) as Box<dyn Conditionable>;
    let entry_ma60 = Box::new(MovingAverage::new(Window::Minutes(60), CommonField::Typical)) as Box<dyn Conditionable>;
    
    let entry_condition = Condition::new(entry_ma30, entry_ma60);

    let entry_strategy = PositionStrategy::new(
        entry_condition,
        SizingStrategy::Fixed(100), // Buy 100 shares
        OrderType::MarketBuy(100),
        Action::Entry,
        Some("MA Cross Entry".to_string())
    );

    // Set 2 for Exit Condition
    // We want to exit when MA30 crosses BELOW MA60.
    // Currently, we need to create new instances of the indicators because Condition takes ownership.
    // TODO: Optimize this to share indicator instances.
    let exit_ma30 = Box::new(MovingAverage::new(Window::Minutes(30), CommonField::Typical)) as Box<dyn Conditionable>;
    let exit_ma60 = Box::new(MovingAverage::new(Window::Minutes(60), CommonField::Typical)) as Box<dyn Conditionable>;
    
    let exit_condition = Condition::new(exit_ma30, exit_ma60);

    let exit_strategy = PositionStrategy::new(
        exit_condition,
        SizingStrategy::Fixed(100), // Sell 100 shares (or close position)
        OrderType::MarketSell(100),
        Action::Exit,
        Some("MA Cross Exit".to_string())
    );

    // Create the Strategy container
    let strategy = Strategy::new(
        "MA Momentum".to_string(),
        vec![entry_strategy, exit_strategy]
    );

    Box::new(strategy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ohlcv::Row;
    use crate::equity::master::EquityTracker;

    #[test]
    fn test_ma_momentum_backtest() {
        // TODO: Implement a proper data loader or mock data generator
        // For now, we will manually create a few rows to verify the tracker runs
        
        let mut tracker = EquityTracker::new(Box::new(create_ma_momentum));
        
        // Generate some dummy data
        // Ticker A: Price goes up
        let base_time = 1614556800000000000; // 2021-03-01 00:00:00 UTC
        
        for i in 0..100 {
            let price = 100.0 + (i as f64);
            let row = Row {
                timestamp: base_time + (i * 60 * 1_000_000_000), // +1 minute per row
                open: price,
                high: price + 1.0,
                low: price - 1.0,
                close: price,
                volume: 1000,
                ticker: "AAPL".to_string(),
            };
            
            let result = tracker.process_row(&row);
            assert!(result.is_ok(), "Failed to process row: {:?}", result.err());
        }
        
        // TODO: Verify trades were generated. 
        // Currently EquityTracker doesn't expose executed trades, only updates state.
        // ALSO MISSING: PositionStrategy needs a field to specify the Trigger type (e.g. CrossAbove vs CrossBelow).
        // Currently it holds a Condition but doesn't know WHICH check to perform on it.
    }
}
