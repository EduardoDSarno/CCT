//! Volatility indicators: True Range (TR) and Average True Range (ATR)

use crate::indicators::candle::Candle;

const DEFAULT_ATR_PERIOD: usize = 14;

/// Calculates the True Range for a single candle.
///
/// True Range is the greatest of:
/// - Current High - Current Low (candle range)
/// - |Current High - Previous Close|
/// - |Current Low - Previous Close|
///
/// For the first candle (no previous close), returns the candle's range.
pub fn true_range(candle: &Candle, prev_close: Option<f64>) -> f64 {
    match prev_close {
        Some(prev) => {
            let high_prev = (candle.get_high() - prev).abs();
            let low_prev = (candle.get_low() - prev).abs();
            candle.range().max(high_prev).max(low_prev)
        }
        None => candle.range(),
    }
}

/// Calculates the Average True Range (ATR) over a slice of candles.
///
/// ATR measures market volatility by averaging True Range values.
/// Pass `None` to use the default period of 14, or `Some(n)` for a custom period.
///
/// Returns `None` if there are not enough candles for the given period.
pub fn atr(candles: &[Candle], period: Option<usize>) -> Option<f64> {
    let period = period.unwrap_or(DEFAULT_ATR_PERIOD);

    if period == 0 || candles.len() < period {
        return None;
    }

    let start_index = candles.len() - period;
    let mut total_tr = 0.0;

    for i in start_index..candles.len() {
        let prev_close = if i > 0 {
            Some(candles[i - 1].get_close())
        } else {
            None
        };
        total_tr += true_range(&candles[i], prev_close);
    }

    Some(total_tr / period as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_candles() -> Vec<Candle> {
        vec![
            Candle::new(0, 100.0, 105.0, 95.0, 102.0, 1000.0),
            Candle::new(0, 102.0, 108.0, 100.0, 106.0, 1200.0),
            Candle::new(0, 106.0, 110.0, 104.0, 109.0, 1100.0),
        ]
    }

    #[test]
    fn test_true_range_no_previous() {
        let candle = Candle::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0);
        let tr = true_range(&candle, None);
        assert_eq!(tr, 15.0); // 110 - 95
    }

    #[test]
    fn test_true_range_with_previous() {
        let candle = Candle::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0);
        let tr = true_range(&candle, Some(90.0));
        assert_eq!(tr, 20.0); // max(15, |110-90|, |95-90|) = 20
    }

    #[test]
    fn test_atr_insufficient_candles() {
        let candles = sample_candles();
        let result = atr(&candles, Some(5));
        assert!(result.is_none());
    }

    #[test]
    fn test_atr_with_enough_candles() {
        let candles = sample_candles();
        let result = atr(&candles, Some(3)).unwrap();
        assert!(result > 0.0);
    }
}
