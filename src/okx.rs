use crate::candle::candle_stick;
use crate::common::{BasicService, ExchangeFeed, Service};
use crate::config::Config;
use async_trait::async_trait;
use futures::sink::SinkExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketResponse {
    table: Option<String>,
    data: Option<Vec<CandlestickData>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketResponseArg {
    channel: String,
    instId: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketResponseOkx {
    event: Option<String>,
    arg: WebSocketResponseArg,
    data: Option<Vec<Vec<String>>>, // Data comes as a vector of vectors of strings
}

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketRequest {
    op: String,
    args: Vec<ChannelRequest>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChannelRequest {
    channel: String,
    instId: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CandlestickData {
    timestamp: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

pub struct OKX {
    pub service: BasicService,
}

impl OKX {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn Error>> {
        let config = Config::from_file(config_path)?;

        Ok(OKX {
            service: BasicService::new(
                config.okx.base_url,
                config.okx.symbol,
                config.okx.enable,
                "OKXN".to_string(),
            ),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.service.base_url()
    }

    pub fn enable(&self) -> bool {
        self.service.enable()
    }

    pub fn name(&self) -> &str {
        &self.service.name()
    }

    pub fn symbol(&self) -> &str {
        &self.service.symbol()
    }
}

impl Service for OKX {
    fn name(&self) -> &str {
        &self.name() // ✅ Returns the field value
    }

    fn symbol(&self) -> &str {
        &self.symbol()
    }

    fn base(&self) -> &str {
        &self.base_url()
    }
}

#[async_trait]
impl ExchangeFeed for OKX {
    fn parse_text(text: Utf8Bytes, candle_stick_data: &mut candle_stick) {
        match serde_json::from_str::<WebSocketResponseOkx>(&text) {
            Ok(response) => {
                if let Some(event) = response.event {
                    if (event == "subscribe") {
                        println!("subscribe successful: {:?}", response.arg);
                    } else if (event == "error") {
                        println!("fail subscribe.");
                    }
                } else {
                    if let Some(data) = response.data {
                        // Now you have parsed the data, you can process it
                        for candle in data {
                            candle_stick_data.ts = candle[0].parse::<u64>().unwrap_or(0);
                            candle_stick_data.o = candle[1].parse::<f64>().unwrap_or(0.0);
                            candle_stick_data.h = candle[2].parse::<f64>().unwrap_or(0.0);
                            candle_stick_data.l = candle[3].parse::<f64>().unwrap_or(0.0);
                            candle_stick_data.c = candle[4].parse::<f64>().unwrap_or(0.0);
                            candle_stick_data.v = candle[5].parse::<f64>().unwrap_or(0.0);

                            // Print the candlestick data
                            println!("CandleStick data: {:?}", candle_stick_data);
                        }
                    }
                }
            }
            Err(err) => {
                println!("Failed to deserialize message: {}", err);
            }
        }
    }

    fn get_sub_channel(symbol: &str) -> (&str, &str) {
        // println!("url: {:?}", base);
        let parts_sym: Vec<&str> = symbol.split('@').collect();

        if parts_sym.len() < 2 {
            panic!("incorrect symbol: {}", symbol);
        }

        let symbol = parts_sym[0];
        let channel = parts_sym[1];

        println!("sym: {}, channel:{}", symbol, channel);

        (symbol, channel)
    }

    fn get_connection_str<'a>(base: &'a str, symbol: &'a str) -> String {
        format!("{}", base)
    }

    async fn send_subscribe(
        channel: &str,
        symbol: &str,
        ws_stream: &mut WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    ) {
        let subscribe_message = WebSocketRequest {
            op: "subscribe".to_string(),
            args: vec![ChannelRequest {
                channel: format!("{}", channel),
                instId: format!("{}", symbol),
            }],
        };

        let message = serde_json::to_string(&subscribe_message).expect("sending subscription.");
        ws_stream
            .send(Message::Text(message.into()))
            .await
            .expect("send done.");
    }
}
