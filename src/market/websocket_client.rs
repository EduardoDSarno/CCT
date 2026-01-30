//! Generic WebSocket client for exchange connections.
//! See docs/market/README.md for architecture overview.

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::market::market_data::MarketData;
use crate::market::message_parser::MessageParser;
use crate::market::streams::Stream;

// Design: WebSocketClient<P: MessageParser> is generic over the parser type.
// This allows reusing all WebSocket logic (connection, reconnection, channels,
// subscription tracking) while each exchange only implements MessageParser.
// Adding a new exchange = implement ~6 methods in MessageParser, done.

/// Generic WebSocket client that works with any exchange.
/// Exchange-specific logic is provided by the MessageParser implementation.
pub struct WebSocketClient<P: MessageParser> {
    parser: Arc<P>,
    subscriptions: Vec<Stream>,
    connected_at: Option<Instant>,  // for 24h reconnection limit tracking
    is_connected: bool,
    ws_sender: Option<mpsc::Sender<Message>>,
    read_handle: Option<JoinHandle<()>>, // handle for tasks
    write_handle: Option<JoinHandle<()>>, // handle for tasks
}
// This WebSocket client works with any parser type, as long as that parser knows how to parse messages
impl<P: MessageParser> WebSocketClient<P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser: Arc::new(parser),
            subscriptions: Vec::new(),
            connected_at: None,
            is_connected: false,
            ws_sender: None,
            read_handle: None,
            write_handle: None,
        }
    }

    pub fn name(&self) -> &'static str {
        self.parser.name()
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn subscriptions(&self) -> &[Stream] {
        &self.subscriptions
    }

    /// Checks if connection needs refresh (approaching 24h limit).
    pub fn needs_reconnect(&self) -> bool {
        if let Some(connected_at) = self.connected_at {
            let max_duration = Duration::from_secs(self.parser.max_connection_duration_secs());
            connected_at.elapsed() > max_duration
        } else {
            false
        }
    }

    /// Connects to the WebSocket endpoint.
    /// Spawns background tasks for message handling.
    /// Returns a receiver channel for market data.
    pub async fn connect(&mut self) -> Result<mpsc::Receiver<MarketData>, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = self.parser.endpoint();
        
        println!("[{}] Connecting to {}...", self.parser.name(), endpoint);

        // Connect to the WebSocket endpoint (fallback if primary fails)
        let (ws_stream, _response) = match connect_async(endpoint).await {
            Ok(result) => result,
            Err(primary_err) => {
                if let Some(fallback) = self.parser.fallback_endpoint() {
                    eprintln!(
                        "[{}] Primary connection failed ({}). Trying fallback {}...",
                        self.parser.name(),
                        primary_err,
                        fallback
                    );
                    connect_async(fallback).await?
                } else {
                    return Err(primary_err.into());
                }
            }
        };
        let (write, read) = ws_stream.split();

        // Channel for sending messages TO the WebSocket
        let (ws_tx, mut ws_rx) = mpsc::channel::<Message>(100);
        self.ws_sender = Some(ws_tx);

        // Channel for market data FROM the WebSocket
        let (market_data_tx, market_data_rx) = mpsc::channel::<MarketData>(1000);

        self.is_connected = true;
        self.connected_at = Some(Instant::now());

        let parser = Arc::clone(&self.parser);

        // Task: handle outgoing messages (write to WebSocket)
        let write = Arc::new(Mutex::new(write));
        let write_clone = Arc::clone(&write);
        
        // This spawns a background async task whose only job is to forward messages from a channel to a WebSocket writer.
        let write_handle = tokio::spawn(async move {
            let mut write = write_clone.lock().await;
            while let Some(msg) = ws_rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    eprintln!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        // Task: handle incoming messages (read from WebSocket)
        let read_handle = tokio::spawn(async move {
            let mut read = read;
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        // Parse and send market data
                        if let Some(market_data) = parser.parse_message(&text) {
                            match market_data_tx.try_send(market_data) {
                                Ok(_) => {}
                                Err(TrySendError::Full(_)) => {
                                    eprintln!(
                                        "[{}] Market data channel full; dropping message",
                                        parser.name()
                                    );
                                }
                                Err(TrySendError::Closed(_)) => {
                                    eprintln!(
                                        "[{}] Market data channel closed; stopping read loop",
                                        parser.name()
                                    );
                                    break;
                                }
                            }
                        }
                        // Control messages (subscription confirmations, etc.) are ignored
                    }
                    Ok(Message::Ping(_data)) => {
                        println!("[{}] Ping received", parser.name());
                        // Pong handled automatically by tungstenite
                    }
                    Ok(Message::Pong(_)) => {
                        // Connection alive
                    }
                    Ok(Message::Close(frame)) => {
                        println!("[{}] Connection closed: {:?}", parser.name(), frame);
                        break;
                    }
                    Ok(Message::Binary(_)) => {
                        // Binary messages not used for market data
                    }
                    Err(e) => {
                        eprintln!("[{}] WebSocket error: {}", parser.name(), e);
                        break;
                    }
                    _ => {}
                }
            }
            println!("[{}] Read task ended", parser.name());
        });

        self.write_handle = Some(write_handle);
        self.read_handle = Some(read_handle);

        println!("[{}] Connected successfully!", self.parser.name());

        Ok(market_data_rx)
    }

    pub async fn subscribe(&mut self, stream: Stream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_connected {
            return Err("Not connected".into());
        }

        if self.subscriptions.contains(&stream) {
            return Ok(());
        }

        // each client will have its own subscribe format
        let msg = self.parser.format_subscribe(&stream);
        
        if let Some(sender) = &self.ws_sender {
            sender.send(Message::Text(msg.into())).await?; // into to build Utf8Bytes
            self.subscriptions.push(stream.clone());
            println!("[{}] Subscribed to {:?}", self.parser.name(), stream);
        }

        Ok(())
    }

    pub async fn unsubscribe(&mut self, stream: &Stream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_connected {
            return Err("Not connected".into());
        }

        // each client will have its own unsubscribe format
        let msg = self.parser.format_unsubscribe(stream);
        
        if let Some(sender) = &self.ws_sender {
            sender.send(Message::Text(msg.into())).await?;
            self.subscriptions.retain(|s| s != stream);
            println!("[{}] Unsubscribed from {:?}", self.parser.name(), stream);
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(sender) = &self.ws_sender {
            let _ = sender.send(Message::Close(None)).await;
        }
        self.ws_sender = None;
        if let Some(handle) = self.read_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.write_handle.take() {
            handle.abort();
        }
        self.is_connected = false;
        self.connected_at = None;
        println!("[{}] Disconnected", self.parser.name());
    }

    /// Reconnects and restores all subscriptions.
    pub async fn reconnect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[{}] Reconnecting...", self.parser.name());
        
        let subs = self.subscriptions.clone();
        
        self.disconnect().await;
        self.subscriptions.clear();
        self.connect().await?;
        
        // Restore subscriptions
        for stream in subs {
            self.subscribe(stream).await?;
        }

        println!("[{}] Reconnected and restored {} subscriptions", 
                 self.parser.name(), self.subscriptions.len());
        
        Ok(())
    }

    /// Reconnects if the connection is nearing the exchange's maximum duration.
    pub async fn reconnect_if_needed(&mut self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if self.needs_reconnect() {
            self.reconnect().await?;
            return Ok(true);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestParser;

    impl MessageParser for TestParser {
        fn endpoint(&self) -> &str {
            "wss://example.invalid/ws"
        }

        fn format_subscribe(&self, _stream: &Stream) -> String {
            "{\"op\":\"subscribe\"}".to_string()
        }

        fn format_unsubscribe(&self, _stream: &Stream) -> String {
            "{\"op\":\"unsubscribe\"}".to_string()
        }

        fn parse_message(&self, _msg: &str) -> Option<MarketData> {
            None
        }

        fn name(&self) -> &'static str {
            "Test"
        }

        fn max_connection_duration_secs(&self) -> u64 {
            1
        }
    }

    #[tokio::test]
    async fn test_subscribe_dedup() {
        let mut client = WebSocketClient::new(TestParser);
        let (tx, _rx) = mpsc::channel::<Message>(10);
        client.ws_sender = Some(tx);
        client.is_connected = true;

        let stream = Stream::candles("BTCUSDT", crate::indicators::timeframe::Timeframe::M1);
        client.subscribe(stream.clone()).await.unwrap();
        client.subscribe(stream).await.unwrap();

        assert_eq!(client.subscriptions.len(), 1);
    }

    #[tokio::test]
    async fn test_disconnect_resets_state() {
        let mut client = WebSocketClient::new(TestParser);
        let (tx, _rx) = mpsc::channel::<Message>(10);
        client.ws_sender = Some(tx);
        client.is_connected = true;
        client.connected_at = Some(Instant::now());

        client.disconnect().await;

        assert!(!client.is_connected);
        assert!(client.ws_sender.is_none());
        assert!(client.connected_at.is_none());
    }

    #[test]
    fn test_needs_reconnect_true() {
        let mut client = WebSocketClient::new(TestParser);
        client.connected_at = Some(Instant::now() - Duration::from_secs(2));
        assert!(client.needs_reconnect());
    }
}
