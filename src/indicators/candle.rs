use super::candle_patterns::CandlePatterns;

#[derive(Debug, Clone, Copy)]
pub struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
        }
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

    // true reange from a candle is calculated by the following the commum formula
    pub fn true_range(&self, candle_patterns: &CandlePatterns, index: usize) -> f64 {
        if index == 0 {
            return self.high - self.low;
        }

        let prev_close = candle_patterns.get_previous_close(index);
        (self.high - self.low)
            .max((self.high - prev_close).abs())
            .max((self.low - prev_close).abs())
    }

    
}