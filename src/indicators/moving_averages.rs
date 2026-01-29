//! Moving Average indicators: Simple Moving Average (SMA) and Exponential Moving Average (EMA)

use crate::indicators::candle::Candle;

/// Calculates the Simple Moving Average (SMA) over a slice of candles.
///
/// SMA = (C1 + C2 + ... + Cn) / n
///
/// Uses the closing prices of the most recent `period` candles.
/// Returns `0.0` if there are not enough candles for the given period.
pub fn sma(candles: &[Candle], period: usize) -> f64 {
    if period == 0 || candles.len() < period {
        return 0.0;
    }

    let start_index = candles.len() - period;
    let sum: f64 = candles[start_index..]
        .iter()
        .map(|c| c.get_close())
        .sum();

    sum / period as f64
}

/// Calculates the Exponential Moving Average (EMA) over a slice of candles.
///
/// EMA gives more weight to recent prices using a smoothing multiplier.
/// EMA = Close * multiplier + EMA_prev * (1 - multiplier)
/// where multiplier = 2 / (period + 1)
///
/// The first EMA value is seeded with the SMA of the first `period` candles.
/// Returns `0.0` if there are not enough candles for the given period.
pub fn ema(candles: &[Candle], period: usize) -> f64 {
    let series = ema_series(candles, period);
    series.last().copied().unwrap_or(0.0)
}

/// Calculates the full EMA series for all candles.
///
/// Returns a vector of EMA values starting from the first calculable point.
/// The returned vector will have length `candles.len() - period + 1`.
/// Returns an empty vector if there are not enough candles.
///
/// Useful for crossover detection where you need historical EMA values.
pub fn ema_series(candles: &[Candle], period: usize) -> Vec<f64> {
    if period == 0 || candles.len() < period {
        return Vec::new();
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema_values = Vec::with_capacity(candles.len() - period + 1);

    // Seed the first EMA with SMA of the first `period` candles
    let initial_sma: f64 = candles[..period]
        .iter()
        .map(|c| c.get_close())
        .sum::<f64>()
        / period as f64;

    ema_values.push(initial_sma);

    // Calculate EMA for remaining candles
    for i in period..candles.len() {
        let close = candles[i].get_close();
        let prev_ema = ema_values.last().unwrap();
        let new_ema = close * multiplier + prev_ema * (1.0 - multiplier);
        ema_values.push(new_ema);
    }

    ema_values
}

/// Calculates the full SMA series for all candles.
///
/// Returns a vector of SMA values starting from the first calculable point.
/// The returned vector will have length `candles.len() - period + 1`.
/// Returns an empty vector if there are not enough candles.
pub fn sma_series(candles: &[Candle], period: usize) -> Vec<f64> {
    if period == 0 || candles.len() < period {
        return Vec::new();
    }

    let mut sma_values = Vec::with_capacity(candles.len() - period + 1);

    for i in (period - 1)..candles.len() {
        let start = i + 1 - period;
        let sum: f64 = candles[start..=i]
            .iter()
            .map(|c| c.get_close())
            .sum();
        sma_values.push(sum / period as f64);
    }

    sma_values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_candles() -> Vec<Candle> {
        // Create candles with known closing prices: 10, 11, 12, 13, 14
        vec![
            Candle::new(0, 10.0, 11.0, 9.0, 10.0, 1000.0),
            Candle::new(0, 11.0, 12.0, 10.0, 11.0, 1000.0),
            Candle::new(0, 12.0, 13.0, 11.0, 12.0, 1000.0),
            Candle::new(0, 13.0, 14.0, 12.0, 13.0, 1000.0),
            Candle::new(0, 14.0, 15.0, 13.0, 14.0, 1000.0),
        ]
    }

    fn trending_up_candles() -> Vec<Candle> {
        // Strong uptrend: 100, 105, 110, 115, 120, 126, 133, 141
        vec![
            Candle::new(0, 100.0, 102.0, 98.0, 100.0, 1000.0),
            Candle::new(0, 103.0, 107.0, 102.0, 105.0, 1000.0),
            Candle::new(0, 106.0, 112.0, 105.0, 110.0, 1000.0),
            Candle::new(0, 111.0, 117.0, 110.0, 115.0, 1000.0),
            Candle::new(0, 116.0, 122.0, 115.0, 120.0, 1000.0),
            Candle::new(0, 121.0, 128.0, 120.0, 126.0, 1000.0),
            Candle::new(0, 127.0, 135.0, 126.0, 133.0, 1000.0),
            Candle::new(0, 134.0, 143.0, 133.0, 141.0, 1000.0),
        ]
    }

    #[test]
    fn test_sma_basic() {
        let candles = sample_candles();
        // SMA of last 3 candles: (12 + 13 + 14) / 3 = 13.0
        let result = sma(&candles, 3);
        assert_eq!(result, 13.0);
    }

    #[test]
    fn test_sma_full_period() {
        let candles = sample_candles();
        // SMA of all 5 candles: (10 + 11 + 12 + 13 + 14) / 5 = 12.0
        let result = sma(&candles, 5);
        assert_eq!(result, 12.0);
    }

    #[test]
    fn test_sma_insufficient_candles() {
        let candles = sample_candles();
        let result = sma(&candles, 10);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_sma_zero_period() {
        let candles = sample_candles();
        let result = sma(&candles, 0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_ema_basic() {
        let candles = sample_candles();
        let result = ema(&candles, 3);
        // EMA should be calculable and positive
        assert!(result > 0.0);
    }

    #[test]
    fn test_ema_insufficient_candles() {
        let candles = sample_candles();
        let result = ema(&candles, 10);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_ema_weights_recent_more() {
        let candles = trending_up_candles();
        let sma_val = sma(&candles, 5);
        let ema_val = ema(&candles, 5);

        // In an uptrend, EMA should be higher than SMA because it weights recent prices more
        assert!(
            ema_val > sma_val,
            "EMA ({}) should be greater than SMA ({}) in uptrend",
            ema_val,
            sma_val
        );
    }

    #[test]
    fn test_ema_series_length() {
        let candles = sample_candles();
        let series = ema_series(&candles, 3);
        // With 5 candles and period 3, we should get 3 EMA values (5 - 3 + 1)
        assert_eq!(series.len(), 3);
    }

    #[test]
    fn test_sma_series_length() {
        let candles = sample_candles();
        let series = sma_series(&candles, 3);
        // With 5 candles and period 3, we should get 3 SMA values
        assert_eq!(series.len(), 3);
    }

    #[test]
    fn test_sma_series_values() {
        let candles = sample_candles();
        let series = sma_series(&candles, 3);
        // First SMA: (10 + 11 + 12) / 3 = 11.0
        // Second SMA: (11 + 12 + 13) / 3 = 12.0
        // Third SMA: (12 + 13 + 14) / 3 = 13.0
        assert_eq!(series[0], 11.0);
        assert_eq!(series[1], 12.0);
        assert_eq!(series[2], 13.0);
    }
}
