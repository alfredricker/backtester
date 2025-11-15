# Efficient Data Access Patterns

## Quick Reference

### Pattern 1: Sequential Backtesting (Most Common)
```rust
use strategy_tester::data_manager::DataManager;
use chrono::NaiveDate;

let dm = DataManager::new(data_dir, prefix, 50);

for date in backtest_dates {
    // This will cache automatically - very efficient!
    let data = dm.get_ticker_date("AAPL", date)?;
    
    // Indicators can look back at previously cached dates
    let sma = price_sma(&data, Window::Bars(50), PriceField::Close);
}
// Expected cache hit rate: 80-95%
```

### Pattern 2: Multi-Ticker Strategy
```rust
// Get all tickers for a date, then filter
let all_data = dm.get_all_tickers_date(date)?;

// Group by ticker in memory
let by_ticker: HashMap<String, Vec<Row>> = all_data
    .into_iter()
    .fold(HashMap::new(), |mut map, row| {
        map.entry(row.ticker.clone()).or_insert_with(Vec::new).push(row);
        map
    });

// Process each ticker
for (ticker, data) in by_ticker {
    // Your strategy logic
}
```

### Pattern 3: Single Ticker Analysis
```rust
// For analyzing one ticker across many days
let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

// Option A: Cache-friendly (for strategies that need day boundaries)
let data = dm.get_ticker_range("AAPL", start, end)?;

// Option B: Lazy loading (for pure analysis, ignores day boundaries)
let data = dm.lazy_query_ticker("AAPL", start, end)?;
```

### Pattern 4: Indicator Windows Across Date Boundaries
```rust
// Common problem: 50-bar SMA needs data from previous days

// Solution: Accumulate data as you iterate
let mut historical_buffer = VecDeque::with_capacity(1000);

for date in backtest_dates {
    let today_data = dm.get_ticker_date("AAPL", date)?;
    
    // Add today's data to buffer
    for row in &today_data {
        historical_buffer.push_back(row.clone());
    }
    
    // Keep buffer size reasonable (e.g., 200 bars = ~1 day of 1-min data)
    while historical_buffer.len() > 200 {
        historical_buffer.pop_front();
    }
    
    // Now calculate indicators on the full buffer
    let buffer_vec: Vec<Row> = historical_buffer.iter().cloned().collect();
    let sma = price_sma(&buffer_vec, Window::Bars(50), PriceField::Close);
    
    // Generate signals based on today's data only
    for row in &today_data {
        // Your strategy logic using indicators
    }
}
```

### Pattern 5: Pre-warming Cache
```rust
// Before starting backtest, pre-load common dates
let dates = vec![
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
    NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
];

dm.preload_dates(&dates)?;

// Now your backtest will have immediate cache hits
```

## Performance Tips

### Memory Management
```rust
// Cache size calculation:
// 1 day of 5-min bars = ~78 bars per ticker
// 10 tickers * 78 bars * 100 bytes = ~78 KB per day
// Cache of 50 = ~4 MB (very reasonable!)

// For 1-minute bars:
// 1 day = ~390 bars per ticker
// 10 tickers * 390 bars * 100 bytes = ~390 KB per day
// Cache of 50 = ~20 MB
```

### When to Use What

| Use Case | Method | Why |
|----------|--------|-----|
| Backtesting (sequential days) | `get_ticker_date()` | Natural cache pattern |
| Multi-ticker screening | `get_all_tickers_date()` | Load once, filter many |
| Single ticker deep-dive | `lazy_query_ticker()` | Efficient for long ranges |
| Intraday strategy | `get_ticker_date()` | Day boundaries matter |
| Position sizing research | `get_ticker_range()` | Need exact date alignment |

### Monitoring Performance
```rust
// Check your cache efficiency
let stats = dm.stats();
println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);

// Goals:
// - Sequential backtest: >80% hit rate
// - Random access: >50% hit rate
// - Single ticker analysis: >90% hit rate

if stats.hit_rate() < 0.5 {
    println!("Consider increasing cache size!");
}
```

### Indicator Calculation Best Practices

```rust
// BAD: Recalculating on every bar
for row in &data {
    let sma = price_sma(&data, Window::Bars(20), PriceField::Close);
    // This recalculates the entire SMA each time!
}

// GOOD: Calculate once, or incrementally
let sma = price_sma(&data, Window::Bars(20), PriceField::Close);

// Or for streaming:
let mut ema_value = None;
for i in 0..data.len() {
    ema_value = price_ema(
        &data[..=i], 
        Window::Bars(20), 
        PriceField::Close, 
        ema_value  // Pass previous value
    );
}
```

## Data Organization Recommendations

### Current Setup (Date-based)
```
data/
  nasdaq_data_2024-01-01.parquet
  nasdaq_data_2024-01-02.parquet
```
✅ Perfect for sequential backtesting
✅ Easy to manage and version
✅ Natural for daily updates

### If You Need Better Single-Ticker Performance
```
data/
  by_ticker/
    AAPL_2024-01.parquet
    AAPL_2024-02.parquet
    TSLA_2024-01.parquet
```
Then use lazy scanning:
```rust
let pattern = format!("data/by_ticker/{}_*.parquet", ticker);
let lf = LazyFrame::scan_parquet(&pattern, Default::default())?;
```

### Hybrid Approach (Best of Both)
Keep both organizations:
- Daily files for backtesting
- Ticker files for analysis
- Script to convert between them

## Common Pitfalls

### ❌ Don't: Load everything at start
```rust
// This will kill your memory
let mut all_data = HashMap::new();
for date in all_dates {
    all_data.insert(date, dm.get_all_tickers_date(date)?);
}
```

### ✅ Do: Load on-demand with caching
```rust
// DataManager handles this automatically
for date in all_dates {
    let data = dm.get_ticker_date(ticker, date)?;
    // Process immediately, cache handles reuse
}
```

### ❌ Don't: Create new DataManager per query
```rust
for date in dates {
    let dm = DataManager::new(...); // Loses all cache!
    let data = dm.get_ticker_date(ticker, date)?;
}
```

### ✅ Do: Reuse DataManager
```rust
let dm = DataManager::new(...);
for date in dates {
    let data = dm.get_ticker_date(ticker, date)?;
    // Cache accumulates
}
```

## Benchmark Examples

Based on typical hardware (SSD, 16GB RAM):

| Operation | Cold (no cache) | Hot (cached) |
|-----------|----------------|--------------|
| Load 1 day, 1 ticker | 1-5 ms | <0.1 ms |
| Load 1 day, 100 tickers | 10-50 ms | <1 ms |
| Calculate 20-bar SMA | 0.01 ms | 0.01 ms |
| Sequential backtest (100 days) | ~100 ms | ~10 ms |

## Next Steps

1. Start with `get_ticker_date()` for backtesting
2. Monitor cache hit rate with `dm.stats()`
3. Adjust cache size based on your strategy's lookback period
4. Consider `lazy_query_ticker()` for analysis/research tasks
5. Profile your strategy to find bottlenecks (hint: usually not I/O with this design!)

