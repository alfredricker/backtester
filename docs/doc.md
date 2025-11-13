The most fundamental thing to write is the indicators module
that we can build from our data. Since our data is just timestamp OHLCV ticker, our indicators for now are limited to
volume, price, time based (and common ones such as sma which can apply to volume and price)
See types::ohlcv to see the fundamental data

We use the indicators module to build the events module, which tracks mostly 'crossings', i.e., time later than 15:59:59 or average daily volume > 800000, or sma::volume::minutes(50) > ema::volume::hours(5)

Entry types and exit types:
Time based, either specify a timestamp or a time elapse since entry (in trading minutes, hours, or days)
Another option of time based is "auction based" i.e. get  out at 9:30:00 or 16:00:00 
Event based:
price crosses below a threshold (stop loss sell),
indicator crosses below a threshold, etc.

TRACKING THE LEAST AMOUNT OF DATA
seems like every indicator should be passed at least 1 TimeWindow
HOD(TimeWindow::Days(1))
Track high of day seems simple -- you store the high of the day in the HOD struct, and every new datapoint
you do a simple greater than check. But for sliding windows, this is a little more difficult.

## SOLUTION: Trait-Based Window Trackers

We've implemented a trait-based system where different indicator types use different tracking algorithms:

### WindowTracker Trait
The base trait that all trackers implement:
- `push(timestamp, value)` - Add new data point
- `get()` - Get current result
- `prune(current_timestamp)` - Remove expired data
- `clear()` - Reset the tracker

### ExtremumTracker (Max/Min Indicators)
Uses **Monotonic Deque** algorithm - O(1) amortized time!

**How it works:**
- Only stores "potential future maximums/minimums"
- When adding a value that's larger (for max) than older values, those older values can NEVER be the max again, so we discard them
- The front of the deque is always the current max/min

**Example:** Tracking max with values [3, 1, 4, 1, 5]
```
Add 3: deque=[3], max=3
Add 1: deque=[3,1], max=3  (keep 1, might be max after 3 expires)
Add 4: deque=[4], max=4    (removed 3 and 1, they can never be max now!)
Add 1: deque=[4,1], max=4
Add 5: deque=[5], max=5    (removed everything)
```

**Space efficiency:** Often much smaller than window size! Only stores values that could potentially be the max/min.

### SumTracker (Average Indicators)
Maintains running sum and count - O(1) operations
- Stores all values in window with timestamps
- Keeps running sum updated
- Average = sum / count

### VarianceTracker (Std Dev Indicators)
Tracks sum and sum of squared differences for variance calculation

## Usage Example

```rust
use backtester::indicators::price::{HighOfPeriod, LowOfPeriod, MovingAverage};
use backtester::indicators::time::TimeWindow;

// Create indicators
let mut hod = HighOfPeriod::new(TimeWindow::Days(1), PriceField::High);
let mut lod = LowOfPeriod::new(TimeWindow::Days(1), PriceField::Low);
let mut ma = MovingAverage::new(TimeWindow::Bars(20), PriceField::Close);

// Update with each tick
for row in data {
    hod.update(&row);
    lod.update(&row);
    ma.update(&row);
    
    println!("High: {:?}, Low: {:?}, MA: {:?}", 
        hod.get(), lod.get(), ma.get());
}
```

## Why This is Efficient

**Without monotonic deque (naive approach):**
- Store all values in window: O(W) space
- Scan all values to find max after max expires: O(W) time
- Total per operation: O(W)

**With monotonic deque:**
- Store only potential maximums: O(W) space worst case, often much less
- Get max: O(1) time (just look at front)
- Add value: O(1) amortized (each element added once, removed once)
- Total per operation: O(1) amortized

For a backtest with 1 million ticks and window size 1000, this is 1000x faster!