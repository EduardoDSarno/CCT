//! Stream types for WebSocket subscriptions.

/// Represents different types of market data streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stream {
    /// Candlestick/Kline data stream
    Candles { symbol: String, interval: String },
    /// Real-time trade stream
    Trades { symbol: String },
    /// Funding rate stream (futures)
    Funding { symbol: String },

    // Future use
    /// Mark price stream (futures)
    MarkPrice { symbol: String },
    /// Order book depth stream
    OrderBook { symbol: String, depth: String },
    /// Open interest stream (futures)
    OpenInterest { symbol: String },
    /// Liquidation stream (futures)
    Liquidations { symbol: String },
}

impl Stream {
    /// Creates a new candles stream subscription.
    pub fn candles(symbol: impl Into<String>, interval: impl Into<String>) -> Self {
        Self::Candles {
            symbol: symbol.into(),
            interval: interval.into(),
        }
    }

    /// Creates a new trades stream subscription.
    pub fn trades(symbol: impl Into<String>) -> Self {
        Self::Trades {
            symbol: symbol.into(),
        }
    }

    /// Returns the symbol for this stream.
    pub fn symbol(&self) -> &str {
        match self {
            Stream::Candles { symbol, .. } => symbol,
            Stream::Trades { symbol } => symbol,
            Stream::Funding { symbol } => symbol,
            Stream::MarkPrice { symbol } => symbol,
            Stream::OrderBook { symbol, .. } => symbol,
            Stream::OpenInterest { symbol } => symbol,
            Stream::Liquidations { symbol } => symbol,
        }
    }
}