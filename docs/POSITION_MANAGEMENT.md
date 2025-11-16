# Position Management System

## Overview

The position management system provides a flexible, indicator-driven framework for managing trading positions. It integrates seamlessly with the `EquityTracker` and indicators module to enable complex entry/exit strategies.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────────┐
│                      PositionManager                             │
│  - Tracks all positions (open & closed)                         │
│  - Manages entry strategies                                      │
│  - Coordinates with EquityTracker                                │
│  - Calculates P&L                                                │
└───────────────┬─────────────────────────────────────────────────┘
                │
                ├─────────> EntryStrategy
                │           - Entry conditions (Events)
                │           - Position sizing
                │           - Stop loss / Take profit config
                │
                └─────────> Position
                            - Entry/exit prices & timestamps
                            - Exit conditions (Events)
                            - P&L calculations
                            - State management
```

### Integration with Indicators

```
Row (OHLCV data)
    │
    ├──> EquityTracker
    │    ├─> Indicator 0 (e.g., MA 10)
    │    ├─> Indicator 1 (e.g., MA 20)
    │    └─> Indicator 2 (e.g., RSI 14)
    │
    └──> PositionManager
         └─> EntryStrategy.check_entry(indicators, row)
             ├─> Event 1 (e.g., MA crossover)
             ├─> Event 2 (e.g., RSI > 30)
             └─> Returns: true if ALL conditions met
```

## Key Features

### 1. Flexible Entry Strategies

Entry strategies combine multiple conditions using the `Event` trait:

```rust
let mut strategy = EntryStrategy::new(
    "My Strategy".to_string(),
    PositionSide::Long,
    SizingStrategy::PercentOfAccount(2.0),
);

// Add multiple conditions - ALL must be true to enter
strategy.add_condition(Box::new(MAXrossover { ... }));
strategy.add_condition(Box::new(RSIThreshold { ... }));
```

### 2. Multiple Position Sizing Strategies

```rust
// Fixed number of shares
SizingStrategy::Fixed(100)

// Fixed dollar amount
SizingStrategy::FixedDollar(10_000.0)

// Percentage of account
SizingStrategy::PercentOfAccount(2.0)  // 2% of account

// Risk-based (requires stop loss)
SizingStrategy::RiskBased {
    risk_percent: 1.0,      // Risk 1% of account
    stop_distance: 2.0,     // $2 stop distance
}
```

### 3. Stop Loss Configurations

```rust
// Fixed price
StopLossConfig::Fixed(145.50)

// Percentage below entry
StopLossConfig::Percent(2.0)  // 2% below entry

// Fixed dollar amount
StopLossConfig::Points(5.0)   // $5 below entry

// ATR-based (adaptive)
StopLossConfig::ATR(2.0)      // 2x ATR below entry
```

### 4. Take Profit Configurations

```rust
// Fixed price
TakeProfitConfig::Fixed(155.00)

// Percentage above entry
TakeProfitConfig::Percent(5.0)  // 5% above entry

// Fixed dollar amount
TakeProfitConfig::Points(10.0)  // $10 above entry

// Risk/Reward ratio
TakeProfitConfig::RiskRewardRatio(3.0)  // 3:1 R/R
```

### 5. Exit Conditions

Positions can exit based on:
- **Stop Loss**: Price-based protective stop
- **Take Profit**: Price-based profit target
- **Time**: Maximum time in position
- **Custom Events**: Any indicator-based condition

```rust
// Positions automatically check all exit conditions
position.add_exit_condition(Box::new(RSICrossover { ... }));
```

## Usage Example

### Complete Workflow

```rust
// 1. Setup EquityTracker with indicators
let indicator_config = IndicatorConfig {
    enabled: true,
    specs: vec![
        IndicatorSpec::MovingAverage { 
            window: Window::Bars(10), 
            field: CommonField::Close 
        },
        IndicatorSpec::RSI { 
            window: Window::Bars(14), 
            field: CommonField::Close 
        },
    ],
};
let mut equity_tracker = EquityTracker::new(&indicator_config);

// 2. Create PositionManager
let mut position_manager = PositionManager::new(100_000.0);

// 3. Define Entry Strategy
let mut strategy = EntryStrategy::new(
    "MA Cross + RSI".to_string(),
    PositionSide::Long,
    SizingStrategy::PercentOfAccount(2.0),
);

strategy.add_condition(Box::new(MAXrossover { fast_idx: 0, slow_idx: 1 }));
strategy.stop_loss = Some(StopLossConfig::Percent(2.0));
strategy.take_profit = Some(TakeProfitConfig::RiskRewardRatio(3.0));

position_manager.add_entry_strategy(strategy);

