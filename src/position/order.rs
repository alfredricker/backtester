use crate::types::ohlcv::Row;
use crate::utils::get_mc_timestamp;
pub struct Order {
    order_type: OrderType, //contains order distance (price information)
    open_or_close: OrderAction,
    timestamp: i64, // submission timestamp
    good_until: OrderTimeline, // default will be EOD (end of day)
    size: i64,
    fill_size: i64,
    fill_price: Option<f64>,
    completed: bool,
}

impl Order {
    pub fn new(order_type: OrderType, open_or_close: OrderAction, 
        timestamp: i64, good_until: Option<OrderTimeline>, size: i64) -> Result<Self, OrderError> {
        // Validate the order type at a reasonable price point
        // This is a basic check; actual validation happens in check() with real prices
        order_type.validate(100.0)?;
        
        Ok(Self {
            order_type,
            open_or_close,
            timestamp,
            good_until: good_until.unwrap_or(OrderTimeline::EOD),
            size,
            fill_size: 0,
            fill_price: None,
            completed: false
        })
    }

    pub fn check(&mut self, row: &Row) -> Result<(), OrderError> {
        // Check if order is already completed or filled
        if self.completed {
            return Err(OrderError::AlreadyCompleted);
        }
        if self.fill_price.is_some() {
            return Err(OrderError::AlreadyFilled);
        }
        
        // Check if order has expired
        let expired = match self.good_until {
            OrderTimeline::GTC => false,
            OrderTimeline::EOD => {
                row.timestamp > get_mc_timestamp(self.timestamp)
            }
        };
        
        if expired {
            self.completed = true;
            return Ok(());
        }
        
        // Check price conditions for filling the order
        match self.order_type {
            OrderType::MarketBuy() | OrderType::MarketSell() => {
                // Market orders fill immediately at current price
                self.fill_price = Some(row.close);
            }
            OrderType::LimitBuy(distance) => {
                let price = distance.calculate(row.close, self.order_type, None)?;
                if row.low <= price {
                    self.fill_price = Some(row.low);
                }
            }
            OrderType::LimitSell(distance) => {
                let price = distance.calculate(row.close, self.order_type, None)?;
                if row.high >= price {
                    self.fill_price = Some(row.high);
                }
            }
            OrderType::StopMarketBuy(distance) => {
                let price = distance.calculate(row.close, self.order_type, None)?;
                if row.high >= price {
                    self.fill_price = Some(row.high);
                }
            }
            OrderType::StopMarketSell(distance) => {
                let price = distance.calculate(row.close, self.order_type, None)?;
                if row.low <= price {
                    self.fill_price = Some(row.low);
                }
            }
            OrderType::StopLimitBuy(stop_distance, limit_distance) => {
                let stop_price = stop_distance.calculate(row.close, self.order_type, None)?;
                let limit_price = limit_distance.calculate(row.close, self.order_type, None)?;
                // Order triggers when price rises to stop_price, fills at limit_price or better
                if row.high >= stop_price && row.low <= limit_price {
                    self.fill_price = Some(row.low.max(limit_price));
                }
            }
            OrderType::StopLimitSell(stop_distance, limit_distance) => {
                let stop_price = stop_distance.calculate(row.close, self.order_type, None)?;
                let limit_price = limit_distance.calculate(row.close, self.order_type, None)?;
                // Order triggers when price drops to stop_price, fills at limit_price or better
                if row.low <= stop_price && row.high >= limit_price {
                    self.fill_price = Some(row.high.min(limit_price));
                }
            }
            OrderType::AuctionOpen() | OrderType::AuctionClose() => {
                // Auction orders not yet implemented
                return Ok(());
            }
        }
        
        // Mark order as filled if fill price was set
        if self.fill_price.is_some() {
            self.fill_size = self.size; // assume sufficient liquidity
            self.completed = true;
        }
        
        Ok(())
    }
}

pub enum OrderAction {
    Open, // order is to open a position
    Close // order is to close a position
}

