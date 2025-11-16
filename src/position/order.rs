use super::side::Side as PositionSide;

#[derive(Debug, Clone, Copy)]
pub enum OrderDistance {
    /// Fixed price level
    Fixed(f64),
    /// Percentage distance from entry
    Percent(f64),
    /// Fixed dollar distance from entry
    Points(f64),
    /// ATR-based distance (multiple of ATR)
    ATR(f64),
}

impl OrderDistance {
    /// Calculate the actual price given entry price and side
    pub fn calculate(&self, entry_price: f64, side: &PositionSide, _atr: Option<f64>) -> f64 {
        match self {
            OrderDistance::Fixed(price) => *price,
            OrderDistance::Percent(pct) => {
                match side {
                    PositionSide::Long => entry_price * (1.0 + pct / 100.0),
                    PositionSide::Short => entry_price * (1.0 - pct / 100.0),
                    PositionSide::None => entry_price,
                }
            }
            OrderDistance::Points(pts) => {
                match side {
                    PositionSide::Long => entry_price + pts,
                    PositionSide::Short => entry_price - pts,
                    PositionSide::None => entry_price,
                }
            }
            OrderDistance::ATR(multiple) => {
                if let Some(atr) = _atr {
                    match side {
                        PositionSide::Long => entry_price + (atr * multiple),
                        PositionSide::Short => entry_price - (atr * multiple),
                        PositionSide::None => entry_price,
                    }
                } else {
                    entry_price
                }
            }
        }
    }
}

/// OrderType is an enum that represents the type of order to be placed
/// The i64 is the size of the order in shares
#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    MarketBuy(i64),
    MarketSell(i64),
    LimitBuy(i64, OrderDistance),  // size, limit price, distance
    LimitSell(i64, OrderDistance), // size, limit price, distance
    StopMarketBuy(i64, OrderDistance),   // size, stop price, distance
    StopLimitBuy(i64, OrderDistance, OrderDistance), // size, stop price, limit price, distance
    StopMarketSell(i64, OrderDistance),  // size, stop price, distance
    StopLimitSell(i64, OrderDistance, OrderDistance), // size, stop price, limit price, distance
    AuctionOpen(i64),
    AuctionClose(i64),
}

impl OrderType {
    /// Get the size (number of shares) for this order
    pub fn size(&self) -> i64 {
        match self {
            OrderType::MarketBuy(s) | OrderType::MarketSell(s) => *s,
            OrderType::LimitBuy(s, _) | OrderType::LimitSell(s, _) => *s,
            OrderType::StopMarketBuy(s, _) | OrderType::StopMarketSell(s, _) => *s,
            OrderType::StopLimitBuy(s, _, _) | OrderType::StopLimitSell(s, _, _) => *s,
            OrderType::AuctionOpen(s) | OrderType::AuctionClose(s) => *s,
        }
    }
    
    /// Check if this is a buy order
    pub fn is_buy(&self) -> bool {
        matches!(self, 
            OrderType::MarketBuy(_) | 
            OrderType::LimitBuy(_, _) | 
            OrderType::StopMarketBuy(_, _) | 
            OrderType::StopLimitBuy(_, _, _)
        )
    }
    
    /// Check if this is a sell order
    pub fn is_sell(&self) -> bool {
        matches!(self, 
            OrderType::MarketSell(_) | 
            OrderType::LimitSell(_, _) | 
            OrderType::StopMarketSell(_, _) | 
            OrderType::StopLimitSell(_, _, _)
        )
    }
}