// 4. Process market data
for row in market_data {
    // Update indicators
    equity_tracker.process_row(&row)?;
    
    // Check entries/exits
    let actions = position_manager.process_row(&row, &equity_tracker)?;
    
    for action in actions {
        match action {
            PositionAction::Entry { position_id, price, .. } => {
                println!("Opened position {} at ${}", position_id, price);
            }
            PositionAction::Exit { position_id, pnl, .. } => {
                println!("Closed position {} with P&L ${}", position_id, pnl);
            }
        }
    }
}

// 5. Analyze results
let total_pnl = position_manager.total_realized_pnl();
println!("Total P&L: ${}", total_pnl);
```

## Position Lifecycle

```
┌──────────────┐
│ Entry Signal │
│  Detected    │
└──────┬───────┘
       │
       v
┌──────────────┐      ┌─────────────────┐
│  Calculate   │─────>│ Create Position │
│     Size     │      │   (State: Open) │
└──────────────┘      └────────┬────────┘
                               │
                               v
                      ┌────────────────┐
                      │  Monitor Exit  │
                      │   Conditions   │
                      └────────┬───────┘
                               │
           ┌───────────────────┼───────────────────┐
           │                   │                   │
           v                   v                   v
    ┌──────────┐        ┌──────────┐       ┌──────────┐
    │Stop Loss │        │Take Profit│       │  Custom  │
    │   Hit    │        │   Hit     │       │  Event   │
    └─────┬────┘        └─────┬────┘       └────┬─────┘
          │                   │                  │
          └───────────────────┼──────────────────┘
                              v
                    ┌─────────────────┐
                    │ Close Position  │
                    │(State: Closed)  │
                    └─────────────────┘
                              │
                              v
                    ┌─────────────────┐
                    │  Calculate P&L  │
                    │  Update Account │
                    └─────────────────┘
```

## Event System Integration

The position management system leverages the existing `Event` trait for maximum flexibility:

```rust
pub trait Event {
    fn update(&mut self, indicators: &[Indicator], row: &Row) -> bool;
    fn check(&mut self, indicators: &[Indicator], row: &Row) -> bool;
    fn reset(&mut self);
    fn name(&self) -> &str;
}
```

### Creating Custom Entry/Exit Conditions

You can create any condition by implementing the `Event` trait:

```rust
#[derive(Debug)]
struct PriceAboveMA {
    ma_index: usize,
}

impl Event for PriceAboveMA {
    fn check(&mut self, indicators: &[Indicator], row: &Row) -> bool {
        indicators
            .get(self.ma_index)
            .and_then(|ind| ind.get())
            .map(|ma_value| row.close > ma_value)
            .unwrap_or(false)
    }
    
    fn update(&mut self, indicators: &[Indicator], row: &Row) -> bool {
        self.check(indicators, row)
    }
    
    fn reset(&mut self) {}
    
    fn name(&self) -> &str {
        "Price Above MA"
    }
}
```

## Advanced Features

### Multiple Strategies

You can run multiple strategies simultaneously:

```rust
// Long strategy
let mut long_strategy = EntryStrategy::new(
    "Long MA Cross".to_string(),
    PositionSide::Long,
    SizingStrategy::PercentOfAccount(2.0),
);
position_manager.add_entry_strategy(long_strategy);

// Short strategy
let mut short_strategy = EntryStrategy::new(
    "Short MA Cross".to_string(),
    PositionSide::Short,
    SizingStrategy::PercentOfAccount(2.0),
);
position_manager.add_entry_strategy(short_strategy);
```

### Position Tracking

```rust
// Get all open positions
let open_positions = position_manager.open_positions();

// Get all closed positions
let closed_positions = position_manager.closed_positions();

// Get specific position
let position = position_manager.get_position("POS_0");

// Calculate unrealized P&L
let current_prices = HashMap::from([("AAPL".to_string(), 155.0)]);
let unrealized_pnl = position_manager.total_unrealized_pnl(&current_prices);
```

### Account Management

```rust
// Initial account value
let mut pm = PositionManager::new(100_000.0);

// Update account value after trades
pm.update_account_value(105_000.0);

// Position sizing automatically uses current account value
```

## Design Principles

1. **Separation of Concerns**: Entry logic, exit logic, and position tracking are separate
2. **Composability**: Strategies combine multiple Events using logical AND
3. **Type Safety**: Rust's type system prevents invalid states
4. **Flexibility**: Support for various sizing, stop loss, and take profit strategies
5. **Integration**: Seamless integration with EquityTracker and indicators
6. **Extensibility**: Easy to add new condition types via Event trait

## Performance Considerations

- Positions are stored in a `HashMap` for O(1) lookups
- Only positions matching the current ticker are checked for exits
- Indicator values are computed once per row by EquityTracker
- No unnecessary allocations in hot paths

## Future Enhancements

Potential additions:
- [ ] Partial position exits (scale out)
- [ ] Trailing stops
- [ ] Bracket orders
- [ ] Position limits (max positions, max per ticker)
- [ ] Logical operators for conditions (OR, NOT)
- [ ] Commission and slippage modeling
- [ ] Fill simulation with realistic execution

