//! Candlestick pattern detection (Doji, Hammer, Engulfing, etc.)
//!
//! This module contains pattern recognition logic for common
//! candlestick patterns used in technical analysis.
//!
//! Pattern detection uses the component methods from `Candle`:
//! - `body()`, `body_abs()` - candle body size
//! - `range()` - full candle range (high - low)
//! - `upper_wick()`, `lower_wick()` - wick/shadow sizes
//! - `body_ratio()` - body size relative to range
//! - `is_bullish()`, `is_bearish()` - candle direction

use crate::indicators::candle::Candle;
use crate::indicators::timeframe::Timeframe;

/// A collection of candles with associated timeframe for pattern detection
pub struct CandlePatterns {
    candles: Vec<Candle>,
    timeframe: Timeframe,
}

impl CandlePatterns {
    pub fn new(candles: Vec<Candle>, timeframe: Timeframe) -> Self {
        Self { candles, timeframe }
    }

    pub fn get_candles(&self) -> &[Candle] {
        &self.candles
    }

    pub fn get_timeframe(&self) -> Timeframe {
        self.timeframe
    }

    pub fn get_candle(&self, index: usize) -> Option<&Candle> {
        self.candles.get(index)
    }

    pub fn len(&self) -> usize {
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    // ========== Single Candle Patterns ==========

    /// Detects a Doji pattern at the given index.
    ///
    /// A Doji has a very small body relative to its range,
    /// indicating indecision in the market.
    pub fn is_doji(&self, index: usize) -> bool {
        if let Some(candle) = self.get_candle(index) {
            // Doji: body is less than 10% of the range
            candle.body_ratio() < 0.1 && candle.range() > 0.0
        } else {
            false
        }
    }

    /// Detects a Hammer pattern at the given index.
    ///
    /// A Hammer has a small body at the top with a long lower wick
    /// (at least 2x the body size) and little/no upper wick.
    /// Typically appears after a downtrend as a potential reversal signal.
    pub fn is_hammer(&self, index: usize) -> bool {
        if let Some(candle) = self.get_candle(index) {
            let body = candle.body_abs();
            let lower_wick = candle.lower_wick();
            let upper_wick = candle.upper_wick();

            // Hammer: long lower wick, small upper wick, body at top
            body > 0.0
                && lower_wick >= body * 2.0
                && upper_wick <= body * 0.5
        } else {
            false
        }
    }

    /// Detects an Inverted Hammer pattern at the given index.
    ///
    /// An Inverted Hammer has a small body at the bottom with a long upper wick
    /// (at least 2x the body size) and little/no lower wick.
    pub fn is_inverted_hammer(&self, index: usize) -> bool {
        if let Some(candle) = self.get_candle(index) {
            let body = candle.body_abs();
            let lower_wick = candle.lower_wick();
            let upper_wick = candle.upper_wick();

            // Inverted Hammer: long upper wick, small lower wick, body at bottom
            body > 0.0
                && upper_wick >= body * 2.0
                && lower_wick <= body * 0.5
        } else {
            false
        }
    }

    /// Detects a Marubozu pattern at the given index.
    ///
    /// A Marubozu is a candle with no (or very small) wicks,
    /// indicating strong momentum in the direction of the candle.
    pub fn is_marubozu(&self, index: usize) -> bool {
        if let Some(candle) = self.get_candle(index) {
            let body = candle.body_abs();
            let range = candle.range();

            // Marubozu: body is at least 95% of the range
            range > 0.0 && body / range >= 0.95
        } else {
            false
        }
    }

    // ========== Two Candle Patterns ==========

    /// Detects a Bullish Engulfing pattern at the given index.
    ///
    /// A Bullish Engulfing occurs when a bullish candle's body
    /// completely engulfs the previous bearish candle's body.
    pub fn is_bullish_engulfing(&self, index: usize) -> bool {
        if index == 0 {
            return false;
        }

        if let (Some(prev), Some(curr)) = (self.get_candle(index - 1), self.get_candle(index)) {
            prev.is_bearish()
                && curr.is_bullish()
                && curr.get_open() <= prev.get_close()
                && curr.get_close() >= prev.get_open()
        } else {
            false
        }
    }

    /// Detects a Bearish Engulfing pattern at the given index.
    ///
    /// A Bearish Engulfing occurs when a bearish candle's body
    /// completely engulfs the previous bullish candle's body.
    pub fn is_bearish_engulfing(&self, index: usize) -> bool {
        if index == 0 {
            return false;
        }

        if let (Some(prev), Some(curr)) = (self.get_candle(index - 1), self.get_candle(index)) {
            prev.is_bullish()
                && curr.is_bearish()
                && curr.get_open() >= prev.get_close()
                && curr.get_close() <= prev.get_open()
        } else {
            false
        }
    }

    // ========== Three Candle Patterns ==========

    /// Detects a Morning Star pattern at the given index (bullish reversal).
    ///
    /// The index should point to the third (final) candle of the pattern.
    ///
    /// Morning Star structure (3 candles):
    /// 1. Strong bearish candle (sellers in control)
    /// 2. Small body candle - doji or spinning top (indecision)
    /// 3. Strong bullish candle that closes into candle 1's body (buyers take control)
    ///
    /// Meaning: After a downtrend, selling pressure weakens and buyers take control.
    pub fn is_morning_star(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }

        let first = match self.get_candle(index - 2) {
            Some(c) => c,
            None => return false,
        };
        let second = match self.get_candle(index - 1) {
            Some(c) => c,
            None => return false,
        };
        let third = match self.get_candle(index) {
            Some(c) => c,
            None => return false,
        };

        // First candle: strong bearish (body > 50% of range)
        let first_is_strong_bearish = first.is_bearish() && first.body_ratio() > 0.5;

        // Second candle: small body (indecision - body < 30% of range)
        let second_is_small = second.body_ratio() < 0.3;

        // Third candle: strong bullish (body > 50% of range)
        let third_is_strong_bullish = third.is_bullish() && third.body_ratio() > 0.5;

        // Third candle closes into the first candle's body
        // (closes above the midpoint of the first candle's body)
        let first_body_midpoint = (first.get_open() + first.get_close()) / 2.0;
        let third_closes_into_first = third.get_close() > first_body_midpoint;

        first_is_strong_bearish
            && second_is_small
            && third_is_strong_bullish
            && third_closes_into_first
    }

