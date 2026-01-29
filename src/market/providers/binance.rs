//! Binance exchange implementation.
//! See docs/market/BINANCE.md for message formats and details.

use crate::indicators::candle::Candle;
use crate::market::market_data::{MarketData, Trade, TradeSide};
use crate::market::message_parser::MessageParser;
use crate::market::streams::Stream;
use crate::market::websocket_client::WebSocketClient;

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
        let symbol = extract_json_string(msg, r#""s":""#)?;
        
        // Binance nests kline data in a "k" object
        let k_start = msg.find(r#""k":"#)?;
        let k_section = &msg[k_start..];

        let interval = extract_json_string(k_section, r#""i":""#)?;
        let timestamp = extract_json_number(k_section, r#""t":"#)? as u64;
        let open = extract_json_number(k_section, r#""o":""#)?;
        let high = extract_json_number(k_section, r#""h":""#)?;
        let low = extract_json_number(k_section, r#""l":""#)?;
        let close = extract_json_number(k_section, r#""c":""#)?;
        let volume = extract_json_number(k_section, r#""v":""#)?;
        // Binance "x" field indicates if candle is closed (final)
        let is_closed = k_section.contains(r#""x":true"#);

        // Create simple Candle (calculation primitive) and wrap with streaming context
        let candle = Candle::new(timestamp, open, high, low, close, volume);

        Some(MarketData::Candle {
            symbol,
            interval,
            data: candle,
            is_closed,
        })
    }

    /// Parses a Binance trade message into MarketData::Trade.
    /// Normalization: Converts Binance's "m" (is_buyer_maker) to explicit TradeSide.
    fn parse_trade(&self, msg: &str) -> Option<MarketData> {
        let symbol = extract_json_string(msg, r#""s":""#)?;
        let trade_id = extract_json_number(msg, r#""t":"#)? as u64;
        let price = extract_json_number(msg, r#""p":""#)?;
        let quantity = extract_json_number(msg, r#""q":""#)?;
        let timestamp = extract_json_number(msg, r#""T":"#)? as u64;
        
        // Binance uses "m" instead of explicit side - normalize to TradeSide
        let is_buyer_maker = msg.contains(r#""m":true"#);
        
        // Normalization: m=true means buyer was maker, so taker sold
        // This conversion ensures consistent TradeSide across all exchanges
        let side = if is_buyer_maker {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };

        // Keep is_buyer_maker for Binance-specific use cases
        let trade = Trade::new(
            timestamp,
            symbol,
            price,
            quantity,
            trade_id.to_string(),
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
                format!("{}@kline_{}", symbol.to_lowercase(), interval)
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
                format!("{}@kline_{}", symbol.to_lowercase(), interval)
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

/// Simple JSON number extraction (use serde_json in production).
fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let start = json.find(key)? + key.len();
    let rest = &json[start..];
    let end = rest.find([',', '"', '}'])?;
    rest[..end].parse::<f64>().ok()
}

/// Simple JSON string extraction (use serde_json in production).
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let start = json.find(key)? + key.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
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
        let stream = Stream::candles("BTCUSDT", "1m");
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
        let stream = Stream::candles("BTCUSDT", "5m");
        let msg = parser.format_unsubscribe(&stream);
        
        assert!(msg.contains("UNSUBSCRIBE"));
        assert!(msg.contains("btcusdt@kline_5m"));
    }

    #[test]
    fn test_parse_kline_message() {
        let parser = BinanceParser::new();
        
        let msg = r#"{"e":"kline","E":1638747660000,"s":"BTCUSDT","k":{"t":1638747660000,"T":1638747719999,"s":"BTCUSDT","i":"1m","o":"50000.00","c":"50100.00","h":"50200.00","l":"49900.00","v":"100.5","x":false}}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_some());
        
        match result.unwrap() {
            MarketData::Candle { symbol, interval, data, is_closed } => {
                assert_eq!(symbol, "BTCUSDT");
                assert_eq!(interval, "1m");
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
        
        let msg = r#"{"e":"kline","E":1638747660000,"s":"ETHUSDT","k":{"t":1638747660000,"T":1638747719999,"s":"ETHUSDT","i":"5m","o":"3000.00","c":"3050.00","h":"3100.00","l":"2950.00","v":"500.0","x":true}}"#;
        
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
        let msg = r#"{"e":"trade","E":1638747660000,"s":"BTCUSDT","t":12345,"p":"50000.00","q":"0.5","T":1638747660000,"m":false}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_some());
        
        match result.unwrap() {
            MarketData::Trade(trade) => {
                assert_eq!(trade.symbol, "BTCUSDT");
                assert_eq!(trade.price, 50000.00);
                assert_eq!(trade.quantity, 0.5);
                assert_eq!(trade.trade_id, "12345");
                assert_eq!(trade.side, TradeSide::Buy);
                assert_eq!(trade.is_buyer_maker, Some(false));
            }
            _ => panic!("Expected MarketData::Trade"),
        }
    }

    #[test]
    fn test_parse_trade_sell() {
        let parser = BinanceParser::new();
        
        // m:true = buyer is maker = SELL
        let msg = r#"{"e":"trade","E":1638747660000,"s":"ETHUSDT","t":67890,"p":"3000.00","q":"1.0","T":1638747660000,"m":true}"#;
        
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
