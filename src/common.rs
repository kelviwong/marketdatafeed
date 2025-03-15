use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait ExchangeFeed {
    async fn connect(&self,
        on_success: impl FnOnce(String) + Send,
    ) -> Result<String, String>;
}