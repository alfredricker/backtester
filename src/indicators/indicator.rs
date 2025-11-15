use crate::types::ohlcv::Row;
use super::indicators::{MovingAverage, RSI, HighOfPeriod, LowOfPeriod, VWAP};

/// Macro to define indicators and generate all boilerplate code
/// 
/// This macro takes a list of indicator definitions and generates:
/// - The Indicator enum with all variants
/// - All match arms for update(), get(), reset(), and name()
/// - Reduces code duplication and makes adding new indicators easier
///
/// # Usage
/// ```
/// define_indicators! {
///     VariantName(StructType) => "Display Name",
///     // ... more indicators
/// }
/// ```
macro_rules! define_indicators {
    (
        $( $variant:ident($type:ty) => $name:expr ),+ $(,)?
    ) => {
        /// Enum representing all available indicator types
        /// 
        /// This provides a unified interface for working with different indicators
        /// through a single enum type, enabling dynamic indicator selection at runtime.
        #[derive(Debug)]
        pub enum Indicator {
            $(
                $variant($type),
            )+
        }

        impl Indicator {
            /// Update the indicator with a new data row
            pub fn update(&mut self, row: &Row) {
                match self {
                    $(
                        Indicator::$variant(ind) => ind.update(row),
                    )+
                }
            }
            
            /// Get the current indicator value
            pub fn get(&self) -> Option<f64> {
                match self {
                    $(
                        Indicator::$variant(ind) => ind.get(),
                    )+
                }
            }
            
            /// Reset the indicator state
            pub fn reset(&mut self) {
                match self {
                    $(
                        Indicator::$variant(ind) => ind.reset(),
                    )+
                }
            }
            
            /// Get a human-readable name for the indicator type
            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        Indicator::$variant(_) => $name,
                    )+
                }
            }
        }
    };
}

// Define all indicators using the macro
define_indicators! {
    MovingAverage(MovingAverage) => "Moving Average",
    RSI(RSI) => "RSI",
    HighOfPeriod(HighOfPeriod) => "High of Period",
    LowOfPeriod(LowOfPeriod) => "Low of Period",
    VWAP(VWAP) => "VWAP",
}

