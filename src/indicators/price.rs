// Price-specific indicators
// Convenient wrappers around common indicators for OHLCV price fields

use crate::types::ohlcv::{Row, OHLCV};
use super::common::{sma, ema, wma, std_dev};
use super::time::TimeWindow;

/// Price field selector for indicators
pub enum PriceField {
    Open,
    High,
    Low,
    Close,
    /// Typical Price: (High + Low + Close) / 3
    Typical,
    /// Weighted Close: (High + Low + Close + Close) / 4
    WeightedClose,
    /// Median Price: (High + Low) / 2
    Median,
}

impl PriceField {
    pub fn extract(&self, row: &Row) -> f64 {
        match self {
            PriceField::Open => row.open,
            PriceField::High => row.high,
            PriceField::Low => row.low,
            PriceField::Close => row.close,
            PriceField::Typical => (row.high + row.low + row.close) / 3.0,
            PriceField::WeightedClose => (row.high + row.low + row.close + row.close) / 4.0,
            PriceField::Median => (row.high + row.low) / 2.0,
        }
    }
}

// HIGH OF DAY
pub struct High{
    high: f64,
    timeline: TimeWindow,
}

impl HOD{

}

/// Simple Moving Average for price
pub fn price_sma(data: &[Row], window: TimeWindow, field: PriceField) -> Option<f64> {
    sma(data, window, |row| field.extract(row))
}

/// Exponential Moving Average for price
pub fn price_ema(
    data: &[Row],
    window: TimeWindow,
    field: PriceField,
    previous_ema: Option<f64>,
) -> Option<f64> {
    ema(data, window, |row| field.extract(row), previous_ema)
}

/// Weighted Moving Average for price
pub fn price_wma(data: &[Row], window: TimeWindow, field: PriceField) -> Option<f64> {
    wma(data, window, |row| field.extract(row))
}

/// Standard deviation for price
pub fn price_std_dev(data: &[Row], window: TimeWindow, field: PriceField) -> Option<f64> {
    std_dev(data, window, |row| field.extract(row))
}

/// Bollinger Bands - returns (middle, upper, lower)
pub fn bollinger_bands(
    data: &[Row],
    window: TimeWindow,
    field: PriceField,
    num_std_dev: f64,
) -> Option<(f64, f64, f64)> {
    let middle = price_sma(data, window, field)?;
    let std = price_std_dev(data, window, PriceField::Close)?;
    let upper = middle + (num_std_dev * std);
    let lower = middle - (num_std_dev * std);
    Some((middle, upper, lower))
}

/// Average True Range (ATR) - measures volatility
pub fn atr(data: &[Row], window: TimeWindow) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }

    let true_ranges: Vec<f64> = data
        .windows(2)
        .map(|pair| {
            let prev = &pair[0];
            let curr = &pair[1];
            
            let h_l = curr.high - curr.low;
            let h_pc = (curr.high - prev.close).abs();
            let l_pc = (curr.low - prev.close).abs();
            
            h_l.max(h_pc).max(l_pc)
        })
        .collect();

    if true_ranges.is_empty() {
        return None;
    }

    // Create temporary data with true ranges as close prices
    let tr_data: Vec<Row> = true_ranges
        .into_iter()
        .enumerate()
        .map(|(i, tr)| Row {
            timestamp: data[i + 1].timestamp,
            open: tr,
            high: tr,
            low: tr,
            close: tr,
            volume: 0,
            ticker: data[i + 1].ticker.clone(),
        })
        .collect();

    sma(&tr_data, window, |row| row.close)
}

/// Rate of Change (ROC) - percentage change over n periods
pub fn roc(data: &[Row], periods: usize, field: PriceField) -> Option<f64> {
    if data.len() <= periods {
        return None;
    }

    let current = field.extract(data.last()?);
    let previous = field.extract(&data[data.len() - periods - 1]);

    if previous == 0.0 {
        return None;
    }

    Some(((current - previous) / previous) * 100.0)
}

/// Relative Strength Index (RSI)
pub fn rsi(data: &[Row], window: TimeWindow, field: PriceField) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }

    let period = match window {
        TimeWindow::Bars(n) => n,
        _ => return None, // RSI typically uses bar-based periods
    };

    if data.len() <= period {
        return None;
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in (data.len() - period)..data.len() {
        if i == 0 {
            continue;
        }
        let current = field.extract(&data[i]);
        let previous = field.extract(&data[i - 1]);
        let change = current - previous;

        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(change.abs());
        }
    }

    let avg_gain: f64 = gains.iter().sum::<f64>() / period as f64;
    let avg_loss: f64 = losses.iter().sum::<f64>() / period as f64;

    if avg_loss == 0.0 {
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
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
    fn test_price_sma() {
        let data = create_test_data();
        let result = price_sma(&data, TimeWindow::Bars(3), PriceField::Close);
        assert_eq!(result, Some(104.0));
    }

    #[test]
    fn test_typical_price() {
        let data = create_test_data();
        let result = price_sma(&data, TimeWindow::Bars(1), PriceField::Typical);
        // Last row: (108 + 103 + 106) / 3 = 105.666...
        assert!(result.is_some());
    }
}
