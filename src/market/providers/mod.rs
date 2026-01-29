//! Exchange provider implementations.

pub mod binance;

// Re-export for convenience
pub use binance::{BinanceClient, BinanceParser, new_binance_client};