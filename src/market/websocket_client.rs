//! Generic WebSocket client for exchange connections.
//! See docs/market/README.md for architecture overview.

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
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
    ws_sender: Option<mpsc::Sender<String>>,
    // Design: Single channel for all market data types (Candle, Trade, OrderBook, Funding)
    // Consumer pattern-matches on MarketData enum to handle each type
    market_data_sender: Option<mpsc::Sender<MarketData>>,
}

impl<P: MessageParser> WebSocketClient<P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser: Arc::new(parser),
            subscriptions: Vec::new(),
            connected_at: None,
            is_connected: false,
            ws_sender: None,
            market_data_sender: None,
        }
    }

    /// Sets the channel for sending parsed market data to consumers.
    pub fn set_market_data_sender(&mut self, sender: mpsc::Sender<MarketData>) {
        self.market_data_sender = Some(sender);
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

        // Connect to the WebSocket endpoint
        let (ws_stream, _response) = connect_async(endpoint).await?;
        let (write, read) = ws_stream.split();

        // Channel for sending messages TO the WebSocket
        let (ws_tx, mut ws_rx) = mpsc::channel::<String>(100);
        self.ws_sender = Some(ws_tx);

        // Channel for market data FROM the WebSocket
        let (market_data_tx, market_data_rx) = if self.market_data_sender.is_none() {
            let (tx, rx) = mpsc::channel::<MarketData>(1000);
            self.market_data_sender = Some(tx.clone());
            (tx, Some(rx))
        } else {
            (self.market_data_sender.clone().unwrap(), None)
        };

        self.is_connected = true;
        self.connected_at = Some(Instant::now());

        let parser = Arc::clone(&self.parser);

        // Task: handle outgoing messages (write to WebSocket)
        let write = Arc::new(Mutex::new(write));
        let write_clone = Arc::clone(&write);
        
        tokio::spawn(async move {
            let mut write = write_clone.lock().await;
            while let Some(msg) = ws_rx.recv().await {
                if let Err(e) = write.send(Message::Text(msg.into())).await {
                    eprintln!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        // Task: handle incoming messages (read from WebSocket)
        tokio::spawn(async move {
            let mut read = read;
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        // Parse and send market data
                        if let Some(market_data) = parser.parse_message(&text) {
                            if let Err(e) = market_data_tx.send(market_data).await {
                                eprintln!("[{}] Failed to send market data: {}", parser.name(), e);
                                break;
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

        println!("[{}] Connected successfully!", self.parser.name());

        Ok(market_data_rx.unwrap_or_else(|| {
            let (_tx, rx) = mpsc::channel(1);
            rx
        }))
    }

    pub async fn subscribe(&mut self, stream: Stream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_connected {
            return Err("Not connected".into());
        }

        // each client will have its own subscribe format
        let msg = self.parser.format_subscribe(&stream);
        
        if let Some(sender) = &self.ws_sender {
            sender.send(msg).await?;
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
            sender.send(msg).await?;
            self.subscriptions.retain(|s| s != stream);
            println!("[{}] Unsubscribed from {:?}", self.parser.name(), stream);
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        self.ws_sender = None;
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
}
