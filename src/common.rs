use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait ExchangeFeed : Service {
    async fn connect(&self,
        on_success: impl FnOnce(String) + Send,
    ) -> Result<String, String>;
}

pub trait Service {
    fn name(&self) -> &str;

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