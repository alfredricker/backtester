// Volume-specific indicators
// Convenient wrappers around common indicators for volume analysis

use crate::types::ohlcv::Row;
use super::common::{sma, ema, wma, std_dev};
use super::time::TimeWindow;

/// Simple Moving Average for volume
pub fn volume_sma(data: &[Row], window: TimeWindow) -> Option<f64> {
    sma(data, window, |row| row.volume as f64)
}

/// Exponential Moving Average for volume
pub fn volume_ema(
    data: &[Row],
    window: TimeWindow,
    previous_ema: Option<f64>,
) -> Option<f64> {
    ema(data, window, |row| row.volume as f64, previous_ema)
}

/// Weighted Moving Average for volume
pub fn volume_wma(data: &[Row], window: TimeWindow) -> Option<f64> {
    wma(data, window, |row| row.volume as f64)
}

/// Standard deviation for volume
pub fn volume_std_dev(data: &[Row], window: TimeWindow) -> Option<f64> {
    std_dev(data, window, |row| row.volume as f64)
}

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

/// Average Daily Volume (ADV)
/// Useful for liquidity checks
pub fn average_daily_volume(data: &[Row], days: usize) -> Option<f64> {
    volume_sma(data, TimeWindow::Days(days as i64))
}

/// Volume Rate of Change
/// Percentage change in volume over n periods
pub fn volume_roc(data: &[Row], periods: usize) -> Option<f64> {
    if data.len() <= periods {
        return None;
    }

    let current = data.last()?.volume as f64;
    let previous = data[data.len() - periods - 1].volume as f64;

    if previous == 0.0 {
        return None;
    }

    Some(((current - previous) / previous) * 100.0)
}

/// Relative Volume (RVOL)
/// Current volume compared to average volume
pub fn relative_volume(data: &[Row], window: TimeWindow) -> Option<f64> {
    let current = data.last()?.volume as f64;
    let avg = volume_sma(data, window)?;

    if avg == 0.0 {
        return None;
    }

    Some(current / avg)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<Row> {
        vec![
            Row {
                timestamp: 1000,
                open: 100.0,
                high: 105.0,
                low: 99.0,
                close: 102.0,
                volume: 1000,
                ticker: "TEST".to_string(),
            },
            Row {
                timestamp: 2000,
                open: 102.0,
                high: 106.0,
                low: 101.0,
                close: 104.0,
                volume: 1200,
                ticker: "TEST".to_string(),
            },
            Row {
                timestamp: 3000,
                open: 104.0,
                high: 108.0,
                low: 103.0,
                close: 106.0,
                volume: 1500,
                ticker: "TEST".to_string(),
            },
        ]
    }

    #[test]
    fn test_volume_sma() {
        let data = create_test_data();
        let result = volume_sma(&data, TimeWindow::Bars(3));
        assert_eq!(result, Some(1233.3333333333333));
    }

    #[test]
    fn test_obv() {
        let data = create_test_data();
        let result = obv(&data);
        // Price goes up each time, so OBV = 1200 + 1500 = 2700
        assert_eq!(result, Some(2700));
    }

    #[test]
    fn test_relative_volume() {
        let data = create_test_data();
        let result = relative_volume(&data, TimeWindow::Bars(3));
        // Current volume is 1500, average is 1233.33, so RVOL = 1500/1233.33
        assert!(result.is_some());
        assert!(result.unwrap() > 1.0);
    }
}
