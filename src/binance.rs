use crate::candle::CandleStickBuilder;
use crate::common::ExchangeFeed;
use crate::config::Config;
use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::Deserialize;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::error::Error;

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

pub struct Binance{
    pub base_url: String,
    pub symbol: String,
    pub enable: bool,
}

impl Binance {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::from_file(config_path)?;

        Ok(Binance {
            base_url: config.binance.base_url,
            symbol: config.binance.symbol,
            enable: config.binance.enable,
        })
    }
}

#[async_trait]
impl ExchangeFeed for Binance {
    async fn connect(
        &self,
        on_success: impl FnOnce(String) + Send,
    ) -> Result<String, String> {

        if (!self.enable) {
            return Ok("Disabled. Binance.".to_string());
        }
        
        let connection_str = format!("{}{}", self.base_url, self.symbol);
        println!("connecting... {:?}", connection_str);

        // Connect to Binance WebSocket
        let mut ws_stream = None;
        match connect_async(connection_str).await {
            Ok((mut stream, _)) => {
                // Connection was successful
                ws_stream = Some(stream);
                println!("Connected to WebSocket server");
            }
            Err(e) => {
                // Handle the error if the connection fails
                return Err(format!("Error Connecting: {:?}", e));
            }
        }

        on_success("Binance connected.".to_string());

        let mut ws_stream = ws_stream.unwrap(); 

        let mut candle_stick_data = CandleStickBuilder::new().build();
        candle_stick_data.source = "Binance".to_string();

        while let Some(result) = ws_stream.next().await {
            match result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            match serde_json::from_str::<WebSocketMessage>(&text) {
                                Ok(ws_message) => {
                                    // Update candle stick data if deserialization is successful
                                    candle_stick_data.ts = ws_message.kline.open_time;
                                    candle_stick_data.c =
                                        ws_message.kline.close.parse::<f64>().unwrap_or(0.0);
                                    candle_stick_data.o =
                                        ws_message.kline.open.parse::<f64>().unwrap_or(0.0);
                                    candle_stick_data.h =
                                        ws_message.kline.high.parse::<f64>().unwrap_or(0.0);
                                    candle_stick_data.l =
                                        ws_message.kline.low.parse::<f64>().unwrap_or(0.0);
                                    candle_stick_data.v =
                                        ws_message.kline.volume.parse::<f64>().unwrap_or(0.0);

                                    println!("CandleStick data: {:?}", candle_stick_data);
                                }
                                Err(e) => {
                                    println!("Failed to deserialize message: {:?}", e);
                                    println!("{:?}", text);
                                }
                            }
                        }
                        _ => (),
                    }
                }
                Err(e) => {
                    // Return error message in case of connection failure
                    return Err(format!("Error receiving message: {:?}", e));
                }
            }
        }

        Ok("Finished. Binance".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
