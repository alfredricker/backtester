use chrono::NaiveTime;
use crate::indicators::window::Window;
use crate::position::sizing::SizingStrategy;

/// Global configuration for strategy testing
#[derive(Debug, Clone)]
pub struct Config {
    pub starting_buying_power: f64,
    /// Market hours and trading sessions
    pub market_hours: MarketHours,
    /// Maximum time in position (in trading minutes, hours, or days)
    pub max_position_time: Option<Window>,
    /// slippage
    pub slippage: f64,
    /// replace orders? How do you replace positions
    pub replacement_strategy: ReplacementStrategy,
    /// sizing strategy
    pub sizing_strategy: SizingStrategy,
}

/// Configuration for market hours and trading sessions
#[derive(Debug, Clone)]
pub struct MarketHours {
    /// Include pre-market hours (typically 4:00 AM - 9:30 AM ET)
    pub include_premarket: bool,
    /// Include post-market hours (typically 4:00 PM - 8:00 PM ET)
    pub include_postmarket: bool,
    /// Regular trading hours start time (typically 9:30:00 AM ET)
    pub market_open: NaiveTime,
    /// Regular trading hours end time (typically 4:00:00 PM ET)
    pub market_close: NaiveTime,
    /// Pre-market start time (typically 4:00:00 AM ET)
    pub premarket_open: NaiveTime,
    /// Post-market end time (typically 8:00:00 PM ET)
    pub postmarket_close: NaiveTime,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            market_hours: MarketHours::default(),
            max_position_time: Some(Window::Days(30)),
            starting_buying_power: 1e5,
            slippage: 0.001, // 0.1% slippage
            replacement_strategy: ReplacementStrategy::Cancel,
            sizing_strategy: SizingStrategy::Fixed(100),
        }
    }
}

impl Default for MarketHours {
    fn default() -> Self {
        MarketHours {
            include_premarket: false,
            include_postmarket: false,
            market_open: NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            market_close: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            premarket_open: NaiveTime::from_hms_opt(4, 0, 0).unwrap(),
            postmarket_close: NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        }
    }
}

impl MarketHours {
    /// Get the earliest valid time for the trading session on a given day
    pub fn earliest_valid_time(&self) -> NaiveTime {
        if self.include_premarket {
            self.premarket_open
        } else {
            self.market_open
        }
    }

    /// Get the latest valid time for the trading session on a given day
    pub fn latest_valid_time(&self) -> NaiveTime {
        if self.include_postmarket {
            self.postmarket_close
        } else {
            self.market_close
        }
    }

    /// Check if a time of day is within valid trading hours
    pub fn is_valid_time(&self, time: NaiveTime) -> bool {
        let earliest = self.earliest_valid_time();
        let latest = self.latest_valid_time();
        time >= earliest && time <= latest
    }
}

/// When maximum buying power is reached, what do we do?
#[derive(Debug,Clone)]
pub enum ReplacementStrategy {
    Queue, // queue up positions that can't be filled because of bp constraints, check if they can be filled in on a FI basis once bp is freed
    ReplaceOldest, // automatically replace the oldest position that was filled
    ReplaceNewest, // automatically replace the newest position that was filled
    ReplaceSignal, // replace by weakest signal (if new position signal is stronger than weakest signal) 
    Cancel, // cancel the pending order, keep portfolio as is
}


// Global static config - in a real app you'd use lazy_static or once_cell
// For now, we'll pass it around or use thread_local
use std::sync::RwLock;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

/// Initialize the global configuration
pub fn init_config(config: Config) {
    let _ = GLOBAL_CONFIG.set(RwLock::new(config));
}

/// Get a copy of the current global configuration
pub fn get_config() -> Config {
    GLOBAL_CONFIG
        .get_or_init(|| RwLock::new(Config::default()))
        .read()
        .unwrap()
        .clone()
}

