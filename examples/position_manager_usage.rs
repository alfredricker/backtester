// Example demonstrating the flexible Position Management system
// 
// This shows how to:
// 1. Create a PositionManager with initial capital
// 2. Define entry strategies with various conditions
// 3. Configure stop loss, take profit, and position sizing
// 4. Process market data and execute trades
// 5. Track P&L and position lifecycle
//
// To run: cargo run --example position_manager_usage

use strategy_tester::position::manager::{
    PositionManager, EntryStrategy, PositionSide, SizingStrategy,
    StopLossConfig, TakeProfitConfig, PositionAction,
};
use strategy_tester::equity::master::EquityTracker;
use strategy_tester::config::{IndicatorConfig, IndicatorSpec};
use strategy_tester::types::ohlcv::Row;
use strategy_tester::indicators::{window::Window, fields::CommonField};
use strategy_tester::events::event::{Event, Threshold};
use std::collections::HashMap;

// Example: Simple crossover event (would be implemented in events module)
// This is a placeholder to show the concept
#[derive(Debug)]
struct MAXrossover {
    fast_idx: usize,
    slow_idx: usize,
    last_cross_above: bool,
}

impl Event for MAXrossover {
    fn update(&mut self, indicators: &[strategy_tester::indicators::indicator::Indicator], _row: &Row) -> bool {
        if let (Some(fast), Some(slow)) = (
            indicators.get(self.fast_idx).and_then(|i| i.get()),
            indicators.get(self.slow_idx).and_then(|i| i.get()),
        ) {
            let cross_above = fast > slow;
            let triggered = cross_above && !self.last_cross_above;
            self.last_cross_above = cross_above;
            triggered
        } else {
            false
        }
    }
    
    fn check(&mut self, indicators: &[strategy_tester::indicators::indicator::Indicator], row: &Row) -> bool {
        self.update(indicators, row)
    }
    
    fn reset(&mut self) {
        self.last_cross_above = false;
    }
    
    fn name(&self) -> &str {
        "MA Crossover"
    }
}

