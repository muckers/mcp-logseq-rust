use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub logseq_api_token: String,
    pub logseq_api_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let logseq_api_token = std::env::var("LOGSEQ_API_TOKEN")
            .map_err(|_| anyhow::anyhow!("LOGSEQ_API_TOKEN not set"))?;
        
        let logseq_api_url = std::env::var("LOGSEQ_API_URL")
            .unwrap_or_else(|_| "http://localhost:12315".to_string());
        
        Ok(Config {
            logseq_api_token,
            logseq_api_url,
        })
    }
}