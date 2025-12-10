# Backtesting Guide

This guide details how the backtesting engine works, how to configure it, and how to implement custom trading strategies.

## 1. The Backtest Process

The strategy tester uses an event-driven, row-by-row processing model. This ensures that strategies only have access to data available at the current timestamp, preventing look-ahead bias.

### Step-by-Step Flow

1.  **Initialization**:
    *   The `BacktestEngine` is initialized with a `Config` and a `Strategy` factory function.
    *   The `Portfolio` is created with starting capital defined in `Config`.

2.  **Data Ingestion**:
    *   Historical data (OHLCV) is read (typically from Parquet files) and sorted by timestamp.
    *   The engine iterates through each row of data sequentially.

3.  **Row Processing (`process_row`)**:
    For each data point (ticker, timestamp, open, high, low, close, volume):

    a.  **Price Update**: The portfolio updates the current price for the ticker. This is used for tracking unrealized PnL.

    b.  **Context & Strategy Initialization** (if new ticker):
        *   A `TickerContext` is created for the ticker.
        *   A new instance of your `Strategy` is created.
        *   `Strategy::setup()` is called, allowing the strategy to register indicators (e.g., SMA, RSI) with the context.

    c.  **Context Update**:
        *   The `TickerContext` updates all registered indicators with the new data row.

    d.  **Signal Generation**:
        *   `Strategy::generate_signals()` is executed.
        *   The strategy analyzes indicator values and current price data from the `TickerContext`.
        *   It returns a list of `Signal`s (e.g., `Trigger(OrderType::MarketBuy)`).

    e.  **Signal Execution**:
        *   The `Portfolio` processes each signal.
        *   **Entry**: Checks buying power, creates a new `Position`, and logs the trade.
        *   **Exit**: Closes existing positions, calculates PnL, updates buying power, and logs the trade.
        *   **Logging**: Every trade (Entry/Exit) generates a `TradeLog` containing price, timestamp, strategy name, and indicator values at that moment.

## 2. Configuration

Configuration is managed via the `Config` struct in `src/config.rs`.

### Key Configuration Options

*   **`starting_buying_power`**: Initial cash available for trading.
*   **`market_hours`**: Defines valid trading times (Pre-market, Market Open, Post-market).
*   **`max_position_time`**: (Optional) Force close positions after a certain duration.
*   **`slippage`**: Percentage of price to simulate slippage cost (e.g., `0.001` for 0.1%).

### Example Usage

```rust
use strategy_tester::config::{Config, MarketHours};
use chrono::NaiveTime;

let config = Config {
    starting_buying_power: 100_000.0,
    slippage: 0.0005, // 0.05%
    market_hours: MarketHours {
        include_premarket: false,
        include_postmarket: false,
        ..MarketHours::default()
    },
    ..Config::default()
};
```

## 3. Writing a Strategy

To create a strategy, implement the `Strategy` trait found in `src/strategy.rs`.

### The `Strategy` Trait

```rust
pub trait Strategy {
    /// Setup indicators for a specific ticker
    fn setup(&self, context: &mut TickerContext);

    /// Logic to generate signals based on context
    fn generate_signals(&mut self, context: &TickerContext) -> Vec<Signal>;

    /// Strategy display name
    fn name(&self) -> &str;
}
```

### Implementation Steps

1.  **Define Struct**: Create a struct to hold any state your strategy needs (e.g., "previous EMA value" for crossover detection).
2.  **Implement `setup`**: Register the indicators you need.
3.  **Implement `generate_signals`**:
    *   Retrieve indicator values from `context.get_indicator("name")`.
    *   Apply logic (e.g., `if fast_ma > slow_ma`).
    *   Return `Signal`s.

### Example: Moving Average Crossover

```rust
use strategy_tester::strategy::Strategy;
use strategy_tester::backtest::context::TickerContext;
use strategy_tester::backtest::signal::Signal;
use strategy_tester::position::order::OrderType;
use strategy_tester::indicators::indicators::MovingAverage;
use strategy_tester::indicators::window::Window;
use strategy_tester::indicators::fields::CommonField;

pub struct MaCrossoverStrategy {
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

impl Strategy for MaCrossoverStrategy {
    fn name(&self) -> &str { "MA Crossover" }

    fn setup(&self, context: &mut TickerContext) {
        // 1. Register Indicators
        context.add_indicator("fast_ma", Box::new(MovingAverage::new(Window::Minutes(10), CommonField::Close)));
        context.add_indicator("slow_ma", Box::new(MovingAverage::new(Window::Minutes(50), CommonField::Close)));
    }

    fn generate_signals(&mut self, context: &TickerContext) -> Vec<Signal> {
        let mut signals = Vec::new();
        
        // 2. Get Current Values
        let fast = context.get_indicator("fast_ma");
        let slow = context.get_indicator("slow_ma");

        // 3. Check Logic (Golden Cross)
        if let (Some(curr_f), Some(curr_s), Some(prev_f), Some(prev_s)) = 
            (fast, slow, self.prev_fast, self.prev_slow) 
        {
            if curr_f > curr_s && prev_f <= prev_s {
                signals.push(Signal::new_trigger(
                    context.ticker.clone(), 
                    OrderType::MarketBuy()
                ));
            }
        }

        // 4. Update State
        self.prev_fast = fast;
        self.prev_slow = slow;
        
        signals
    }
}
```

## 4. Running a Backtest

Currently, the entry point is typically in `src/main.rs` or an example file.

1.  **Load Data**: Use `parsing::parquet::read_parquet_by_date` or similar to get a DataFrame.
2.  **Initialize Engine**: Create `BacktestEngine` with your config and strategy factory.
3.  **Iterate**: Loop through rows and call `process_row`.
4.  **Analyze**: Inspect `engine.trade_logs` or `engine.portfolio` results.

```rust
// Simplified Main Loop
let mut engine = BacktestEngine::new(config, Box::new(|| Box::new(MyStrategy::new())));

// Iterate through Polars DataFrame rows (pseudocode)
for row in rows {
    engine.process_row(&row);
}

// Analyze Logs
println!("Total Trades: {}", engine.trade_logs.len());
```

