use async_trait::async_trait;

#[async_trait]
pub trait ExchangeFeed {
    const WS_URL: &'static str;
    async fn connect(
        symbols: &str,
        on_success: impl FnOnce(String) + Send,
    ) -> Result<String, String>;
}