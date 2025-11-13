# Indicator Architecture

## Overview

All indicators follow the **update() + get() pattern** for efficient streaming/backtesting:

```rust
// Create indicator
let mut indicator = SomeIndicator::new(TimeWindow::Bars(20), field);

// In your backtest loop
for row in data {
    indicator.update(&row);  // Process new data
    
    if let Some(value) = indicator.get() {
        // Use the indicator value
    }
}
```

## Tracker Types

We have specialized trackers optimized for different calculation patterns:

### 1. ExtremumTracker (Max/Min)
**Use for:** High, Low, Maximum, Minimum over a window  
**Algorithm:** Monotonic Deque  
**Complexity:** O(1) amortized per operation  
**Space:** Often much less than window size

```rust
let mut hod = HighOfPeriod::new(TimeWindow::Days(1), CommonField::High);
let mut lod = LowOfPeriod::new(TimeWindow::Days(1), CommonField::Low);
```

### 2. SumTracker (Averages)
**Use for:** SMA, averages, sums  
**Algorithm:** Running sum with deque  
**Complexity:** O(1) per operation  
**Space:** O(W) where W is window size

```rust
let mut ma = MovingAverage::new(TimeWindow::Bars(20), CommonField::Close);
```

### 3. VarianceTracker (Standard Deviation)
**Use for:** Standard deviation, variance, Bollinger Bands  
**Algorithm:** Tracks sum and sum of squared differences  
**Complexity:** O(1) per operation  
**Space:** O(W)

```rust
// Use in calculators.rs for now
// Can create StdDev indicator wrapper if needed
```

### 4. VWAPTracker (Volume-Weighted Average)
**Use for:** VWAP  
**Algorithm:** Tracks Σ(price×volume) and Σ(volume)  
**Complexity:** O(1) per operation  
**Space:** O(W)

```rust
let mut vwap = VWAP::typical(TimeWindow::Days(1));
// or
let mut vwap = VWAP::new(TimeWindow::Hours(4), CommonField::Close);
```

### 5. RSITracker (Relative Strength Index)
**Use for:** RSI  
**Algorithm:** Tracks average gains and losses  
**Complexity:** O(1) per operation  
**Space:** O(W)

```rust
let mut rsi = RSI::close(TimeWindow::Bars(14));
// or
let mut rsi = RSI::new(TimeWindow::Bars(14), CommonField::Typical);
```

### 6. HistoryTracker (Full Value Storage)
**Use for:** Complex indicators, pattern recognition  
**Algorithm:** Stores all values in window  
**Complexity:** O(1) per operation, O(W) to scan values  
**Space:** O(W) - stores everything

```rust
use super::tracker::HistoryTracker;

let mut tracker = HistoryTracker::new(TimeWindow::Bars(50));
tracker.push(timestamp, value);
tracker.prune(timestamp);

// Access all values for custom calculations
for (ts, val) in tracker.values() {
    // Your logic here
}
```

## Module Organization

```
src/indicators/
├── tracker.rs       - Core tracking algorithms (don't use directly)
├── general.rs       - Stateful indicators (use these!)
├── calculators.rs   - Batch calculation functions
├── time.rs          - TimeWindow enum and helpers
├── price.rs         - Price-specific indicators (empty for now)
└── volume.rs        - Volume-specific indicators (empty for now)
```

## Available Indicators

### In `general.rs` (Stateful - Use These!)

| Indicator | Constructor | Window Type | Fields |
|-----------|------------|-------------|--------|
| `HighOfPeriod` | `new(window, field)` | Any | All |
| `LowOfPeriod` | `new(window, field)` | Any | All |
| `MovingAverage` | `new(window, field)` | Any | All |
| `VWAP` | `typical(window)` or `new(window, field)` | Any | All |
| `RSI` | `close(window)` or `new(window, field)` | Any | All |

### In `calculators.rs` (Batch - For Historical Queries)

| Function | Purpose | Window Type |
|----------|---------|-------------|
| `vwap(data, window)` | Volume-weighted average | Any |
| `obv(data)` | On-balance volume | N/A (cumulative) |
| `mfi(data, window)` | Money Flow Index | Bars only |

## CommonField Options

All indicators accept these field types:

```rust
pub enum CommonField {
    Open,           // Opening price
    High,           // High price
    Low,            // Low price  
    Close,          // Closing price
    Volume,         // Volume (as f64)
    Median,         // (High + Low) / 2
    Typical,        // (High + Low + Close) / 3
    WeightedClose,  // (High + Low + Close + Close) / 4
}
```

## TimeWindow Options

```rust
TimeWindow::Bars(n)      // Last N bars/candles
TimeWindow::Minutes(m)   // Last M minutes
TimeWindow::Hours(h)     // Last H hours
TimeWindow::Days(d)      // Last D days
```

## Example: Complete Strategy

```rust
use crate::indicators::general::{HighOfPeriod, LowOfPeriod, MovingAverage, RSI, VWAP};
use crate::indicators::time::TimeWindow;
use crate::indicators::general::CommonField;

// Initialize indicators
let mut hod = HighOfPeriod::new(TimeWindow::Days(1), CommonField::High);
let mut lod = LowOfPeriod::new(TimeWindow::Days(1), CommonField::Low);
let mut ma20 = MovingAverage::new(TimeWindow::Bars(20), CommonField::Close);
let mut ma50 = MovingAverage::new(TimeWindow::Bars(50), CommonField::Close);
let mut rsi = RSI::close(TimeWindow::Bars(14));
let mut vwap = VWAP::typical(TimeWindow::Days(1));

// Backtest loop
for row in data {
    // Update all indicators
    hod.update(&row);
    lod.update(&row);
    ma20.update(&row);
    ma50.update(&row);
    rsi.update(&row);
    vwap.update(&row);
    
    // Strategy logic
    if let (Some(close), Some(ma20_val), Some(ma50_val), Some(rsi_val)) = 
        (Some(row.close), ma20.get(), ma50.get(), rsi.get()) {
        
        // Golden cross + oversold RSI
        if ma20_val > ma50_val && rsi_val < 30.0 {
            println!("BUY SIGNAL at {}", row.timestamp);
        }
        
        // Death cross + overbought RSI
        if ma20_val < ma50_val && rsi_val > 70.0 {
            println!("SELL SIGNAL at {}", row.timestamp);
        }
    }
}
```

## Adding New Indicators

To add a new indicator:

1. **Determine which tracker type fits** (or create a new specialized tracker)
2. **Add the indicator struct to `general.rs`**:
   ```rust
   pub struct MyIndicator {
       tracker: SomeTracker,
       field: CommonField,
   }
   
   impl MyIndicator {
       pub fn new(window: TimeWindow, field: CommonField) -> Self { ... }
       pub fn update(&mut self, row: &Row) { ... }
       pub fn get(&self) -> Option<f64> { ... }
       pub fn reset(&mut self) { ... }
   }
   ```

3. **Done!** Follow the same pattern everywhere.

## Performance Notes

- **ExtremumTracker** is incredibly efficient - often 100-1000x faster than naive approaches
- **All trackers** use O(1) amortized operations per update
- **Memory usage** is proportional to window size (some trackers use less)
- **No allocations** during updates (pre-allocated VecDeques)

## Future Enhancements

Potential additions:
- `EMATracker` - Exponential moving average
- `ATRTracker` - Average true range
- `BollingerBands` - Using VarianceTracker
- `MACD` - Combining multiple EMAs
- `StochasticTracker` - Stochastic oscillator
- `MFITracker` - Better MFI implementation