#[derive(Debug, Clone, Copy)]
pub enum OrderDistance {
    /// Fixed price level
    Fixed(f64),
    /// Percentage distance from current price
    Percent(f64),
    /// Fixed dollar distance from current price
    Points(f64),
    /// ATR-based distance (multiple of ATR)
    ATR(f64),
}

impl OrderDistance {
    /// Calculate the order price given current price and side
    pub fn calculate(&self, current_price: f64, order_type: OrderType, _atr: Option<f64>) -> Result<f64, OrderError> {
        let is_buy = order_type.is_buy();
        match self {
            OrderDistance::Fixed(price) => Ok(*price),
            OrderDistance::Percent(pct) => {
                if is_buy {
                    Ok(current_price * (1.0 + pct / 100.0))
                } else {
                    Ok(current_price * (1.0 - pct / 100.0))
                }
            }
            OrderDistance::Points(pts) => {
                if is_buy {
                    Ok(current_price + pts)
                } else {
                    Ok(current_price - pts)
                }
            }
            OrderDistance::ATR(multiple) => {
                if let Some(atr) = _atr {
                    if is_buy {
                        Ok(current_price + (atr * multiple))
                    } else {
                        Ok(current_price - (atr * multiple))
                    }
                } else {
                    Err(OrderError::ATRRequired) // will panic if no ATR is provided and OrderDistance::ATR is used
                }
            }
        }
    }   
}

/// OrderType is an enum that represents the type of order to be placed
#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    MarketBuy(),
    MarketSell(),
    LimitBuy(OrderDistance),  // limit price, distance
    LimitSell(OrderDistance), // limit price, distance
    StopMarketBuy(OrderDistance),   // stop price, distance
    StopLimitBuy(OrderDistance, OrderDistance), // stop price, limit price, distance
    StopMarketSell(OrderDistance),  // stop price, distance
    StopLimitSell(OrderDistance, OrderDistance), // stop price, limit price, distance
    AuctionOpen(),
    AuctionClose(),
}

impl OrderType {    
    /// Check if this is a buy order
    pub fn is_buy(&self) -> bool {
        matches!(self, 
            OrderType::MarketBuy() | 
            OrderType::LimitBuy(_) | 
            OrderType::StopMarketBuy(_) | 
            OrderType::StopLimitBuy(_, _)
        )
    }
    
    /// Check if this is a sell order
    pub fn is_sell(&self) -> bool {
        matches!(self, 
            OrderType::MarketSell() | 
            OrderType::LimitSell(_) | 
            OrderType::StopMarketSell(_) | 
            OrderType::StopLimitSell(_, _)
        )
    }
    
    /// Validate that stop/limit prices are in correct order
    /// For StopLimitBuy: stop_price <= limit_price
    /// For StopLimitSell: stop_price >= limit_price
    pub fn validate(&self, current_price: f64) -> Result<(), OrderError> {
        match self {
            OrderType::StopLimitBuy(stop_dist, limit_dist) => {
                let stop = stop_dist.calculate(current_price, *self, None)?;
                let limit = limit_dist.calculate(current_price, *self, None)?;
                if stop > limit {
                    return Err(OrderError::InvalidOrder);
                }
            }
            OrderType::StopLimitSell(stop_dist, limit_dist) => {
                let stop = stop_dist.calculate(current_price, *self, None)?;
                let limit = limit_dist.calculate(current_price, *self, None)?;
                if stop < limit {
                    return Err(OrderError::InvalidOrder);
                }
            }
            _ => {}
        }
        Ok(())
    }
}


pub enum OrderTimeline {
    GTC, // good til cancelled
    EOD, // end of day
    // GTD, good til date (not yet implemented)
}

#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("ATR is required for ATR-based distance")]
    ATRRequired,
    #[error("Order is already completed")]
    AlreadyCompleted,
    #[error("Order is already filled")]
    AlreadyFilled,
    #[error("Order is not valid")]
    InvalidOrder,
}