// Common indicators that can be applied to any data series (price, volume, etc.)

use crate::types::ohlcv::Row;
use super::time::TimeWindow;

/// Simple Moving Average (SMA)
/// Calculates the arithmetic mean of values over a window
pub fn sma(data: &[Row], window: TimeWindow, field_extractor: impl Fn(&Row) -> f64) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    match window {
        TimeWindow::Bars(n) => {
            if data.len() < n {
                return None;
            }
            let sum: f64 = data.iter().rev().take(n).map(&field_extractor).sum();
            Some(sum / n as f64)
        }
        _ => {
            // Time-based window
            let reference_time = data.last()?.timestamp;
            let values: Vec<f64> = data
                .iter()
                .filter(|row| window.contains(reference_time, row.timestamp))
                .map(&field_extractor)
                .collect();
            
            if values.is_empty() {
                return None;
            }
            
            let sum: f64 = values.iter().sum();
            Some(sum / values.len() as f64)
        }
    }
}

/// Exponential Moving Average (EMA)
/// Gives more weight to recent values
pub fn ema(
    data: &[Row],
    window: TimeWindow,
    field_extractor: impl Fn(&Row) -> f64,
    previous_ema: Option<f64>,
) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let period = match window {
        TimeWindow::Bars(n) => n,
        TimeWindow::Minutes(m) => m as usize,
        TimeWindow::Hours(h) => h as usize,
        TimeWindow::Days(d) => d as usize,
    };

    if period == 0 {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let current_value = field_extractor(data.last()?);

    match previous_ema {
        Some(prev) => Some((current_value * multiplier) + (prev * (1.0 - multiplier))),
        None => {
            // If no previous EMA, use SMA as the starting point
            sma(data, window, field_extractor)
        }
    }
}

/// Weighted Moving Average (WMA)
/// Linear weights - most recent data has highest weight
pub fn wma(data: &[Row], window: TimeWindow, field_extractor: impl Fn(&Row) -> f64) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    match window {
        TimeWindow::Bars(n) => {
            if data.len() < n {
                return None;
            }
            
            let values: Vec<f64> = data.iter().rev().take(n).map(&field_extractor).collect();
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            
            for (i, value) in values.iter().enumerate() {
                let weight = (n - i) as f64;
                weighted_sum += value * weight;
                weight_sum += weight;
            }
            
            Some(weighted_sum / weight_sum)
        }
        _ => {
            // Time-based window
            let reference_time = data.last()?.timestamp;
            let values: Vec<(f64, i64)> = data
                .iter()
                .filter(|row| window.contains(reference_time, row.timestamp))
                .map(|row| (field_extractor(row), row.timestamp))
                .collect();
            
            if values.is_empty() {
                return None;
            }
            
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            
            for (i, (value, _)) in values.iter().enumerate() {
                let weight = (i + 1) as f64;
                weighted_sum += value * weight;
                weight_sum += weight;
            }
            
            Some(weighted_sum / weight_sum)
        }
    }
}

/// Standard Deviation
pub fn std_dev(data: &[Row], window: TimeWindow, field_extractor: impl Fn(&Row) -> f64) -> Option<f64> {
    let mean = sma(data, window, &field_extractor)?;
    
    match window {
        TimeWindow::Bars(n) => {
            if data.len() < n {
                return None;
            }
            
            let variance: f64 = data
                .iter()
                .rev()
                .take(n)
                .map(|row| {
                    let diff = field_extractor(row) - mean;
                    diff * diff
                })
                .sum::<f64>() / n as f64;
            
            Some(variance.sqrt())
        }
        _ => {
            let reference_time = data.last()?.timestamp;
            let values: Vec<f64> = data
                .iter()
                .filter(|row| window.contains(reference_time, row.timestamp))
                .map(&field_extractor)
                .collect();
            
            if values.is_empty() {
                return None;
            }
            
            let variance: f64 = values
                .iter()
                .map(|&value| {
                    let diff = value - mean;
                    diff * diff
                })
                .sum::<f64>() / values.len() as f64;
            
            Some(variance.sqrt())
        }
    }
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
    fn test_sma() {
        let data = create_test_data();
        let result = sma(&data, TimeWindow::Bars(3), |row| row.close);
        assert_eq!(result, Some(104.0)); // (102 + 104 + 106) / 3
    }

    #[test]
    fn test_sma_volume() {
        let data = create_test_data();
        let result = sma(&data, TimeWindow::Bars(3), |row| row.volume as f64);
        assert_eq!(result, Some(1233.333333333333)); // (1000 + 1200 + 1500) / 3
    }
}