    /// Detects an Evening Star pattern at the given index (bearish reversal).
    ///
    /// The index should point to the third (final) candle of the pattern.
    ///
    /// Evening Star structure (3 candles):
    /// 1. Strong bullish candle (buyers in control)
    /// 2. Small body candle - doji or spinning top (indecision)
    /// 3. Strong bearish candle that closes into candle 1's body (sellers take control)
    ///
    /// Meaning: After an uptrend, buying pressure weakens and sellers take control.
    pub fn is_evening_star(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }

        let first = match self.get_candle(index - 2) {
            Some(c) => c,
            None => return false,
        };
        let second = match self.get_candle(index - 1) {
            Some(c) => c,
            None => return false,
        };
        let third = match self.get_candle(index) {
            Some(c) => c,
            None => return false,
        };

        // First candle: strong bullish (body > 50% of range)
        let first_is_strong_bullish = first.is_bullish() && first.body_ratio() > 0.5;

        // Second candle: small body (indecision - body < 30% of range)
        let second_is_small = second.body_ratio() < 0.3;

        // Third candle: strong bearish (body > 50% of range)
        let third_is_strong_bearish = third.is_bearish() && third.body_ratio() > 0.5;

        // Third candle closes into the first candle's body
        // (closes below the midpoint of the first candle's body)
        let first_body_midpoint = (first.get_open() + first.get_close()) / 2.0;
        let third_closes_into_first = third.get_close() < first_body_midpoint;

