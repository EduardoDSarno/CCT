//! Binance exchange implementation.
//! See docs/market/BINANCE.md for message formats and details.

use crate::indicators::candle::Candle;
use crate::indicators::timeframe::Timeframe;
use crate::market::market_data::{MarketData, Trade, TradeSide};
use crate::market::message_parser::MessageParser;
use crate::market::streams::Stream;
use crate::market::websocket_client::WebSocketClient;
use serde::Deserialize;

pub const BINANCE_WSS_BASE_ENDPOINT: &str = "wss://stream.binance.com:443/ws";
pub const BINANCE_WSS_FALLBACK_ENDPOINT: &str = "wss://stream.binance.com:9443/ws";
pub const BINANCE_API_BASE_ENDPOINT: &str = "wss://ws-api.binance.com:443/ws-api/v3";
pub const BINANCE_API_FALLBACK_ENDPOINT: &str = "wss://ws-api.binance.com:9443/ws-api/v3";


// This is an example of how to implement MessageParser for an exchange.
// The parser converts Binance-specific JSON into normalized MarketData.
// Key normalization: Binance uses "m" (is_buyer_maker) instead of explicit side,
// so we convert it to TradeSide::Buy/Sell for consistency with other exchanges.


/// Binance-specific message parser.
/// Implements MessageParser to convert Binance JSON -> normalized MarketData.
#[derive(Debug, Clone)]
pub struct BinanceParser;

impl BinanceParser {
    pub fn new() -> Self {
        Self
    }

    /// Parses a Binance kline message into MarketData::Candle.
    /// Normalization: Wraps the simple Candle with symbol/interval/is_closed context.
    fn parse_kline(&self, msg: &str) -> Option<MarketData> {
        let event: BinanceKlineEvent = serde_json::from_str(msg).ok()?;
        let interval = Timeframe::from_str(event.k.i.as_str())?;

        // Create simple Candle (calculation primitive) and wrap with streaming context
        let candle = Candle::new(
            event.k.t,
            event.k.o,
            event.k.h,
            event.k.l,
            event.k.c,
            event.k.v,
        );

        Some(MarketData::Candle {
            symbol: event.s,
            interval,
            data: candle,
            is_closed: event.k.x,
        })
    }

    /// Parses a Binance trade message into MarketData::Trade.
    /// Normalization: Converts Binance's "m" (is_buyer_maker) to explicit TradeSide.
    fn parse_trade(&self, msg: &str) -> Option<MarketData> {
        let event: BinanceTradeEvent = serde_json::from_str(msg).ok()?;

        // Binance uses "m" instead of explicit side - normalize to TradeSide
        let is_buyer_maker = event.m;
        
        // Normalization: m=true means buyer was maker, so taker sold
        // This conversion ensures consistent TradeSide across all exchanges
        let side = if is_buyer_maker {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };

        // Keep is_buyer_maker for Binance-specific use cases
        let trade = Trade::new(
            event.trade_time,
            event.s,
            event.p,
            event.q,
            event.t.to_string(),
            side,
        ).with_buyer_maker(is_buyer_maker);

        Some(MarketData::Trade(trade))
    }
}

impl Default for BinanceParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageParser for BinanceParser {
    fn endpoint(&self) -> &str {
        BINANCE_WSS_BASE_ENDPOINT
    }

    fn fallback_endpoint(&self) -> Option<&str> {
        Some(BINANCE_WSS_FALLBACK_ENDPOINT)
    }

