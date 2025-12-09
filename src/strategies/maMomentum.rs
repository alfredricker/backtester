use crate::strategy::Strategy;
use crate::backtest::context::TickerContext;
use crate::backtest::signal::Signal;
use crate::position::order::OrderType;
use crate::indicators::indicators::MovingAverage;
use crate::indicators::window::Window;
use crate::indicators::fields::CommonField;

pub struct MaMomentumStrategy {
    // State to track crossovers
    prev_ma30: Option<f64>,
    prev_ma60: Option<f64>,
}

impl MaMomentumStrategy {
    pub fn new() -> Self {
        Self {
            prev_ma30: None,
            prev_ma60: None,
        }
    }
}

impl Strategy for MaMomentumStrategy {
    fn name(&self) -> &str {
        "MA Momentum"
    }

    fn setup(&self, context: &mut TickerContext) {
        // Define indicators
        context.add_indicator(
            "ma30", 
            Box::new(MovingAverage::new(Window::Minutes(30), CommonField::Typical))
        );
        context.add_indicator(
            "ma60", 
            Box::new(MovingAverage::new(Window::Minutes(60), CommonField::Typical))
        );
    }

    fn generate_signals(&mut self, context: &TickerContext) -> Vec<Signal> {
        let mut signals = Vec::new();
        
        let ma30_curr = context.get_indicator("ma30");
        let ma60_curr = context.get_indicator("ma60");

        // Check for crossover
        if let (Some(curr30), Some(curr60), Some(prev30), Some(prev60)) = 
               (ma30_curr, ma60_curr, self.prev_ma30, self.prev_ma60) 
        {
            // Cross Above (Entry)
            if curr30 > curr60 && prev30 <= prev60 {
                signals.push(Signal::new_trigger(
                    context.ticker.clone(),
                    OrderType::MarketBuy()
                ));
            }
            // Cross Below (Exit)
            else if curr30 < curr60 && prev30 >= prev60 {
                 signals.push(Signal::new_trigger(
                    context.ticker.clone(),
                    OrderType::MarketSell()
                ));
            }
        }

        // Update state
        self.prev_ma30 = ma30_curr;
        self.prev_ma60 = ma60_curr;

        signals
    }
}

// Factory function for the engine
pub fn create_ma_momentum() -> Box<dyn Strategy> {
    Box::new(MaMomentumStrategy::new())
}
