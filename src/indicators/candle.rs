//! Candle (OHLCV) data structure with timestamp

/// Represents a single candlestick with OHLCV data and timestamp.
///
/// The timestamp is stored as Unix time in milliseconds, which is the format
/// used by most cryptocurrency exchanges (Binance, Coinbase, etc.).
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    /// Unix timestamp in milliseconds (candle open time)
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    /// Creates a new Candle.
    ///
    /// `timestamp` should be Unix time in milliseconds (candle open time).
    /// Use `0` for the timestamp if not available (e.g., in tests).
    pub fn new(
        timestamp: u64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        debug_assert!(high >= low, "candle high must be >= low");
        debug_assert!(open >= low && open <= high, "candle open must be within [low, high]");
        debug_assert!(close >= low && close <= high, "candle close must be within [low, high]");

        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Returns the candle's timestamp (Unix time in milliseconds).
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn get_open(&self) -> f64 {
        self.open
    }

    pub fn get_high(&self) -> f64 {
        self.high
    }

    pub fn get_low(&self) -> f64 {
        self.low
    }

    pub fn get_close(&self) -> f64 {
        self.close
    }

    pub fn get_volume(&self) -> f64 {
        self.volume
    }

    /// Returns true if this candle has a valid timestamp (non-zero).
    pub fn has_timestamp(&self) -> bool {
        self.timestamp > 0
    }

    /// Candle Component Calculations
    /// Returns the body size (close - open).
    ///
    /// Positive for green candles, negative for red candles.
    pub fn body(&self) -> f64 {
        self.close - self.open
    }

    /// Returns the absolute body size.
    pub fn body_abs(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Returns the full range of the candle (high - low).
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Returns the upper wick/shadow size.
    ///
    /// Distance from the high to the top of the body.
    pub fn upper_wick(&self) -> f64 {
        self.high - self.close.max(self.open)
    }

    /// Returns the lower wick/shadow size.
    ///
    /// Distance from the bottom of the body to the low.
    pub fn lower_wick(&self) -> f64 {
        self.close.min(self.open) - self.low
    }

    /// Returns true if this is a green candle (close > open).
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Returns true if this is a red candle (close < open).
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Returns the body-to-range ratio (0.0 to 1.0).
    ///
    /// A small ratio indicates a doji-like candle.
    /// Returns 0.0 if range is zero (to avoid division by zero).
    pub fn body_ratio(&self) -> f64 {
        let range = self.range();
        if range == 0.0 {
            0.0
        } else {
            self.body_abs() / range
        }
    }
}
