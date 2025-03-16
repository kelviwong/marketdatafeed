use async_trait::async_trait;
use futures::StreamExt;
use tokio::net::TcpStream;
use std::error::Error;
use tokio_tungstenite::{connect_async, tungstenite::{Message, Utf8Bytes}, WebSocketStream};

use crate::candle::{candle_stick, CandleStickBuilder};

#[async_trait]
pub trait ExchangeFeed: Service {
    async fn connect(&self, on_success: impl FnOnce(String) + Send) -> Result<String, String> {
        let (symbol, channel) = Self::get_sub_channel(self.symbol());

        // let connection_str = format!("{}{}", self.base_url(), self.symbol());
        let connection_str = Self::get_connection_str(self.base(), symbol);
        println!("connecting... {:?}", connection_str);

        // Connect to WebSocket
        let mut ws_stream = None;
        match connect_async(connection_str).await {
            Ok((stream, _)) => {
                // Connection was successful
                ws_stream = Some(stream);
                println!("Connected to WebSocket server");
            }
            Err(e) => {
                // Handle the error if the connection fails
                return Err(format!("Error Connecting: {:?}", e));
            }
        }

        on_success(format!("{} connected.", self.name()));

        let mut ws_stream = ws_stream.unwrap();

        Self::send_subscribe(channel, symbol, &mut ws_stream).await;

        let mut candle_stick_data = CandleStickBuilder::new().build();
        candle_stick_data.source = self.name().to_string();

        while let Some(result) = ws_stream.next().await {
            match result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            Self::parse_text(text, &mut candle_stick_data);
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

        Ok(format!("Finished. {}", self.name()))
    }

    fn parse_text(text: Utf8Bytes, candle_stick_data: &mut candle_stick);

    fn get_sub_channel(symbol: &str) -> (&str, &str) {
        ("", "")
    }

    fn get_connection_str<'a>(base: &'a str, symbol: &'a str) -> String {
        "".to_string()
    }

    async fn send_subscribe(
        channel: &str,
        symbol: &str,
        ws_stream: &mut WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    ) {
    }
}

pub trait Service {
    fn name(&self) -> &str;
    fn symbol(&self) -> &str;
    fn base(&self) -> &str;

    fn start(&self) {
        println!("Starting Serivce: {:?}", self.name());
    }

    fn stop(&self) {
        println!("Stopping Serivce: {:?}", self.name());
    }
}

pub struct BasicService {
    base_url: String,
    symbol: String,
    enable: bool,
    name: String,
}

impl BasicService {
    pub fn new(base_url: String, symbol: String, enable: bool, name: String) -> Self {
        BasicService {
            base_url,
            symbol,
            enable,
            name,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}
