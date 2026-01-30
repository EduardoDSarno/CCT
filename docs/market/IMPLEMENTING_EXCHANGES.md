# Implementing New Exchanges

This guide explains how to add support for a new cryptocurrency exchange.

## Overview

The `MessageParser` trait is the key abstraction that makes our WebSocket client exchange-agnostic. The generic `WebSocketClient<P: MessageParser>` handles all the common WebSocket logic (connection, reconnection, subscription tracking), while each exchange just implements these ~6 methods.

## Required Methods

| Method | Purpose |
|--------|---------|
| `endpoint()` | Primary WebSocket URL |
| `fallback_endpoint()` | Backup URL (optional) |
| `format_subscribe()` | Format subscription JSON |
| `format_unsubscribe()` | Format unsubscription JSON |
| `parse_message()` | Parse incoming JSON into `MarketData` |
| `name()` | Exchange name for logging |

## Step-by-Step Implementation

### 1. Create the Parser File

Create `src/market/providers/<exchange>.rs`:

```rust
use crate::indicators::candle::Candle;
use crate::market::market_data::{MarketData, Trade, TradeSide};
use crate::market::message_parser::MessageParser;
use crate::market::streams::Stream;
use crate::market::websocket_client::WebSocketClient;

pub const EXCHANGE_WSS_ENDPOINT: &str = "wss://api.exchange.com/ws";

#[derive(Debug, Clone)]
pub struct ExchangeParser;

impl ExchangeParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExchangeParser {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Implement MessageParser Trait

```rust
impl MessageParser for ExchangeParser {
    fn endpoint(&self) -> &str {
        EXCHANGE_WSS_ENDPOINT
    }

    fn fallback_endpoint(&self) -> Option<&str> {
        None // Or Some("wss://backup.exchange.com/ws")
    }

    fn name(&self) -> &'static str {
        "ExchangeName"
    }

    fn format_subscribe(&self, stream: &Stream) -> String {
        // Format according to exchange's API docs
        match stream {
            Stream::Candles { symbol, interval } => {
                format!(r#"{{"op":"subscribe","channel":"kline_{}","symbol":"{}"}}"#, 
                        interval, symbol)
            }
            Stream::Trades { symbol } => {
                format!(r#"{{"op":"subscribe","channel":"trade","symbol":"{}"}}"#, symbol)
            }
            _ => String::new(),
        }
    }

    fn format_unsubscribe(&self, stream: &Stream) -> String {
        // Similar to subscribe, but with unsubscribe operation
        match stream {
            Stream::Candles { symbol, interval } => {
                format!(r#"{{"op":"unsubscribe","channel":"kline_{}","symbol":"{}"}}"#, 
                        interval, symbol)
            }
            _ => String::new(),
        }
    }

    fn parse_message(&self, msg: &str) -> Option<MarketData> {
        // Detect message type and parse accordingly
        if msg.contains("\"channel\":\"kline\"") {
            return self.parse_kline(msg);
        }
        if msg.contains("\"channel\":\"trade\"") {
            return self.parse_trade(msg);
        }
        None
    }
}
```

### 3. Implement Parsing Helpers

```rust
impl ExchangeParser {
    fn parse_kline(&self, msg: &str) -> Option<MarketData> {
        // Parse exchange-specific JSON format
        // Extract: symbol, interval, timestamp, OHLCV, is_closed
        
        let candle = Candle::new(timestamp, open, high, low, close, volume);
        
        Some(MarketData::Candle {
            symbol,
            interval,
            data: candle,
            is_closed,
        })
    }

    fn parse_trade(&self, msg: &str) -> Option<MarketData> {
        // Parse trade data
        let trade = Trade::new(timestamp, symbol, price, quantity, trade_id, side);
        Some(MarketData::Trade(trade))
    }
}
```

### 4. Add Convenience Functions

```rust
pub type ExchangeClient = WebSocketClient<ExchangeParser>;

pub fn new_exchange_client() -> ExchangeClient {
    WebSocketClient::new(ExchangeParser::new())
}
```

### 5. Export from providers/mod.rs

```rust
pub mod binance;
pub mod exchange; // Add your new exchange

pub use binance::{BinanceClient, BinanceParser, new_binance_client};
pub use exchange::{ExchangeClient, ExchangeParser, new_exchange_client};
```

### 6. Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::timeframe::Timeframe;

    #[test]
    fn test_format_subscribe_candles() {
        let parser = ExchangeParser::new();
        let stream = Stream::candles("BTCUSDT", Timeframe::M1);
        let msg = parser.format_subscribe(&stream);
        
        // Assert expected format
        assert!(msg.contains("subscribe"));
        assert!(msg.contains("BTCUSDT"));
    }

    #[test]
    fn test_parse_kline_message() {
        let parser = ExchangeParser::new();
        
        // Use actual JSON from exchange's WebSocket
        let msg = r#"{"channel":"kline","data":{"t":1234567890000,...}}"#;
        
        let result = parser.parse_message(msg);
        assert!(result.is_some());
        
        if let Some(MarketData::Candle { symbol, .. }) = result {
            assert_eq!(symbol, "BTCUSDT");
        }
    }
}
```

## Handling Exchange-Specific Fields

If the exchange provides data that others don't:

1. Add the field as `Option<T>` to the appropriate struct
2. Set it in your parser, leave as `None` in other parsers
3. Document which exchange provides it

Example:
```rust
// In market_data.rs
pub struct Trade {
    // ... existing fields ...
    pub vwap: Option<f64>,  // Kraken-specific
}

// In your parser
let trade = Trade::new(...).with_vwap(vwap_value);
```

## Checklist

- [ ] Create parser file in `src/market/providers/`
- [ ] Implement `MessageParser` trait
- [ ] Add parsing helpers for each message type
- [ ] Add type alias and convenience function
- [ ] Export from `providers/mod.rs`
- [ ] Add unit tests for subscribe/unsubscribe formatting
- [ ] Add unit tests for message parsing
- [ ] Test with real WebSocket connection