fn main() {
    println!("Position Manager Example");
    println!("{}", "=".repeat(80));
    
    // 1. Set up the EquityTracker with indicators
    let indicator_config = IndicatorConfig {
        enabled: true,
        specs: vec![
            // Fast MA (10 periods)
            IndicatorSpec::MovingAverage {
                window: Window::Bars(10),
                field: CommonField::Close,
            },
            // Slow MA (20 periods)
            IndicatorSpec::MovingAverage {
                window: Window::Bars(20),
                field: CommonField::Close,
            },
            // RSI for additional filtering
            IndicatorSpec::RSI {
                window: Window::Bars(14),
                field: CommonField::Close,
            },
        ],
    };
    
    let mut equity_tracker = EquityTracker::new(&indicator_config);
    
    // 2. Create PositionManager with $100,000 starting capital
    let mut position_manager = PositionManager::new(100_000.0);
    
    // 3. Define an entry strategy: MA Crossover with risk management
    let mut long_strategy = EntryStrategy::new(
        "MA Crossover Long".to_string(),
        PositionSide::Long,
        SizingStrategy::PercentOfAccount(2.0), // Risk 2% of account per trade
    );
    
    // Add the MA crossover condition
    long_strategy.add_condition(Box::new(MAXrossover {
        fast_idx: 0,  // First indicator (10-period MA)
        slow_idx: 1,  // Second indicator (20-period MA)
        last_cross_above: false,
    }));
    
    // Configure stop loss: 2% below entry
    long_strategy.stop_loss = Some(StopLossConfig::Percent(2.0));
    
    // Configure take profit: 3:1 risk/reward ratio
    long_strategy.take_profit = Some(TakeProfitConfig::RiskRewardRatio(3.0));
    
    position_manager.add_entry_strategy(long_strategy);
    
    // 4. Simulate market data
    println!("\nSimulating market data for AAPL...\n");
    
    let mut price = 150.0;
    let mut timestamp = 1_000_000_000_000_000_000; // Starting timestamp in nanoseconds
    
    // Simulate 50 bars of data
    for i in 0..50 {
        // Create a realistic price movement
        let price_change = if i < 15 {
            -1.0 // Downtrend
        } else if i < 35 {
            1.5 // Uptrend (should trigger MA crossover)
        } else {
            -0.5 // Slight pullback
        };
        
        price += price_change;
        timestamp += 60_000_000_000; // 1 minute intervals
        
        let row = Row {
            timestamp,
            ticker: "AAPL".to_string(),
            open: price - 0.5,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1_000_000,
        };
        
        // Update equity tracker with new data
        if let Err(e) = equity_tracker.process_row(&row) {
            eprintln!("Error processing row: {}", e);
            continue;
        }
        
        // Process the row in position manager
        match position_manager.process_row(&row, &equity_tracker) {
            Ok(actions) => {
                for action in actions {
                    match action {
                        PositionAction::Entry { position_id, ticker, side, size, price, timestamp } => {
                            println!("📈 ENTRY: {} | {:?} {} shares of {} @ ${:.2} (ts: {})",
                                position_id, side, size, ticker, price, timestamp);
                            
                            // Show the position details
                            if let Some(pos) = position_manager.get_position(&position_id) {
                                println!("   Stop Loss: ${:.2} | Take Profit: ${:.2}",
                                    pos.stop_loss.unwrap_or(0.0),
                                    pos.take_profit.unwrap_or(0.0));
                            }
                        }
                        PositionAction::Exit { position_id, ticker, price, pnl, .. } => {
                            println!("📉 EXIT:  {} | {} @ ${:.2} | P&L: ${:.2}",
                                position_id, ticker, price, pnl);
                        }
                    }
                }
            }
            Err(e) => {
                // It's OK if ticker not found in early bars
                if i > 5 {
                    eprintln!("Error processing position row: {}", e);
                }
            }
        }
        
        // Show current open positions periodically
        if i % 10 == 0 && i > 0 {
            let open_positions = position_manager.open_positions();
            if !open_positions.is_empty() {
                println!("\n--- Current Open Positions (Bar {}) ---", i);
                let current_prices: HashMap<String, f64> = 
                    vec![("AAPL".to_string(), price)].into_iter().collect();
                
                for pos in open_positions {
                    let unrealized = pos.unrealized_pnl(price);
                    println!("  {} | {} {} shares @ ${:.2} | Unrealized P&L: ${:.2}",
                        pos.id, pos.ticker, pos.size, pos.entry_price, unrealized);
                }
                println!();
            }
        }
    }
    
    // 5. Final statistics
    println!("\n{}", "=".repeat(80));
    println!("FINAL STATISTICS");
    println!("{}", "=".repeat(80));
    
    let open_positions = position_manager.open_positions();
    let closed_positions = position_manager.closed_positions();
    
    println!("\nOpen Positions: {}", open_positions.len());
    println!("Closed Positions: {}", closed_positions.len());
    
    if !closed_positions.is_empty() {
        println!("\nClosed Trades:");
        println!("{:-<80}", "");
        for pos in &closed_positions {
            println!("{} | {} {:?} | Entry: ${:.2} | Exit: ${:.2} | P&L: ${:.2} ({:.2}%)",
                pos.id,
                pos.ticker,
                pos.side,
                pos.entry_price,
                pos.exit_price.unwrap_or(0.0),
                pos.pnl().unwrap_or(0.0),
                pos.pnl_percent().unwrap_or(0.0),
            );
        }
    }
    
    let total_realized = position_manager.total_realized_pnl();
    println!("\nTotal Realized P&L: ${:.2}", total_realized);
    
    if !open_positions.is_empty() {
        let current_prices: HashMap<String, f64> = 
            vec![("AAPL".to_string(), price)].into_iter().collect();
        let total_unrealized = position_manager.total_unrealized_pnl(&current_prices);
        println!("Total Unrealized P&L: ${:.2}", total_unrealized);
    }
    
    println!("\nFinal Account Value: ${:.2}", 100_000.0 + total_realized);
}

