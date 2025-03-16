use crate::candle::candle_stick;
use crate::common::{BasicService, Exchange, ExchangeFeed, Service};
use crate::config::Config;
use async_trait::async_trait;
use serde::Deserialize;
use tokio_tungstenite::tungstenite::Utf8Bytes;

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

// private field ensure caller cannot create this struct directly.
// so it need to go through new
pub struct Binance {
    pub service: BasicService,
}

impl Binance {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::from_file(config_path)?;

        Ok(Binance {
            service: BasicService::new(
                config.binance.base_url,
                config.binance.symbol,
                config.binance.enable,
                "Binance".to_string(),
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

impl Exchange for Binance {
    fn new(config_path: &str) -> Self where Self: Sized {
        Binance::new(config_path).unwrap()
    }
}

impl Service for Binance {
    fn name(&self) -> &str {
        &self.name() // ✅ Returns the field value
    }
    
    fn symbol(&self) -> &str {
        &self.symbol()
    }
    
    fn base(&self) -> &str {
        &self.base_url()
    }

    fn enable(&self) -> bool {
        self.enable()
    }
}

#[async_trait]
impl ExchangeFeed for Binance {
    fn parse_text(text: Utf8Bytes, candle_stick_data: &mut candle_stick) {
        match serde_json::from_str::<WebSocketMessage>(&text) {
            Ok(ws_message) => {
                // Update candle stick data if deserialization is successful
                candle_stick_data.ts = ws_message.kline.open_time;
                candle_stick_data.c = ws_message.kline.close.parse::<f64>().unwrap_or(0.0);
                candle_stick_data.o = ws_message.kline.open.parse::<f64>().unwrap_or(0.0);
                candle_stick_data.h = ws_message.kline.high.parse::<f64>().unwrap_or(0.0);
                candle_stick_data.l = ws_message.kline.low.parse::<f64>().unwrap_or(0.0);
                candle_stick_data.v = ws_message.kline.volume.parse::<f64>().unwrap_or(0.0);

                println!("CandleStick data: {:?}", candle_stick_data);
            }
            Err(e) => {
                println!("Failed to deserialize message: {:?}", e);
                println!("{:?}", text);
            }
        }
    }

    fn get_connection_str<'a>(base: &'a str, symbol: &'a str) -> String {
        format!("{}{}",base, symbol)
    }

    fn get_sub_channel(symbol: &str) -> (&str, &str) {
        (symbol, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