    fn name(&self) -> &'static str {
        "Binance"
    }

    fn format_subscribe(&self, stream: &Stream) -> String {
        let stream_name = match stream {
            Stream::Candles { symbol, interval } => {
                format!("{}@kline_{}", symbol.to_lowercase(), interval.as_str())
            }
            Stream::Trades { symbol } => {
                format!("{}@trade", symbol.to_lowercase())
            }
            Stream::Funding { symbol } => {
                format!("{}@markPrice", symbol.to_lowercase())
            }
            Stream::MarkPrice { symbol } => {
                format!("{}@markPrice", symbol.to_lowercase())
            }
            Stream::OrderBook { symbol, depth } => {
                format!("{}@depth{}", symbol.to_lowercase(), depth)
            }
            Stream::OpenInterest { symbol } => {
                format!("{}@openInterest", symbol.to_lowercase())
            }
            Stream::Liquidations { symbol } => {
                format!("{}@forceOrder", symbol.to_lowercase())
            }
        };

        format!(
            r#"{{"method":"SUBSCRIBE","params":["{}"],"id":1}}"#,
            stream_name
        )
    }

    fn format_unsubscribe(&self, stream: &Stream) -> String {
        let stream_name = match stream {
            Stream::Candles { symbol, interval } => {
                format!("{}@kline_{}", symbol.to_lowercase(), interval.as_str())
            }
            Stream::Trades { symbol } => {
                format!("{}@trade", symbol.to_lowercase())
            }
            Stream::Funding { symbol } => {
                format!("{}@markPrice", symbol.to_lowercase())
            }
            Stream::MarkPrice { symbol } => {
                format!("{}@markPrice", symbol.to_lowercase())
            }
            Stream::OrderBook { symbol, depth } => {
                format!("{}@depth{}", symbol.to_lowercase(), depth)
            }
            Stream::OpenInterest { symbol } => {
                format!("{}@openInterest", symbol.to_lowercase())
            }
            Stream::Liquidations { symbol } => {
                format!("{}@forceOrder", symbol.to_lowercase())
            }
        };

        format!(
            r#"{{"method":"UNSUBSCRIBE","params":["{}"],"id":1}}"#,
            stream_name
        )
    }

    fn parse_message(&self, msg: &str) -> Option<MarketData> {
        // Detect message type by "e" field
        if msg.contains(r#""e":"kline""#) {
            return self.parse_kline(msg);
        }

        if msg.contains(r#""e":"trade""#) {
            return self.parse_trade(msg);
        }

        // TODO: Add more message types
        // - Order book: "e":"depthUpdate"
        // - Mark price/funding: "e":"markPriceUpdate"

        None // Unknown or control message
    }
}

#[derive(Debug, Deserialize)]
struct BinanceKlineEvent {
    s: String,
    k: BinanceKline,
}

#[derive(Debug, Deserialize)]
struct BinanceKline {
    t: u64,
    i: String,
    #[serde(deserialize_with = "de_f64")]
    o: f64,
    #[serde(deserialize_with = "de_f64")]
    h: f64,
    #[serde(deserialize_with = "de_f64")]
    l: f64,
    #[serde(deserialize_with = "de_f64")]
    c: f64,
    #[serde(deserialize_with = "de_f64")]
    v: f64,
    x: bool,
}

#[derive(Debug, Deserialize)]
struct BinanceTradeEvent {
    s: String,
    t: u64,
    #[serde(deserialize_with = "de_f64")]
    p: f64,
    #[serde(deserialize_with = "de_f64")]
    q: f64,
    #[serde(rename = "T")]
    trade_time: u64,
    m: bool,
}

fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct F64Visitor;

    impl<'de> serde::de::Visitor<'de> for F64Visitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or number representing a float")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
            Ok(value as f64)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value as f64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse::<f64>().map_err(E::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse::<f64>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(F64Visitor)
}

pub type BinanceClient = WebSocketClient<BinanceParser>;

