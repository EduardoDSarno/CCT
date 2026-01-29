# Binance Provider

Binance-specific implementation details and WebSocket message formats.

## Endpoints

| Type | URL |
|------|-----|
| Primary | `wss://stream.binance.com:443/ws` |
| Fallback | `wss://stream.binance.com:9443/ws` |

## Supported Streams

| Stream Type | Format | Example |
|-------------|--------|---------|
| Kline/Candles | `<symbol>@kline_<interval>` | `btcusdt@kline_1m` |
| Trades | `<symbol>@trade` | `btcusdt@trade` |
| Order Book | `<symbol>@depth<levels>` | `btcusdt@depth20` |
| Mark Price/Funding | `<symbol>@markPrice` | `btcusdt@markPrice` |

## Message Formats

### Kline Message

```json
{
  "e": "kline",
  "E": 1638747660000,
  "s": "BTCUSDT",
  "k": {
    "t": 1638747660000,
    "T": 1638747719999,
    "s": "BTCUSDT",
    "i": "1m",
    "o": "50000.00",
    "c": "50100.00",
    "h": "50200.00",
    "l": "49900.00",
    "v": "100.5",
    "x": false
  }
}
```

| Field | Description |
|-------|-------------|
| `e` | Event type ("kline") |
| `E` | Event time (ms) |
| `s` | Symbol |
| `k.t` | Kline start time (ms) |
| `k.T` | Kline close time (ms) |
| `k.i` | Interval |
| `k.o` | Open price |
| `k.c` | Close price |
| `k.h` | High price |
| `k.l` | Low price |
| `k.v` | Volume |
| `k.x` | Is kline closed? |

### Trade Message

```json
{
  "e": "trade",
  "E": 1638747660000,
  "s": "BTCUSDT",
  "t": 12345,
  "p": "50000.00",
  "q": "0.5",
  "T": 1638747660000,
  "m": true
}
```

| Field | Description |
|-------|-------------|
| `e` | Event type ("trade") |
| `E` | Event time (ms) |
| `s` | Symbol |
| `t` | Trade ID |
| `p` | Price |
| `q` | Quantity |
| `T` | Trade time (ms) |
| `m` | Is buyer the maker? |

### Understanding `is_buyer_maker` (m)

Binance uses the `m` field instead of an explicit buy/sell side:

- `m = true` → Buyer was the **maker** → Trade was a **SELL** (taker sold)
- `m = false` → Buyer was the **taker** → Trade was a **BUY** (taker bought)

Our parser converts this to `TradeSide::Buy` or `TradeSide::Sell` for consistency.

## Subscribe/Unsubscribe Format

```json
// Subscribe
{"method":"SUBSCRIBE","params":["btcusdt@kline_1m"],"id":1}

// Unsubscribe
{"method":"UNSUBSCRIBE","params":["btcusdt@kline_1m"],"id":1}
```

## Connection Limits

- Max connection duration: 24 hours (we reconnect at 23 hours)
- Max streams per connection: 1024
- Max incoming messages: 5 per second

## Currently Implemented

- [x] Kline/Candle parsing
- [x] Trade parsing
- [ ] Order book parsing
- [ ] Mark price/funding parsing

## Usage

```rust
use crate::market::{new_binance_client, Stream, MarketData};

let mut client = new_binance_client();
let rx = client.connect().await?;

client.subscribe(Stream::candles("BTCUSDT", "1m")).await?;
client.subscribe(Stream::trades("BTCUSDT")).await?;

while let Some(data) = rx.recv().await {
    match data {
        MarketData::Candle { symbol, data, is_closed, .. } => {
            if is_closed {
                println!("{}: Closed at {}", symbol, data.get_close());
            }
        }
        MarketData::Trade(trade) => {
            let side = if trade.side == TradeSide::Buy { "BUY" } else { "SELL" };
            println!("{}: {} {} @ {}", trade.symbol, side, trade.quantity, trade.price);
        }
        _ => {}
    }
}
```
