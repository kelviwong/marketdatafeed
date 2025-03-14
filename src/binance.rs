use std::thread;

use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::Deserialize;
use tokio::task;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::runtime::Runtime;
use tokio::task::spawn_blocking;

// mod ExchangeFeeder {
#[derive(Debug, Deserialize)]
struct KlineData {
    #[serde(rename = "t")]
    open_time: u64,
    #[serde(rename = "o")]
    open: String,
    #[serde(rename = "h")]
    high: String,
    #[serde(rename = "l")]
    low: String,
    #[serde(rename = "c")]
    close: String,
    #[serde(rename = "v")]
    volume: String,
}

#[derive(Debug, Deserialize)]
struct WebSocketMessage {
    #[serde(rename = "k")]
    kline: KlineData,
}

#[async_trait]
pub trait ExchangeFeed {
    const WS_URL: &'static str;
    async fn connect(symbols: &str);
    // async fn start_connection_spin(symbols: &str);
}

pub struct Binance;

#[async_trait]
impl ExchangeFeed for Binance {
    const WS_URL: &str = "wss://stream.binance.com:9443/ws/";

    //wss://stream.binance.com:9443/ws/btcusdt@kline_1m
    async fn connect(symbols: &str) {
        
        let connection_str = format!("{}{}", Self::WS_URL, symbols);

        // Connect to Binance WebSocket
        let (mut ws_stream, _) = connect_async(connection_str).await.expect("connected.");
        // let (mut write, mut read) = ws_stream.split();

        // let task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_stream.next().await {
                match msg {
                    Message::Text(text) => {
                        // Deserialize the received JSON message into a WebSocketMessage
                        let _ws_message: WebSocketMessage =
                            serde_json::from_str(&text).expect("Failed to deserialize message");
                        println!("Recieved: {:?}", _ws_message);
                    }
                    _ => (),
                }
            }
        // });

        // Ok(task)
    }

    // async fn start_connection_spin(symbols: &str) {
    //     println!("starting {:?}", symbols);
    //     // Initialize the runtime manually
    //     let rt = Runtime::new().unwrap();
    
    //     // Run the async function in the Tokio runtime
    //     rt.block_on(async {
    //         match Self::connect(symbols).await {
    //             Ok(_) => println!("Connection successful."),
    //             Err(e) => println!("Error connecting: {}", e),
    //         }
    //     });
    // }
}
// }

#[cfg(test)]
mod tests {
    use super::*;
}
