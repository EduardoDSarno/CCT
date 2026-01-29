# Market Data Types

Exchange-agnostic data structures for market data received from WebSocket streams.

## Architecture Decisions

### Why Candle is wrapped but Trade/OrderBook/Funding are not

- **Candle** is a *calculation primitive* used by indicators (`is_doji()`, `atr()`, `ema()`).
  It doesn't inherently need to know its symbol/interval - that's streaming context.
  The `Candle` struct in `indicators/candle.rs` stays simple for clean indicator code.

- **Trade/OrderBook/Funding** are *discrete events* - each happens once, for one symbol.
  They naturally contain their symbol because you can't process them without knowing it.

### Exchange Compatibility (Binance, Bybit, Hyperliquid)

Fields use `Option<T>` when only some exchanges provide them:

| Field | Type | Exchange Support |
|-------|------|------------------|
| `Trade::is_buyer_maker` | `Option<bool>` | Binance only |
| `PriceLevel::num_orders` | `Option<u32>` | Hyperliquid only |
| `OrderBookUpdate::sequence` | `Option<u64>` | Varies by exchange |
| `FundingRate::next_funding_time` | `Option<u64>` | Varies by exchange |
| `FundingRate::mark_price` | `Option<f64>` | Varies by exchange |

This allows adding new exchanges without breaking existing code - just set exchange-specific fields to `None` when not available.

## Data Types

### MarketData Enum

The unified enum that carries all market data through a single channel:

```rust
pub enum MarketData {
    Candle { symbol, interval, data: Candle, is_closed },
    Trade(Trade),
    OrderBook(OrderBookUpdate),
    Funding(FundingRate),
}
```

### TradeSide

```rust
pub enum TradeSide {
    Buy,
    Sell,
}
```

Note: Binance provides `is_buyer_maker` boolean instead of explicit side. The parser converts: `is_buyer_maker = true` means the trade was a SELL (buyer was taker, seller was maker).

### Trade

A single trade event from the exchange:

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `u64` | Unix timestamp in milliseconds |
| `symbol` | `String` | Trading pair (e.g., "BTCUSDT") |
| `price` | `f64` | Execution price |
| `quantity` | `f64` | Trade quantity (base asset) |
| `trade_id` | `String` | Unique trade identifier |
| `side` | `TradeSide` | Buy or Sell |
| `is_buyer_maker` | `Option<bool>` | Binance-specific |

### PriceLevel

A single row in the order book:

| Field | Type | Description |
|-------|------|-------------|
| `price` | `f64` | Price at this level |
| `quantity` | `f64` | Total quantity available |
| `num_orders` | `Option<u32>` | Number of orders (Hyperliquid only) |

### OrderBookUpdate

Order book snapshot or delta update:

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `u64` | Unix timestamp in milliseconds |
| `symbol` | `String` | Trading pair |
| `bids` | `Vec<PriceLevel>` | Buy orders (price descending) |
| `asks` | `Vec<PriceLevel>` | Sell orders (price ascending) |
| `is_snapshot` | `bool` | True = full snapshot, False = delta |
| `sequence` | `Option<u64>` | Sequence number for ordering |

### FundingRate

Funding rate event for perpetual futures:

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `u64` | Unix timestamp in milliseconds |
| `symbol` | `String` | Trading pair |
| `rate` | `f64` | Funding rate (positive = longs pay shorts) |
| `next_funding_time` | `Option<u64>` | Next settlement timestamp |
| `mark_price` | `Option<f64>` | Current mark price |

## Usage Example

```rust
match market_data {
    MarketData::Candle { symbol, data, is_closed, .. } => {
        if is_closed {
            candles.push(data);  // Just the Candle, ready for indicators
        }
    }
    MarketData::Trade(trade) => { 
        println!("{}: {} @ {}", trade.symbol, trade.quantity, trade.price);
    }
    MarketData::OrderBook(book) => {
        if book.is_snapshot {
            // Replace entire order book
        } else {
            // Apply delta update
        }
    }
    MarketData::Funding(funding) => {
        println!("{}: Funding rate {}", funding.symbol, funding.rate);
    }
}
```

## Warning: is_closed Flag

If `is_closed` is `false`, the candle is still updating. Do not store or use for indicator calculations until `is_closed` is `true`.
