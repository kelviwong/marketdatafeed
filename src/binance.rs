use async_trait::async_trait;
use crate::candle::CandleStickBuilder;
use crate::common::ExchangeFeed;
use futures::stream::StreamExt;
use serde::Deserialize;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

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

pub struct Binance;

#[async_trait]
impl ExchangeFeed for Binance {
    const WS_URL: &str = "wss://stream.binance.com:9443/ws/";

    //wss://stream.binance.com:9443/ws/btcusdt@kline_1m
    async fn connect(
        symbols: &str,
        on_success: impl FnOnce(String) + Send,
    ) -> Result<String, String> {
        let connection_str = format!("{}{}", Self::WS_URL, symbols);

        // Connect to Binance WebSocket
        let (mut ws_stream, _) = connect_async(connection_str).await.expect("connected.");

        on_success("Binance connected.".to_string());

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

        Ok("Finished. Binance".to_string())
    }
}
// }

#[cfg(test)]
mod tests {
    use super::*;
}
