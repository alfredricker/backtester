use chrono::NaiveTime;
use crate::indicators::indicator::Indicator;
use crate::indicators::indicators::{MovingAverage, RSI, HighOfPeriod, LowOfPeriod, VWAP};
use crate::indicators::window::Window;
use crate::indicators::fields::{CommonField, PriceField};

/// Global configuration for strategy testing
#[derive(Debug, Clone)]
pub struct Config {
    pub buying_power: f64,
    /// Market hours and trading sessions
    pub market_hours: MarketHours,
    /// Configuration for which indicators to track for each ticker
    pub indicator_config: IndicatorConfig,
    /// Maximum time in position (in trading minutes, hours, or days)
    pub max_position_time: Option<Window>,
}

/// Specification for creating indicators
#[derive(Debug, Clone)]
pub enum IndicatorSpec {
    MovingAverage { window: Window, field: CommonField },
    RSI { window: Window, field: CommonField },
    HighOfPeriod { window: Window, field: CommonField },
    LowOfPeriod { window: Window, field: CommonField },
    VWAP { window: Window, price_field: Option<PriceField> },
}

impl IndicatorSpec {
    /// Create an indicator instance from this specification
    pub fn build(&self) -> Indicator {
        match self {
            IndicatorSpec::MovingAverage { window, field } => {
                Indicator::MovingAverage(MovingAverage::new(*window, *field))
            }
            IndicatorSpec::RSI { window, field } => {
                Indicator::RSI(RSI::new(*window, *field))
            }
            IndicatorSpec::HighOfPeriod { window, field } => {
                Indicator::HighOfPeriod(HighOfPeriod::new(*window, *field))
            }
            IndicatorSpec::LowOfPeriod { window, field } => {
                Indicator::LowOfPeriod(LowOfPeriod::new(*window, *field))
            }
            IndicatorSpec::VWAP { window, price_field } => {
                Indicator::VWAP(VWAP::new(*window, *price_field))
            }
        }
    }
}

/// Configuration for which indicators to track for each ticker
#[derive(Debug, Clone)]
pub struct IndicatorConfig {
    /// Enable/disable indicator tracking globally
    pub enabled: bool,
    /// List of indicator specifications
    pub specs: Vec<IndicatorSpec>,
}

impl IndicatorConfig {
    /// Create a default indicator configuration with common indicators
    pub fn default_indicators() -> Self {
        Self {
            enabled: true,
            specs: vec![
                IndicatorSpec::MovingAverage {
                    window: Window::Bars(20),
                    field: CommonField::Close,
                },
                IndicatorSpec::RSI {
                    window: Window::Bars(14),
                    field: CommonField::Close,
                },
                IndicatorSpec::HighOfPeriod {
                    window: Window::Days(1),
                    field: CommonField::High,
                },
                IndicatorSpec::LowOfPeriod {
                    window: Window::Days(1),
                    field: CommonField::Low,
                },
                IndicatorSpec::VWAP {
                    window: Window::Days(1),
                    price_field: Some(PriceField::Typical),
                },
            ],
        }
    }
    
    /// Create indicators for a new ticker
    pub fn create_indicators(&self) -> Vec<Indicator> {
        if !self.enabled {
            return vec![];
        }
        self.specs.iter().map(|spec| spec.build()).collect()
    }
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self::default_indicators()
    }
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
            indicator_config: IndicatorConfig::default(),
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
