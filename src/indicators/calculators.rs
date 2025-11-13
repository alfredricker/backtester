/// SPECIFIC CALCULATIONS THAT ARE NOT USED IN GENERAL TRACKERS OR INDICATORS
use crate::types::ohlcv::Row;
use super::time::TimeWindow;

/// Volume-Weighted Average Price (VWAP)
/// Typical price weighted by volume
pub fn vwap(data: &[Row], window: TimeWindow) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    match window {
        TimeWindow::Bars(n) => {
            if data.len() < n {
                return None;
            }

            let mut total_pv = 0.0;
            let mut total_volume = 0.0;

            for row in data.iter().rev().take(n) {
                let typical_price = (row.high + row.low + row.close) / 3.0;
                total_pv += typical_price * row.volume as f64;
                total_volume += row.volume as f64;
            }

            if total_volume == 0.0 {
                return None;
            }

            Some(total_pv / total_volume)
        }
        _ => {
            let reference_time = data.last()?.timestamp;
            let mut total_pv = 0.0;
            let mut total_volume = 0.0;

            for row in data.iter().filter(|row| window.contains(reference_time, row.timestamp)) {
                let typical_price = (row.high + row.low + row.close) / 3.0;
                total_pv += typical_price * row.volume as f64;
                total_volume += row.volume as f64;
            }

            if total_volume == 0.0 {
                return None;
            }

            Some(total_pv / total_volume)
        }
    }
}


/// On-Balance Volume (OBV)
/// Cumulative volume based on price direction
pub fn obv(data: &[Row]) -> Option<i64> {
    if data.len() < 2 {
        return None;
    }

    let mut obv_value: i64 = 0;

    for i in 1..data.len() {
        let current_close = data[i].close;
        let prev_close = data[i - 1].close;

        if current_close > prev_close {
            obv_value += data[i].volume;
        } else if current_close < prev_close {
            obv_value -= data[i].volume;
        }
        // If equal, volume is not added
    }

    Some(obv_value)
}


/// Money Flow Index (MFI)
/// Volume-weighted RSI
pub fn mfi(data: &[Row], window: TimeWindow) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }

    let period = match window {
        TimeWindow::Bars(n) => n,
        _ => return None,
    };

    if data.len() <= period {
        return None;
    }

    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;

    for i in (data.len() - period)..data.len() {
        if i == 0 {
            continue;
        }

        let current_typical = (data[i].high + data[i].low + data[i].close) / 3.0;
        let prev_typical = (data[i - 1].high + data[i - 1].low + data[i - 1].close) / 3.0;
        let money_flow = current_typical * data[i].volume as f64;

        if current_typical > prev_typical {
            positive_flow += money_flow;
        } else if current_typical < prev_typical {
            negative_flow += money_flow;
        }
    }

    if negative_flow == 0.0 {
        return Some(100.0);
    }

    let money_flow_ratio = positive_flow / negative_flow;
    Some(100.0 - (100.0 / (1.0 + money_flow_ratio)))
}