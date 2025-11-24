use std::collections::VecDeque;
use crate::position::order::Order;
use crate::position::side::Side;

pub struct Status {
    pub orders: VecDeque<Order>, // this makes more sense to be in portfolio
    pub size: i64,
    pub average_entry_price: Option<f64>,
    pub side: Side,
}

impl Status {
    pub fn new(order: Order) -> Self {
        Self {
            orders: VecDeque::from([order]),
            size: 0,
            average_entry_price: None,
            side: Side::None,
        }
    }
}