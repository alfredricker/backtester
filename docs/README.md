# Trading Strategy Tester

A Rust-based quantitative trading strategy testing framework using Polars for efficient data handling and analysis.

## Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Cargo (comes with Rust)

## Building

To build the project:

```bash
cargo build
```

## Running

To run the main program:

```bash
cargo run
```

To run with verbose output:

```bash
RUST_LOG=debug cargo run
```

## Documentation

- [**Backtesting Guide**](BACKTESTING_GUIDE.md): Step-by-step guide to the backtest engine, creating strategies, and configuration.

## Project Structure

```
src/
├── main.rs                 # Entry point
├── backtest/               # Backtest engine core (Portfolio, Engine, Context)
├── strategies/             # User-defined strategies (e.g., maMomentum)
├── parsing/
│   ├── mod.rs            # Constants
│   └── parquet.rs        # Parquet file reading
├── types/
│   ├── ohlcv.rs          # OHLCV data types
│   └── log.rs            # Trade logging types
├── indicators/           # Technical indicators implementation
├── position/             # Position management, orders, and sizing
└── config.rs             # Configuration logic
```

## Dependencies

- **polars** - Fast dataframe library for data manipulation
- **chrono** - Date and time handling

## Current Functionality

The main program:
1. Creates a test date (March 1, 2021)
2. Reads the corresponding parquet file from `DATA_LOAD_DIR`
3. Displays the first 5 rows

To test, ensure you have parquet files at:
```
/home/fred/Data/quant/5min/nasdaq_data_YYYY-MM-DD.parquet
```

## Example Output

```
shape: (5, n_cols)
┌─────────┬──────────┬───────┬─────────┬──────────┐
│ symbol  ┆ time     ┆ open  ┆ high    ┆ low      │
│ ---     ┆ ---      ┆ ---   ┆ ---     ┆ ---      │
│ str     ┆ datetime ┆ f64   ┆ f64     ┆ f64      │
╞═════════╪══════════╪═══════╪═════════╪══════════╡
│ AAPL    ┆ ...      ┆ 123.4 ┆ 124.1   ┆ 122.8    │
│ ...     ┆ ...      ┆ ...   ┆ ...     ┆ ...      │
└─────────┴──────────┴───────┴─────────┴──────────┘
```

