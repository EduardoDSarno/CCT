//! Market data types for WebSocket streams.
//! See docs/market/MARKET_DATA.md for detailed documentation.

use crate::indicators::candle::Candle;
use crate::indicators::timeframe::Timeframe;


// Fields use Option<T> when only some exchanges provide them.
// This allows adding new exchanges without breaking existing code - just set
// exchange-specific fields to None when not available.
// Examples: is_buyer_maker (Binance), num_orders (Hyperliquid), sequence (varies)

/// Side of a trade (buyer or seller initiated).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// A single price level in an order book.
#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
    // Option<T> because only Hyperliquid provides order count at each level
    pub num_orders: Option<u32>,
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64) -> Self {
        Self {
            price,
            quantity,
            num_orders: None,
        }
    }

    /// Creates a new price level with order count (Hyperliquid).
    pub fn with_order_count(price: f64, quantity: f64, num_orders: u32) -> Self {
        Self {
            price,
            quantity,
            num_orders: Some(num_orders),
        }
    }
}

/// A single trade event from the exchange.
/// Design: Trade has symbol baked in because trades are discrete events -
/// each happens once, for one symbol. You can't process a trade without knowing its symbol.
#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: u64,
    pub symbol: String,  // baked in - trades are discrete events that need symbol context
    pub price: f64,
    pub quantity: f64,
    pub trade_id: String,
    pub side: TradeSide,
    // Option<T> because only Binance provides this field
    // true = buyer was maker, so taker sold; false = buyer was taker, so taker bought
    pub is_buyer_maker: Option<bool>,
}

impl Trade {
    pub fn new(
        timestamp: u64,
        symbol: impl Into<String>,
        price: f64,
        quantity: f64,
        trade_id: impl Into<String>,
        side: TradeSide,
    ) -> Self {
        Self {
            timestamp,
            symbol: symbol.into(),
            price,
            quantity,
            trade_id: trade_id.into(),
            side,
            is_buyer_maker: None,
        }
    }

    // Binance Specific
    pub fn with_buyer_maker(mut self, is_buyer_maker: bool) -> Self {
        self.is_buyer_maker = Some(is_buyer_maker);
        self
    }
}

/// Order book snapshot or delta update.
/// Design: Like Trade, OrderBookUpdate has symbol baked in - it's a discrete event.
#[derive(Debug, Clone)]
pub struct OrderBookUpdate {
    pub timestamp: u64,
    pub symbol: String,  // baked in - order book updates are discrete events
    /// Bid levels (buy orders), sorted by price descending
    pub bids: Vec<PriceLevel>,
    /// Ask levels (sell orders), sorted by price ascending
    pub asks: Vec<PriceLevel>,
    /// True = full snapshot, False = delta update
    pub is_snapshot: bool,
    // Option<T> because not all exchanges provide sequence numbers
    pub sequence: Option<u64>,
}

impl OrderBookUpdate {
    pub fn snapshot(
        timestamp: u64,
        symbol: impl Into<String>,
        bids: Vec<PriceLevel>,
        asks: Vec<PriceLevel>,
    ) -> Self {
        Self {
            timestamp,
            symbol: symbol.into(),
            bids,
            asks,
            is_snapshot: true,
            sequence: None,
        }
    }

    pub fn delta(
        timestamp: u64,
        symbol: impl Into<String>,
        bids: Vec<PriceLevel>,
        asks: Vec<PriceLevel>,
    ) -> Self {
        Self {
            timestamp,
            symbol: symbol.into(),
            bids,
            asks,
            is_snapshot: false,
            sequence: None,
        }
    }

    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }
}

/// Funding rate event for perpetual futures.
/// Design: Like Trade, FundingRate has symbol baked in - it's a discrete event.
#[derive(Debug, Clone)]
pub struct FundingRate {
    pub timestamp: u64,
    pub symbol: String,  // baked in - funding events are discrete events
    /// Positive = longs pay shorts
    pub rate: f64,
    // Option<T> because not all exchanges provide these fields
    pub next_funding_time: Option<u64>,
    pub mark_price: Option<f64>,
}

impl FundingRate {
    pub fn new(timestamp: u64, symbol: impl Into<String>, rate: f64) -> Self {
        Self {
            timestamp,
            symbol: symbol.into(),
            rate,
            next_funding_time: None,
            mark_price: None,
        }
    }

    pub fn with_next_funding_time(mut self, next_funding_time: u64) -> Self {
        self.next_funding_time = Some(next_funding_time);
        self
    }

    pub fn with_mark_price(mut self, mark_price: f64) -> Self {
        self.mark_price = Some(mark_price);
        self
    }
}


// - Candle is a *calculation primitive* used by indicators (is_doji, atr, ema).
//   It doesn't need symbol/interval for calculations - that's streaming context.
//   The Candle struct in indicators/candle.rs stays simple for clean indicator code.
//
// - Trade/OrderBook/Funding are *discrete events* - each happens once, for one symbol.
//   They naturally contain their symbol because you can't process them without it.

/// Unified market data enum for all stream types.
/// Allows a single channel to carry all types of market data.
#[derive(Debug, Clone)]
pub enum MarketData {
    /// Candle wrapped with streaming context (symbol, interval, is_closed).
    /// The inner Candle is a calculation primitive - doesn't need symbol for indicators.
    /// WARNING: If is_closed=false, candle is still updating - don't use for calculations yet.
    Candle {
        symbol: String,    // streaming context, not needed for indicator calculations
        interval: Timeframe,  // streaming context, not needed for indicator calculations
        data: Candle,      // the actual calculation primitive
        is_closed: bool,   // IMPORTANT: only use for calculations when true
    },
    // These types have symbol baked in - they're discrete events
    Trade(Trade),
    OrderBook(OrderBookUpdate),
    Funding(FundingRate),
}