pub fn new_binance_client() -> BinanceClient {
    WebSocketClient::new(BinanceParser::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_subscribe_candles() {
        let parser = BinanceParser::new();
        let stream = Stream::candles("BTCUSDT", Timeframe::M1);
        let msg = parser.format_subscribe(&stream);
        
        assert!(msg.contains("SUBSCRIBE"));
        assert!(msg.contains("btcusdt@kline_1m"));
    }

    #[test]
    fn test_format_subscribe_trades() {
        let parser = BinanceParser::new();
        let stream = Stream::trades("ETHUSDT");
        let msg = parser.format_subscribe(&stream);
        
        assert!(msg.contains("SUBSCRIBE"));
        assert!(msg.contains("ethusdt@trade"));
    }

    #[test]
    fn test_format_unsubscribe_candles() {
        let parser = BinanceParser::new();
        let stream = Stream::candles("BTCUSDT", Timeframe::M5);
        let msg = parser.format_unsubscribe(&stream);
        
        assert!(msg.contains("UNSUBSCRIBE"));
        assert!(msg.contains("btcusdt@kline_5m"));
    }

    #[test]
    fn test_parse_kline_message() {
        let parser = BinanceParser::new();
        
        let msg = r#"{"e":"kline","E":1638747660000,"s":"BTCUSDT","k":{"t":1638747660000,"T":1638747719999,"s":"BTCUSDT","i":"1m","f":100,"L":200,"o":"50000.00","c":"50100.00","h":"50200.00","l":"49900.00","v":"100.5","n":100,"x":false,"q":"1.0000","V":"500","Q":"0.500","B":"123456"}}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_some());
        
        match result.unwrap() {
            MarketData::Candle { symbol, interval, data, is_closed } => {
                assert_eq!(symbol, "BTCUSDT");
                assert_eq!(interval, Timeframe::M1);
                assert_eq!(data.get_timestamp(), 1638747660000);
                assert_eq!(data.get_open(), 50000.00);
                assert_eq!(data.get_close(), 50100.00);
                assert_eq!(data.get_high(), 50200.00);
                assert_eq!(data.get_low(), 49900.00);
                assert_eq!(data.get_volume(), 100.5);
                assert!(!is_closed);
            }
            _ => panic!("Expected MarketData::Candle"),
        }
    }

    #[test]
    fn test_parse_kline_closed() {
        let parser = BinanceParser::new();
        
        let msg = r#"{"e":"kline","E":1672515782136,"s":"ETHUSDT","k":{"t":1672515780000,"T":1672515839999,"s":"ETHUSDT","i":"5m","o":"3000.00","c":"3050.00","h":"3100.00","l":"2950.00","v":"500.0","x":true}}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_some());
        
        if let Some(MarketData::Candle { is_closed, .. }) = result {
            assert!(is_closed);
        } else {
            panic!("Expected MarketData::Candle");
        }
    }

    #[test]
    fn test_parse_trade_message() {
        let parser = BinanceParser::new();
        
        // m:false = buyer is taker = BUY
        let msg = r#"{"e":"trade","E":1672515782136,"s":"BNBBTC","t":12345,"p":"0.001","q":"100","T":1672515782136,"m":true,"M":true}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_some());
        
        match result.unwrap() {
            MarketData::Trade(trade) => {
                assert_eq!(trade.symbol, "BNBBTC");
                assert_eq!(trade.price, 0.001);
                assert_eq!(trade.quantity, 100.0);
                assert_eq!(trade.trade_id, "12345");
                assert_eq!(trade.side, TradeSide::Sell);
                assert_eq!(trade.is_buyer_maker, Some(true));
            }
            _ => panic!("Expected MarketData::Trade"),
        }
    }

    #[test]
    fn test_parse_trade_sell() {
        let parser = BinanceParser::new();
        
        // m:true = buyer is maker = SELL
        let msg = r#"{"e":"trade","E":123456789,"s":"ETHUSDT","t":67890,"p":"3000.00","q":"1.0","T":123456785,"m":true}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_some());
        
        if let Some(MarketData::Trade(trade)) = result {
            assert_eq!(trade.side, TradeSide::Sell);
            assert_eq!(trade.is_buyer_maker, Some(true));
        } else {
            panic!("Expected MarketData::Trade");
        }
    }

    #[test]
    fn test_parse_subscription_confirmation() {
        let parser = BinanceParser::new();
        
        let msg = r#"{"result":null,"id":1}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_unknown_message() {
        let parser = BinanceParser::new();
        
        let msg = r#"{"e":"unknown","data":"something"}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_none());
    }
}
