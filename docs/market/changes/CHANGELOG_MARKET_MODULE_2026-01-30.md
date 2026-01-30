# Market Module Changes — 2026-01-30

This document details the changes applied to the market module and indicators. Each entry includes:
- **Location** (path you can open directly)
- **Before** (what existed)
- **After** (what changed)
- **Why** (problem addressed)
- **Future benefit** (how this improves reliability/maintainability)

---

## 1) Binance parsing rewritten with `serde_json`
**Location:** `src/market/providers/binance.rs`

**Before**
- Kline/trade parsing used ad‑hoc string scanning (`extract_json_number/string`).
- Looked for stringified numbers (e.g., `"t":"..."`) and the kline object as `"k":"`.
- Any reordering/whitespace or actual numeric fields caused parsing to fail silently (returning `None`).

**After**
- Added `serde`/`serde_json` parsing with typed structs:
  - `BinanceKlineEvent`, `BinanceKline`
  - `BinanceTradeEvent` with `#[serde(rename = "T")] trade_time`
- Added robust float deserializer that accepts string or numeric values.
- Timeframe parsing now converts Binance interval strings to `Timeframe`.

**Why**
- The earlier parser **did not match actual Binance payloads** and was fragile to even minor changes.
- Kline/trade data was being dropped, causing silent failures and no alerts.

**Future benefit**
- Parsing now survives key reordering and handles numeric/string fields consistently.
- Safer extension to new message types (funding, order book) using typed structs.

---

## 2) Stream interval is now `Timeframe` (enum), not String
**Location:**
- `src/market/streams.rs`
- `src/market/market_data.rs`
- `src/indicators/timeframe.rs`
- Docs updated in: `docs/market/README.md`, `docs/market/BINANCE.md`, `docs/market/IMPLEMENTING_EXCHANGES.md`, `docs/market/MARKET_DATA.md`

**Before**
- `Stream::Candles { interval: String }` and `MarketData::Candle { interval: String }`.
- Any value ("1M", "1min", "bad") would compile and be sent to exchanges.

**After**
- `Stream::Candles` and `MarketData::Candle` now use `Timeframe`.
- `Timeframe::from_str()` added for parsing exchange strings.

**Why**
- String intervals were error‑prone and inconsistent with indicator `Timeframe` usage.

**Future benefit**
- Compile‑time safety and consistent timeframe handling across market feeds and indicators.
- Easier to add validation/conversion for new exchanges.

---

## 3) WebSocketClient API cleanup (removed external sender)
**Location:** `src/market/websocket_client.rs`

**Before**
- `set_market_data_sender()` allowed plugging a custom channel.
- `connect()` returned a *dummy receiver* if a sender was already set, causing silent hangs.

**After**
- Removed `set_market_data_sender()` entirely.
- `connect()` always returns the real receiver.

**Why**
- The previous API was a foot‑gun: callers could wait forever on a receiver that never received data.

**Future benefit**
- Clear, predictable API usage.
- Consumers always get the real stream.

---

## 4) WebSocket lifecycle fixes (disconnect/reconnect + fallback)
**Location:** `src/market/websocket_client.rs`

**Before**
- `disconnect()` only cleared flags; did not close sockets or stop tasks.
- `reconnect()` spawned new tasks without stopping old ones.
- Fallback endpoints existed in the parser but were not used.

**After**
- Tracks read/write task handles and aborts them on disconnect.
- Sends close frames on disconnect.
- Uses fallback endpoint when primary connection fails.
- Added `reconnect_if_needed()` helper.

**Why**
- Old connections/tasks could leak and duplicate data.
- Failover behavior was missing.

**Future benefit**
- Reliable reconnection without duplicate subscriptions or memory/task leaks.
- Increased resiliency to endpoint outages.

---

## 5) Backpressure: don’t block read loop
**Location:** `src/market/websocket_client.rs`

**Before**
- `market_data_tx.send().await` could block the read loop if consumers were slow.

**After**
- Uses `try_send()` and drops messages when channel is full, logging the drop.

**Why**
- Avoids WebSocket timeouts due to blocked reads.

**Future benefit**
- Maintains connection health and responsiveness under load.

---

## 6) Stream subscription de‑duplication + order book depth type
**Location:** `src/market/websocket_client.rs`, `src/market/streams.rs`

**Before**
- Subscriptions were stored in a Vec without de‑duplication.
- Order book depth was a free‑form String.

**After**
- `subscribe()` checks for existing subscriptions to avoid duplicates.
- Order book depth is now `u16` and validated (debug assert > 0).

**Why**
- Duplicate subscriptions would create duplicate data and alerts.
- String depth allowed invalid requests.

**Future benefit**
- Cleaner subscription management.
- Safer, more predictable order book configuration.

---

## 7) Indicators return Option<f64> (insufficient data safe)
**Location:**
- `src/indicators/momentum.rs`
- `src/indicators/moving_averages.rs`
- `src/indicators/volatility.rs`

**Before**
- Functions returned `0.0` when there wasn’t enough data.
- `0.0` is a valid value, which could produce false alerts.

**After**
- `rsi`, `sma`, `ema`, `atr` now return `Option<f64>`.
- Tests updated to expect `None` when insufficient data.

**Why**
- Distinguishes “not enough data” from a real numeric output.

**Future benefit**
- Prevents false alert triggers during warm‑up.
- Cleaner strategy logic (explicit `None` handling).

---

## 8) RSI standardization (Wilder smoothing)
**Location:** `src/indicators/momentum.rs`

**Before**
- `rsi()` used a simple average of the last period, while `rsi_series()` used Wilder smoothing.

**After**
- `rsi()` now returns the last value of `rsi_series()`.

**Why**
- Mismatched RSI logic produced inconsistent signals.

**Future benefit**
- Consistent RSI across single‑value and series use cases.

---

## 9) Candle invariants + documentation clarity
**Location:**
- `src/indicators/candle.rs`
- `src/indicators/candle_patterns.rs`

**Before**
- No invariant checks for OHLC ordering.
- `CandlePatterns` stored timeframe but didn’t mention that it’s metadata.

**After**
- Added debug assertions for OHLC ordering.
- Clarified that timeframe is metadata for callers.

**Why**
- Invalid OHLC values can cause negative ranges and invalid indicators.

**Future benefit**
- Easier debugging and safer upstream data handling.

---

## 10) Docs & test improvements
**Location:**
- `tests/market_doc_samples.rs`
- `docs/market/*`

**Before**
- Tests were only for Binance parser and indicators.

**After**
- Added JSON sample tests for Bybit and Hyperliquid using official payload examples.
- Updated docs to reflect Timeframe usage.

**Why**
- Ensures future exchanges are aligned with known payload structures.

**Future benefit**
- Faster onboarding for new exchange parsers.

---

## 11) Tooling fixes
**Location:** `Cargo.toml`

**Before**
- `clippy` listed as a normal dependency, which causes cargo warnings.

**After**
- Moved `clippy` to `[dev-dependencies]`.

**Why**
- `clippy` is a dev tool and not a library dependency.

**Future benefit**
- Cleaner builds and less dependency noise.