impl MarketData {
    pub fn symbol(&self) -> &str {
        match self {
            MarketData::Candle { symbol, .. } => symbol,
            MarketData::Trade(trade) => &trade.symbol,
            MarketData::OrderBook(book) => &book.symbol,
            MarketData::Funding(funding) => &funding.symbol,
        }
    }

    pub fn is_candle(&self) -> bool {
        matches!(self, MarketData::Candle { .. })
    }

    pub fn is_trade(&self) -> bool {
        matches!(self, MarketData::Trade(_))
    }

    pub fn is_order_book(&self) -> bool {
        matches!(self, MarketData::OrderBook(_))
    }

    pub fn is_funding(&self) -> bool {
        matches!(self, MarketData::Funding(_))
    }

    pub fn as_candle(&self) -> Option<(&str, Timeframe, &Candle, bool)> {
        match self {
            MarketData::Candle {
                symbol,
                interval,
                data,
                is_closed,
            } => Some((symbol, *interval, data, *is_closed)),
            _ => None,
        }
    }

    pub fn as_trade(&self) -> Option<&Trade> {
        match self {
            MarketData::Trade(trade) => Some(trade),
            _ => None,
        }
    }

    pub fn as_order_book(&self) -> Option<&OrderBookUpdate> {
        match self {
            MarketData::OrderBook(book) => Some(book),
            _ => None,
        }
    }

    pub fn as_funding(&self) -> Option<&FundingRate> {
        match self {
            MarketData::Funding(funding) => Some(funding),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_side_equality() {
        assert_eq!(TradeSide::Buy, TradeSide::Buy);
        assert_ne!(TradeSide::Buy, TradeSide::Sell);
    }

    #[test]
    fn test_price_level_creation() {
        let level = PriceLevel::new(50000.0, 1.5);
        assert_eq!(level.price, 50000.0);
        assert_eq!(level.quantity, 1.5);
        assert!(level.num_orders.is_none());

        let level_with_count = PriceLevel::with_order_count(50000.0, 1.5, 10);
        assert_eq!(level_with_count.num_orders, Some(10));
    }

    #[test]
    fn test_trade_creation() {
        let trade = Trade::new(
            1638747660000,
            "BTCUSDT",
            50000.0,
            0.5,
            "12345",
            TradeSide::Buy,
        );
        assert_eq!(trade.timestamp, 1638747660000);
        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.price, 50000.0);
        assert_eq!(trade.quantity, 0.5);
        assert_eq!(trade.trade_id, "12345");
        assert_eq!(trade.side, TradeSide::Buy);
        assert!(trade.is_buyer_maker.is_none());

        let trade_with_maker = trade.with_buyer_maker(true);
        assert_eq!(trade_with_maker.is_buyer_maker, Some(true));
    }

    #[test]
    fn test_order_book_creation() {
        let bids = vec![PriceLevel::new(49900.0, 2.0)];
        let asks = vec![PriceLevel::new(50100.0, 1.5)];

        let snapshot = OrderBookUpdate::snapshot(1638747660000, "BTCUSDT", bids.clone(), asks.clone());
        assert!(snapshot.is_snapshot);
        assert!(snapshot.sequence.is_none());

        let delta = OrderBookUpdate::delta(1638747660000, "BTCUSDT", bids, asks)
            .with_sequence(12345);
        assert!(!delta.is_snapshot);
        assert_eq!(delta.sequence, Some(12345));
    }

    #[test]
    fn test_funding_rate_creation() {
        let funding = FundingRate::new(1638747660000, "BTCUSDT", 0.0001)
            .with_next_funding_time(1638748800000)
            .with_mark_price(50000.0);

        assert_eq!(funding.rate, 0.0001);
        assert_eq!(funding.next_funding_time, Some(1638748800000));
        assert_eq!(funding.mark_price, Some(50000.0));
    }

    #[test]
    fn test_market_data_symbol() {
        let candle = Candle::new(0, 100.0, 110.0, 90.0, 105.0, 1000.0);
        let md_candle = MarketData::Candle {
            symbol: "BTCUSDT".to_string(),
            interval: Timeframe::M1,
            data: candle,
            is_closed: true,
        };
        assert_eq!(md_candle.symbol(), "BTCUSDT");

        let trade = Trade::new(0, "ETHUSDT", 3000.0, 1.0, "1", TradeSide::Buy);
        let md_trade = MarketData::Trade(trade);
        assert_eq!(md_trade.symbol(), "ETHUSDT");
    }

    #[test]
    fn test_market_data_type_checks() {
        let candle = Candle::new(0, 100.0, 110.0, 90.0, 105.0, 1000.0);
        let md = MarketData::Candle {
            symbol: "BTCUSDT".to_string(),
            interval: Timeframe::M1,
            data: candle,
            is_closed: true,
        };

        assert!(md.is_candle());
        assert!(!md.is_trade());
        assert!(!md.is_order_book());
        assert!(!md.is_funding());
    }

    #[test]
    fn test_market_data_as_candle() {
        let candle = Candle::new(1000, 100.0, 110.0, 90.0, 105.0, 1000.0);
        let md = MarketData::Candle {
            symbol: "BTCUSDT".to_string(),
            interval: Timeframe::M5,
            data: candle,
            is_closed: false,
        };

        let (symbol, interval, data, is_closed) = md.as_candle().unwrap();
        assert_eq!(symbol, "BTCUSDT");
        assert_eq!(interval, Timeframe::M5);
        assert_eq!(data.get_open(), 100.0);
        assert!(!is_closed);
    }
}
