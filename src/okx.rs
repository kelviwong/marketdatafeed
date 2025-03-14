use async_trait::async_trait;
use crate::candle::CandleStickBuilder;
use crate::common::ExchangeFeed;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
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

pub struct OKX;

#[async_trait]
impl ExchangeFeed for OKX {
    const WS_URL: &str = "wss://ws.okx.com:8443/ws/v5/business";

    async fn connect(
        symbols: &str,
        on_success: impl FnOnce(String) + Send,
    ) -> Result<String, String> {
        println!("url: {:?}", Self::WS_URL);
        let (mut ws_stream, _) = connect_async(Self::WS_URL).await.expect("connected.");

        let subscribe_message = WebSocketRequest {
            op: "subscribe".to_string(),
            args: vec![ChannelRequest {
                channel: "candle1m".to_string(),
                instId: format!("{}", symbols),
            }],
        };

        let message = serde_json::to_string(&subscribe_message).expect("sending subscription.");
        ws_stream
            .send(Message::Text(message.into()))
            .await
            .expect("send done.");

        on_success("OKX connected.".to_string());

        let mut candle_stick_data = CandleStickBuilder::new().build();
        candle_stick_data.source = "OKX".to_string();

        // Listen for messages from WebSocket
        // Listen for incoming messages
        while let Some(Ok(msg)) = ws_stream.next().await {
            match msg {
                Message::Text(text) => {
                    // Deserialize the received message to WebSocketResponse
                    match serde_json::from_str::<WebSocketResponseOkx>(&text) {
                        Ok(response) => {
                            if let Some(event) = response.event {
                                if (event == "subscribe") {
                                    println!("subscribe successful");
                                } else if (event == "error") {
                                    println!("fail subscribe.");
                                }
                            } else {
                                if let Some(data) = response.data {
                                    // Now you have parsed the data, you can process it
                                    for candle in data {
                                        candle_stick_data.ts =
                                            candle[0].parse::<u64>().unwrap_or(0);
                                        candle_stick_data.o =
                                            candle[1].parse::<f64>().unwrap_or(0.0);
                                        candle_stick_data.h =
                                            candle[2].parse::<f64>().unwrap_or(0.0);
                                        candle_stick_data.l =
                                            candle[3].parse::<f64>().unwrap_or(0.0);
                                        candle_stick_data.c =
                                            candle[4].parse::<f64>().unwrap_or(0.0);
                                        candle_stick_data.v =
                                            candle[5].parse::<f64>().unwrap_or(0.0);

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
                _ => (),
            }
        }

        Ok("Finished. OKX".to_string())
    }
}