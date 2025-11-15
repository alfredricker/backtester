// Example demonstrating the EquityTracker system
// 
// This shows how to:
// 1. Configure indicators in Config
// 2. Create an EquityTracker
// 3. Process rows sequentially
// 4. Handle timestamp validation
// 5. Query indicator values
//
// To run: cargo run --example equity_tracker_usage

use strategy_tester::equity::master::EquityTracker;
use strategy_tester::config::{Config, IndicatorSpec, IndicatorConfig};
use strategy_tester::types::ohlcv::Row;
use strategy_tester::indicators::{
    window::Window,
    fields::CommonField,
};

fn main() {
    println!("EquityTracker Example");
    println!("{}", "=".repeat(60));
    
    // 1. Create custom indicator configuration
    let indicator_config = IndicatorConfig {
        enabled: true,
        specs: vec![
            IndicatorSpec::MovingAverage {
                window: Window::Bars(3),
                field: CommonField::Close,
            },
            IndicatorSpec::RSI {
                window: Window::Bars(2),
                field: CommonField::Close,
            },
            IndicatorSpec::HighOfPeriod {
                window: Window::Bars(5),
                field: CommonField::High,
            },
        ],
    };
    
    // 2. Create EquityTracker
    let mut tracker = EquityTracker::new(&indicator_config);
    println!("\nCreated EquityTracker with {} indicators per ticker", 
             indicator_config.specs.len());
    
    // 3. Create sample data (multiple tickers, ordered by timestamp)
    let rows = vec![
        Row {
            timestamp: 1000,
            ticker: "AAPL".to_string(),
            open: 150.0,
            high: 152.0,
            low: 149.5,
            close: 151.0,
            volume: 1000000,
        },
        Row {
            timestamp: 1000,
            ticker: "TSLA".to_string(),
            open: 200.0,
            high: 205.0,
            low: 199.0,
            close: 203.0,
            volume: 800000,
        },
        Row {
            timestamp: 2000,
            ticker: "AAPL".to_string(),
            open: 151.0,
            high: 154.0,
            low: 150.5,
            close: 153.0,
            volume: 1100000,
        },
        Row {
            timestamp: 2000,
            ticker: "TSLA".to_string(),
            open: 203.0,
            high: 208.0,
            low: 202.0,
            close: 207.0,
            volume: 900000,
        },
        Row {
            timestamp: 3000,
            ticker: "AAPL".to_string(),
            open: 153.0,
            high: 155.0,
            low: 152.0,
            close: 154.5,
            volume: 950000,
        },
        Row {
            timestamp: 3000,
            ticker: "TSLA".to_string(),
            open: 207.0,
            high: 210.0,
            low: 206.0,
            close: 209.0,
            volume: 850000,
        },
    ];
    
    println!("\n{}", "=".repeat(60));
    println!("Processing {} rows...\n", rows.len());
    
    // 4. Process all rows
    for (i, row) in rows.iter().enumerate() {
        match tracker.process_row(row) {
            Ok(_) => {
                println!("Row {}: {} @ timestamp {} - Close: {:.2}", 
                         i + 1, row.ticker, row.timestamp, row.close);
                
                // Query indicators for this ticker
                if let Some(indicators) = tracker.get_indicators(&row.ticker) {
                    for indicator in indicators {
                        if let Some(value) = indicator.get() {
                            println!("    {}: {:.2}", indicator.name(), value);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("ERROR processing row {}: {}", i + 1, e);
            }
        }
    }
    
    // 5. Summary statistics
    println!("\n{}", "=".repeat(60));
    println!("Summary:");
    println!("  Total tickers tracked: {}", tracker.ticker_count());
    println!("  Tickers: {:?}", tracker.tickers());
    
    for ticker in tracker.tickers() {
        if let Some(last_ts) = tracker.last_timestamp(&ticker) {
            println!("  {}: last timestamp = {}", ticker, last_ts);
        }
    }
    
    // 6. Demonstrate timestamp validation error
    println!("\n{}", "=".repeat(60));
    println!("Testing timestamp validation...");
    
    let bad_row = Row {
        timestamp: 2500,  // Goes backwards!
        ticker: "AAPL".to_string(),
        open: 154.0,
        high: 155.0,
        low: 153.0,
        close: 154.0,
        volume: 1000000,
    };
    
    match tracker.process_row(&bad_row) {
        Ok(_) => println!("  Row processed (unexpected)"),
        Err(e) => println!("  ✓ Caught error: {}", e),
    }
    
    println!("\n{}", "=".repeat(60));
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Automatic ticker initialization on first row");
    println!("  ✓ Per-ticker indicator tracking");
    println!("  ✓ Timestamp monotonicity validation");
    println!("  ✓ Multiple tickers processed in parallel");
    println!("  ✓ Easy indicator value queries");
}

