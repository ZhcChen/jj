pub mod fund_source;
pub mod http_client;

use anyhow::Result;
use fund_source::EastmoneyFundSource;
use http_client::HttpClient;

const EASTMONEY_BASE_URL: &str = "https://fund.eastmoney.com";

pub fn default_fund_source() -> Result<EastmoneyFundSource> {
    Ok(EastmoneyFundSource::new(HttpClient::new(
        EASTMONEY_BASE_URL,
    )?))
}
