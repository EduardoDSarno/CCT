//! Market data module for exchange connections.
//! See docs/market/README.md for detailed documentation.

pub mod market_data;
pub mod message_parser;
pub mod websocket_client;
pub mod streams;
pub mod providers;

// Re-exports for convenience
pub use market_data::{
    MarketData,
    Trade,
    OrderBookUpdate,
    FundingRate,
    TradeSide,
    PriceLevel,
};
pub use message_parser::MessageParser;
pub use websocket_client::WebSocketClient;
pub use streams::Stream;

// Re-export provider convenience functions
pub use providers::binance::new_binance_client;
