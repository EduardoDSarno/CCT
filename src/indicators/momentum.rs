//! Momentum indicators: Relative Strength Index (RSI)

use crate::indicators::candle::Candle;

const DEFAULT_RSI_PERIOD: usize = 14;

/// Calculates the Relative Strength Index (RSI) over a slice of candles.
///
/// RSI is a momentum oscillator that measures the speed and magnitude of price changes.
/// It oscillates between 0 and 100.
///
/// RSI = 100 - (100 / (1 + RS))
/// where RS = Average Gain / Average Loss over the period
///
/// Common interpretation:
/// - RSI > 70: Overbought (potential sell signal)
/// - RSI < 30: Oversold (potential buy signal)
///
/// Pass `None` to use the default period of 14, or `Some(n)` for a custom period.
/// Returns `0.0` if there are not enough candles (need at least period + 1 candles).
pub fn rsi(candles: &[Candle], period: Option<usize>) -> f64 {
    let period = period.unwrap_or(DEFAULT_RSI_PERIOD);

    // Need at least period + 1 candles to calculate `period` price changes
    if period == 0 || candles.len() < period + 1 {
        return 0.0;
    }

    let changes = price_changes(candles);
    let (gains, losses) = gains_and_losses(&changes);

    // Use the most recent `period` gains/losses
    let start_index = gains.len().saturating_sub(period);
    let recent_gains = &gains[start_index..];
    let recent_losses = &losses[start_index..];

    let avg_gain: f64 = recent_gains.iter().sum::<f64>() / period as f64;
    let avg_loss: f64 = recent_losses.iter().sum::<f64>() / period as f64;

    if avg_loss == 0.0 {
        // No losses means RSI is 100 (maximum bullish)
        return 100.0;
    }

    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

/// Calculates the RSI series for all calculable points.
///
/// Returns a vector of RSI values. The first value corresponds to the point
/// where we have enough data (period + 1 candles).
/// Returns an empty vector if there are not enough candles.
pub fn rsi_series(candles: &[Candle], period: Option<usize>) -> Vec<f64> {
    let period = period.unwrap_or(DEFAULT_RSI_PERIOD);

    if period == 0 || candles.len() < period + 1 {
        return Vec::new();
    }

    let changes = price_changes(candles);
    let (gains, losses) = gains_and_losses(&changes);

    let mut rsi_values = Vec::with_capacity(changes.len() - period + 1);

    // Calculate initial averages using simple average
    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    // First RSI value
    let first_rsi = if avg_loss == 0.0 {
        100.0
    } else {
        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    };
    rsi_values.push(first_rsi);

    // Calculate subsequent RSI values using smoothed averages (Wilder's smoothing)
    for i in period..changes.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rsi_val = if avg_loss == 0.0 {
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        rsi_values.push(rsi_val);
    }

    rsi_values
}

/// Calculates price changes between consecutive candles.
///
/// Returns a vector of changes where each value is: current_close - previous_close
fn price_changes(candles: &[Candle]) -> Vec<f64> {
    candles
        .windows(2)
        .map(|pair| pair[1].get_close() - pair[0].get_close())
        .collect()
}

/// Separates price changes into gains and losses.
///
/// Returns a tuple of (gains, losses) where:
/// - gains[i] = change if positive, else 0
/// - losses[i] = |change| if negative, else 0
fn gains_and_losses(changes: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let gains: Vec<f64> = changes.iter().map(|&c| if c > 0.0 { c } else { 0.0 }).collect();

    let losses: Vec<f64> = changes
        .iter()
        .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
        .collect();

    (gains, losses)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uptrend_candles() -> Vec<Candle> {
        // Strong uptrend: prices consistently rising
        // Closes: 100, 102, 105, 108, 112, 116, 120, 125, 130, 136, 142, 148, 155, 162, 170
        vec![
            Candle::new(99.0, 101.0, 98.0, 100.0, 1000.0),
            Candle::new(100.0, 103.0, 99.0, 102.0, 1000.0),
            Candle::new(102.0, 106.0, 101.0, 105.0, 1000.0),
            Candle::new(105.0, 109.0, 104.0, 108.0, 1000.0),
            Candle::new(108.0, 113.0, 107.0, 112.0, 1000.0),
            Candle::new(112.0, 117.0, 111.0, 116.0, 1000.0),
            Candle::new(116.0, 121.0, 115.0, 120.0, 1000.0),
            Candle::new(120.0, 126.0, 119.0, 125.0, 1000.0),
            Candle::new(125.0, 131.0, 124.0, 130.0, 1000.0),
            Candle::new(130.0, 137.0, 129.0, 136.0, 1000.0),
            Candle::new(136.0, 143.0, 135.0, 142.0, 1000.0),
            Candle::new(142.0, 149.0, 141.0, 148.0, 1000.0),
            Candle::new(148.0, 156.0, 147.0, 155.0, 1000.0),
            Candle::new(155.0, 163.0, 154.0, 162.0, 1000.0),
            Candle::new(162.0, 171.0, 161.0, 170.0, 1000.0),
        ]
    }

    fn downtrend_candles() -> Vec<Candle> {
        // Strong downtrend: prices consistently falling
        // Closes: 170, 165, 160, 154, 148, 142, 135, 128, 121, 114, 107, 100, 93, 86, 80
        vec![
            Candle::new(172.0, 173.0, 169.0, 170.0, 1000.0),
            Candle::new(170.0, 171.0, 164.0, 165.0, 1000.0),
            Candle::new(165.0, 166.0, 159.0, 160.0, 1000.0),
            Candle::new(160.0, 161.0, 153.0, 154.0, 1000.0),
            Candle::new(154.0, 155.0, 147.0, 148.0, 1000.0),
            Candle::new(148.0, 149.0, 141.0, 142.0, 1000.0),
            Candle::new(142.0, 143.0, 134.0, 135.0, 1000.0),
            Candle::new(135.0, 136.0, 127.0, 128.0, 1000.0),
            Candle::new(128.0, 129.0, 120.0, 121.0, 1000.0),
            Candle::new(121.0, 122.0, 113.0, 114.0, 1000.0),
            Candle::new(114.0, 115.0, 106.0, 107.0, 1000.0),
            Candle::new(107.0, 108.0, 99.0, 100.0, 1000.0),
            Candle::new(100.0, 101.0, 92.0, 93.0, 1000.0),
            Candle::new(93.0, 94.0, 85.0, 86.0, 1000.0),
            Candle::new(86.0, 87.0, 79.0, 80.0, 1000.0),
        ]
    }

    fn sideways_candles() -> Vec<Candle> {
        // Sideways movement: alternating up and down
        // Closes: 100, 102, 100, 103, 101, 104, 102, 105, 103, 106, 104, 107, 105, 108, 106
        vec![
            Candle::new(99.0, 101.0, 98.0, 100.0, 1000.0),
            Candle::new(100.0, 103.0, 99.0, 102.0, 1000.0),
            Candle::new(102.0, 103.0, 99.0, 100.0, 1000.0),
            Candle::new(100.0, 104.0, 99.0, 103.0, 1000.0),
            Candle::new(103.0, 104.0, 100.0, 101.0, 1000.0),
            Candle::new(101.0, 105.0, 100.0, 104.0, 1000.0),
            Candle::new(104.0, 105.0, 101.0, 102.0, 1000.0),
            Candle::new(102.0, 106.0, 101.0, 105.0, 1000.0),
            Candle::new(105.0, 106.0, 102.0, 103.0, 1000.0),
            Candle::new(103.0, 107.0, 102.0, 106.0, 1000.0),
            Candle::new(106.0, 107.0, 103.0, 104.0, 1000.0),
            Candle::new(104.0, 108.0, 103.0, 107.0, 1000.0),
            Candle::new(107.0, 108.0, 104.0, 105.0, 1000.0),
            Candle::new(105.0, 109.0, 104.0, 108.0, 1000.0),
            Candle::new(108.0, 109.0, 105.0, 106.0, 1000.0),
        ]
    }

    #[test]
    fn test_rsi_overbought() {
        let candles = uptrend_candles();
        let result = rsi(&candles, Some(14));
        // Strong uptrend should result in RSI > 70 (overbought)
        assert!(
            result > 70.0,
            "RSI ({}) should be > 70 for strong uptrend",
            result
        );
    }

    #[test]
    fn test_rsi_oversold() {
        let candles = downtrend_candles();
        let result = rsi(&candles, Some(14));
        // Strong downtrend should result in RSI < 30 (oversold)
        assert!(
            result < 30.0,
            "RSI ({}) should be < 30 for strong downtrend",
            result
        );
    }

    #[test]
    fn test_rsi_neutral() {
        let candles = sideways_candles();
        let result = rsi(&candles, Some(14));
        // Sideways movement should result in RSI around 50
        assert!(
            result > 30.0 && result < 70.0,
            "RSI ({}) should be between 30 and 70 for sideways movement",
            result
        );
    }

    #[test]
    fn test_rsi_insufficient_candles() {
        let candles = vec![
            Candle::new(100.0, 105.0, 95.0, 102.0, 1000.0),
            Candle::new(102.0, 108.0, 100.0, 106.0, 1000.0),
        ];
        let result = rsi(&candles, Some(14));
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_rsi_zero_period() {
        let candles = uptrend_candles();
        let result = rsi(&candles, Some(0));
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_rsi_default_period() {
        let candles = uptrend_candles();
        let with_none = rsi(&candles, None);
        let with_14 = rsi(&candles, Some(14));
        assert_eq!(with_none, with_14);
    }

    #[test]
    fn test_price_changes() {
        let candles = vec![
            Candle::new(100.0, 105.0, 95.0, 100.0, 1000.0),
            Candle::new(100.0, 108.0, 98.0, 105.0, 1000.0),
            Candle::new(105.0, 110.0, 102.0, 103.0, 1000.0),
        ];
        let changes = price_changes(&candles);
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0], 5.0); // 105 - 100
        assert_eq!(changes[1], -2.0); // 103 - 105
    }

    #[test]
    fn test_gains_and_losses() {
        let changes = vec![5.0, -3.0, 2.0, -1.0, 4.0];
        let (gains, losses) = gains_and_losses(&changes);

        assert_eq!(gains, vec![5.0, 0.0, 2.0, 0.0, 4.0]);
        assert_eq!(losses, vec![0.0, 3.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_rsi_series_length() {
        let candles = uptrend_candles();
        let series = rsi_series(&candles, Some(5));
        // With 15 candles and period 5, we need 6 candles for first RSI
        // Then we can calculate for remaining 9 candles = 10 values total
        assert_eq!(series.len(), 10);
    }

    #[test]
    fn test_rsi_bounds() {
        // RSI should always be between 0 and 100
        let candles = uptrend_candles();
        let result = rsi(&candles, Some(14));
        assert!(result >= 0.0 && result <= 100.0);

        let candles = downtrend_candles();
        let result = rsi(&candles, Some(14));
        assert!(result >= 0.0 && result <= 100.0);
    }
}