        first_is_strong_bullish
            && second_is_small
            && third_is_strong_bearish
            && third_closes_into_first
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle::new(0, open, high, low, close, 1000.0)
    }

    #[test]
    fn test_is_doji() {
        // Doji: open and close are very close, but has range
        let candles = vec![make_candle(100.0, 105.0, 95.0, 100.5)];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_doji(0));
    }

    #[test]
    fn test_is_not_doji() {
        // Not a doji: significant body
        let candles = vec![make_candle(100.0, 110.0, 95.0, 108.0)];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(!patterns.is_doji(0));
    }

    #[test]
    fn test_is_hammer() {
        // Hammer: small body at top, long lower wick
        // Body: 98-100 = 2, Lower wick: 98-90 = 8, Upper wick: 101-100 = 1
        let candles = vec![make_candle(98.0, 101.0, 90.0, 100.0)];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_hammer(0));
    }

    #[test]
    fn test_is_inverted_hammer() {
        // Inverted Hammer: small body at bottom, long upper wick
        // Body: 100-98 = 2, Upper wick: 110-100 = 10, Lower wick: 98-97 = 1
        let candles = vec![make_candle(98.0, 110.0, 97.0, 100.0)];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_inverted_hammer(0));
    }

    #[test]
    fn test_is_marubozu() {
        // Marubozu: body fills nearly all of range
        let candles = vec![make_candle(100.0, 110.0, 100.0, 110.0)];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_marubozu(0));
    }

    #[test]
    fn test_is_bullish_engulfing() {
        // Bearish candle followed by larger bullish candle
        let candles = vec![
            make_candle(105.0, 106.0, 100.0, 101.0), // Bearish: open 105, close 101
            make_candle(100.0, 110.0, 99.0, 108.0),  // Bullish: open 100, close 108 (engulfs)
        ];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_bullish_engulfing(1));
    }

    #[test]
    fn test_is_bearish_engulfing() {
        // Bullish candle followed by larger bearish candle
        let candles = vec![
            make_candle(100.0, 106.0, 99.0, 105.0),  // Bullish: open 100, close 105
            make_candle(106.0, 107.0, 98.0, 99.0),   // Bearish: open 106, close 99 (engulfs)
        ];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_bearish_engulfing(1));
    }

    #[test]
    fn test_invalid_index() {
        let candles = vec![make_candle(100.0, 105.0, 95.0, 102.0)];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(!patterns.is_doji(10));
        assert!(!patterns.is_bullish_engulfing(0)); // Can't check engulfing on first candle
    }

    #[test]
    fn test_is_morning_star() {
        // Morning Star: bullish reversal pattern
        // 1. Strong bearish candle: open 110, close 100 (body = 10, range = 12)
        // 2. Small body (doji/spinner): open 99, close 98 (body = 1, range = 4)
        // 3. Strong bullish candle: open 99, close 108 (body = 9, range = 10)
        //    Third closes above first's midpoint (105)
        let candles = vec![
            make_candle(110.0, 112.0, 100.0, 100.0), // Strong bearish
            make_candle(99.0, 100.0, 96.0, 98.0),    // Small body (indecision)
            make_candle(99.0, 109.0, 99.0, 108.0),   // Strong bullish, closes into first
        ];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_morning_star(2));
    }

    #[test]
    fn test_is_not_morning_star_weak_third() {
        // Not a Morning Star: third candle doesn't close into first's body
        let candles = vec![
            make_candle(110.0, 112.0, 100.0, 100.0), // Strong bearish
            make_candle(99.0, 100.0, 96.0, 98.0),    // Small body
            make_candle(99.0, 104.0, 99.0, 103.0),   // Bullish but closes below midpoint (105)
        ];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(!patterns.is_morning_star(2));
    }

    #[test]
    fn test_is_evening_star() {
        // Evening Star: bearish reversal pattern
        // 1. Strong bullish candle: open 100, close 110 (body = 10, range = 12)
        // 2. Small body (doji/spinner): open 111, close 112 (body = 1, range = 4)
        // 3. Strong bearish candle: open 111, close 102 (body = 9, range = 10)
        //    Third closes below first's midpoint (105)
        let candles = vec![
            make_candle(100.0, 112.0, 100.0, 110.0), // Strong bullish
            make_candle(111.0, 114.0, 110.0, 112.0), // Small body (indecision)
            make_candle(111.0, 111.0, 101.0, 102.0), // Strong bearish, closes into first
        ];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(patterns.is_evening_star(2));
    }

    #[test]
    fn test_is_not_evening_star_weak_third() {
        // Not an Evening Star: third candle doesn't close into first's body
        let candles = vec![
            make_candle(100.0, 112.0, 100.0, 110.0), // Strong bullish
            make_candle(111.0, 114.0, 110.0, 112.0), // Small body
            make_candle(111.0, 111.0, 106.0, 107.0), // Bearish but closes above midpoint (105)
        ];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(!patterns.is_evening_star(2));
    }

    #[test]
    fn test_morning_star_insufficient_candles() {
        let candles = vec![
            make_candle(100.0, 105.0, 95.0, 102.0),
            make_candle(102.0, 108.0, 100.0, 106.0),
        ];
        let patterns = CandlePatterns::new(candles, Timeframe::H1);
        assert!(!patterns.is_morning_star(1)); // Need 3 candles
        assert!(!patterns.is_evening_star(1));
    }
}