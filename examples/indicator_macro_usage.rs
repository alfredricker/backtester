// Example demonstrating the BENEFIT of the indicator macro
// 
// This shows what the macro GENERATES vs what you'd have to write manually
//
// The macro doesn't change HOW you use indicators - it simplifies HOW you DEFINE them

// ============================================================================
// WITHOUT THE MACRO - What you'd have to write manually:
// ============================================================================
/*
pub enum Indicator {
    MovingAverage(MovingAverage),
    RSI(RSI),
    HighOfPeriod(HighOfPeriod),
    LowOfPeriod(LowOfPeriod),
    VWAP(VWAP),
}

impl Indicator {
    pub fn update(&mut self, row: &Row) {
        match self {
            Indicator::MovingAverage(ind) => ind.update(row),
            Indicator::RSI(ind) => ind.update(row),
            Indicator::HighOfPeriod(ind) => ind.update(row),
            Indicator::LowOfPeriod(ind) => ind.update(row),
            Indicator::VWAP(ind) => ind.update(row),
        }
    }
    
    pub fn get(&self) -> Option<f64> {
        match self {
            Indicator::MovingAverage(ind) => ind.get(),
            Indicator::RSI(ind) => ind.get(),
            Indicator::HighOfPeriod(ind) => ind.get(),
            Indicator::LowOfPeriod(ind) => ind.get(),
            Indicator::VWAP(ind) => ind.get(),
        }
    }
    
    pub fn reset(&mut self) {
        match self {
            Indicator::MovingAverage(ind) => ind.reset(),
            Indicator::RSI(ind) => ind.reset(),
            Indicator::HighOfPeriod(ind) => ind.reset(),
            Indicator::LowOfPeriod(ind) => ind.reset(),
            Indicator::VWAP(ind) => ind.reset(),
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Indicator::MovingAverage(_) => "Moving Average",
            Indicator::RSI(_) => "RSI",
            Indicator::HighOfPeriod(_) => "High of Period",
            Indicator::LowOfPeriod(_) => "Low of Period",
            Indicator::VWAP(_) => "VWAP",
        }
    }
}
*/

// ============================================================================
// WITH THE MACRO - What you actually write:
// ============================================================================
/*
define_indicators! {
    MovingAverage(MovingAverage) => "Moving Average",
    RSI(RSI) => "RSI",
    HighOfPeriod(HighOfPeriod) => "High of Period",
    LowOfPeriod(LowOfPeriod) => "Low of Period",
    VWAP(VWAP) => "VWAP",
}
*/

// ============================================================================
// THE REAL BENEFIT: Adding a new indicator
// ============================================================================
//
// WITHOUT MACRO: Add indicator to 6 places:
//   1. The enum variant
//   2. The update() match arm
//   3. The get() match arm
//   4. The reset() match arm
//   5. The name() match arm
//   6. Import statement
//
// WITH MACRO: Add indicator to 2 places:
//   1. The macro invocation (one line)
//   2. Import statement
//
// This prevents bugs from forgetting to update a match arm when adding indicators

fn main() {
    println!("Indicator Macro Benefits");
    println!("{}", "=".repeat(60));
    println!();
    println!("The macro doesn't change HOW you use indicators.");
    println!("It reduces code duplication when DEFINING them.");
    println!();
    println!("Benefits:");
    println!("  1. Single source of truth - define each indicator once");
    println!("  2. No repetitive match arms to maintain");
    println!("  3. Adding new indicators is less error-prone");
    println!("  4. Compiler will catch missing variants automatically");
    println!();
    println!("To add a new indicator (e.g., Bollinger Bands):");
    println!();
    println!("  define_indicators! {{");
    println!("      MovingAverage(MovingAverage) => \"Moving Average\",");
    println!("      RSI(RSI) => \"RSI\",");
    println!("      HighOfPeriod(HighOfPeriod) => \"High of Period\",");
    println!("      LowOfPeriod(LowOfPeriod) => \"Low of Period\",");
    println!("      VWAP(VWAP) => \"VWAP\",");
    println!("      BollingerBands(BollingerBands) => \"Bollinger Bands\",  // <- Just add this!");
    println!("  }}");
    println!();
    println!("That one line automatically generates all 4 match arms!");
}

