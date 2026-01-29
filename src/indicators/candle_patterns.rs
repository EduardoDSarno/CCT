use crate::indicators::candle::Candle;
use crate::indicators::timeframe::Timeframe;

const DEFAULT_ATR_RANGE: usize = 14;

pub struct CandlePatterns {
    candles: Vec<Candle>,
    timeframe: Timeframe,
}

impl CandlePatterns {
    pub fn new(candles: Vec<Candle>, timeframe: Timeframe) -> Self {
        Self { candles, timeframe }
    }

    pub fn get_candles(&self) -> &Vec<Candle> {
        &self.candles
    }

    pub fn get_timeframe(&self) -> Timeframe {
        self.timeframe
    }

    pub fn get_candle(&self, index: usize) -> &Candle {
        &self.candles[index]
    }

    // getting previous close from the last candle
    pub fn get_previous_close(&self, index: usize) -> f64 {
        self.get_candle(index - 1).get_close()
    }

    /// Calculates the Average True Range (ATR).
    /// Pass `None` to use the default range of 14, or `Some(n)` for a custom range.
    pub fn average_true_range(&self, range: Option<usize>) -> f64 {
        let range = range.unwrap_or(DEFAULT_ATR_RANGE);

        if range == 0 || self.candles.len() < range {
            return 0.0;
        }

        let beginning_of_range = self.candles.len() - range;
        let mut tr = 0.0;

        for i in beginning_of_range..self.candles.len() {
            tr += self.get_candle(i).true_range(self, i);
        }

        tr / range as f64
    }
}