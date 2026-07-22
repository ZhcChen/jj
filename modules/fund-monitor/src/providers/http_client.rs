use anyhow::{Context, Result};
use reqwest::Client;

#[derive(Clone)]
pub struct HttpClient {
    base_url: String,
    client: Client,
}

impl HttpClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("fund-monitor/0.1")
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client,
        })
    }

    pub async fn get_text(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("请求 {url} 失败"))?;

        let response = response
            .error_for_status()
            .with_context(|| format!("请求 {url} 返回错误状态"))?;

        response
            .text()
            .await
            .with_context(|| format!("读取 {url} 响应失败"))
    }
}
