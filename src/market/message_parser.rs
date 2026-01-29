//! MessageParser trait for exchange-specific message handling.
//! See docs/market/IMPLEMENTING_EXCHANGES.md for how to implement this trait.

use crate::market::market_data::MarketData;
use crate::market::streams::Stream;

// This trait is the key abstraction that makes WebSocketClient exchange-agnostic.
// Each exchange implements the follwing methods, WebSocketClient handles everything else.
// Adding a new exchange = implement this trait, no changes to WebSocketClient.
// =============================================================================

/// Trait for exchange-specific message parsing and formatting.
/// Implement this for each exchange (Binance, Bybit, Hyperliquid, etc.)
pub trait MessageParser: Send + Sync + 'static {
    /// Returns the primary WebSocket endpoint URL.
    fn endpoint(&self) -> &str;

    /// Returns a fallback endpoint URL (if primary fails).
    fn fallback_endpoint(&self) -> Option<&str> {
        None
    }

    // Each exchange has different JSON formats for subscribe/unsubscribe
    fn format_subscribe(&self, stream: &Stream) -> String;
    fn format_unsubscribe(&self, stream: &Stream) -> String;

    /// Parses exchange-specific JSON into normalized MarketData.
    /// This is where exchange differences are absorbed - output is always MarketData.
    /// Returns Some(MarketData) for valid data, None for control messages.
    fn parse_message(&self, msg: &str) -> Option<MarketData>;

    fn name(&self) -> &'static str;

    /// Most exchanges have 24h connection limit. Default: 23 hours (safe margin).
    fn max_connection_duration_secs(&self) -> u64 {
        23 * 60 * 60
    }
}
