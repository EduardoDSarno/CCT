//! Candlestick pattern detection (Doji, Hammer, Engulfing, etc.)
//!
//! This module will contain pattern recognition logic for common
//! candlestick patterns used in technical analysis.

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

    // TODO: Add candlestick pattern detection methods:
    // - is_doji(index) -> bool
    // - is_hammer(index) -> bool
    // - is_engulfing(index) -> bool
    // - is_morning_star(index) -> bool
    // - etc.
}