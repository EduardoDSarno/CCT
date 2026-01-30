//! Stream types for WebSocket subscriptions.

use crate::indicators::timeframe::Timeframe;

/// Represents different types of market data streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stream {
    /// Candlestick/Kline data stream
    Candles { symbol: String, interval: Timeframe },
    /// Real-time trade stream
    Trades { symbol: String },
    /// Funding rate stream (futures).
    /// Note: Some exchanges (e.g., Binance) provide funding via the mark price stream.
    Funding { symbol: String },

    // Future use
    /// Mark price stream (futures).
    /// Some exchanges may map this to the same underlying channel as funding.
    MarkPrice { symbol: String },
    /// Order book depth stream
    OrderBook { symbol: String, depth: u16 },
    /// Open interest stream (futures)
    OpenInterest { symbol: String },
    /// Liquidation stream (futures)
    Liquidations { symbol: String },
}

impl Stream {
    /// Creates a new candles stream subscription.
    pub fn candles(symbol: impl Into<String>, interval: Timeframe) -> Self {
        Self::Candles {
            symbol: symbol.into(),
            interval,
        }
    }

    /// Creates a new trades stream subscription.
    pub fn trades(symbol: impl Into<String>) -> Self {
        Self::Trades {
            symbol: symbol.into(),
        }
    }

    /// Creates a new order book stream subscription.
    pub fn order_book(symbol: impl Into<String>, depth: u16) -> Self {
        debug_assert!(depth > 0, "order book depth must be greater than zero");
        Self::OrderBook {
            symbol: symbol.into(),
            depth,
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
