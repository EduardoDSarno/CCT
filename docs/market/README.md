# Market Module

The market module provides infrastructure for connecting to cryptocurrency exchanges and receiving real-time market data.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        market module                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐    ┌──────────────────┐                   │
│  │  market_data    │    │  message_parser  │                   │
│  │  ─────────────  │    │  ──────────────  │                   │
│  │  MarketData     │◄───│  MessageParser   │ (trait)           │
│  │  Trade          │    │                  │                   │
│  │  OrderBookUpdate│    └──────────────────┘                   │
│  │  FundingRate    │             ▲                             │
│  │  TradeSide      │             │ implements                  │
│  │  PriceLevel     │             │                             │
│  └─────────────────┘    ┌────────┴─────────┐                   │
│                         │    providers     │                   │
│  ┌─────────────────┐    │  ────────────    │                   │
│  │ websocket_client│    │  BinanceParser   │                   │
│  │ ────────────────│    │  (future: Bybit) │                   │
│  │ WebSocketClient │    │  (future: HL)    │                   │
│  │                 │    └──────────────────┘                   │
│  └─────────────────┘                                           │
│                                                                 │
│  ┌─────────────────┐                                           │
│  │    streams      │                                           │
│  │  ─────────────  │                                           │
│  │  Stream enum    │ (Candles, Trades, OrderBook, etc.)        │
│  └─────────────────┘                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow

```
Exchange WebSocket
       │
       ▼
┌──────────────────┐
│ WebSocketClient  │  (handles connection, reconnection, channels)
│                  │
│  ┌────────────┐  │
│  │ Parser (P) │  │  (exchange-specific: parse_message, format_subscribe)
│  └────────────┘  │
└──────────────────┘
       │
       ▼ mpsc::Sender<MarketData>
┌──────────────────┐
│ Your Consumer    │  (receives normalized MarketData)
└──────────────────┘
```

## Modules

| Module | Description |
|--------|-------------|
| `market_data` | Normalized data types for all exchanges |
| `message_parser` | Trait for exchange-specific message parsing |
| `websocket_client` | Generic WebSocket client |
| `streams` | Stream subscription types |
| `providers` | Exchange implementations (Binance, etc.) |

## Usage Example

```rust
use crate::indicators::timeframe::Timeframe;
use crate::market::{MarketData, Stream, new_binance_client};

let mut client = new_binance_client();
let rx = client.connect().await?;

client.subscribe(Stream::candles("BTCUSDT", Timeframe::M1)).await?;
client.subscribe(Stream::trades("BTCUSDT")).await?;

while let Some(data) = rx.recv().await {
    match data {
        MarketData::Candle { symbol, data, is_closed, .. } => {
            if is_closed {
                // Process closed candle
            }
        }
        MarketData::Trade(trade) => {
            // Process trade
        }
        _ => {}
    }
}
```

## Related Documentation

- [Market Data Types](./MARKET_DATA.md) - Data structures and design decisions
- [Implementing Exchanges](./IMPLEMENTING_EXCHANGES.md) - How to add new exchange support
- [Binance Provider](./BINANCE.md) - Binance-specific details
