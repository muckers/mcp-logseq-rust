use anyhow::Result;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde_json::Value;
use crate::{config::Config, models::{LogseqApiRequest, LogseqApiResponse}};

pub struct LogseqClient {
    client: Client,
    config: Config,
}

impl LogseqClient {
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::new();
        Ok(Self { client, config })
    }

    async fn call_api(&self, method: &str, args: Vec<Value>) -> Result<Value> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.logseq_api_token))?,
        );

        let request = LogseqApiRequest {
            method: method.to_string(),
            args,
        };

        let response = self.client
            .post(&format!("{}/api", self.config.logseq_api_url))
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let api_response: LogseqApiResponse = response.json().await?;
        
        if let Some(error) = api_response.error {
            anyhow::bail!("Logseq API error: {}", error);
        }
        
        api_response.result.ok_or_else(|| anyhow::anyhow!("No result from API"))
    }

    // Query operations
    pub async fn get_current_graph(&self) -> Result<Value> {
        self.call_api("logseq.App.getCurrentGraph", vec![]).await
    }

    pub async fn get_all_pages(&self) -> Result<Value> {
        self.call_api("logseq.Editor.getAllPages", vec![]).await
    }

    pub async fn get_page(&self, page_name: &str) -> Result<Value> {
        self.call_api("logseq.Editor.getPage", vec![Value::String(page_name.to_string())]).await
    }

    pub async fn get_page_blocks_tree(&self, page_name: &str) -> Result<Value> {
        self.call_api("logseq.Editor.getPageBlocksTree", vec![Value::String(page_name.to_string())]).await
    }

    pub async fn get_block(&self, uuid: &str) -> Result<Value> {
        self.call_api("logseq.Editor.getBlock", vec![Value::String(uuid.to_string())]).await
    }

    pub async fn search(&self, query: &str) -> Result<Value> {
        self.call_api("logseq.DB.q", vec![Value::String(query.to_string())]).await
    }

    // Write operations
    pub async fn create_page(&self, page_name: &str, content: Option<&str>) -> Result<Value> {
        let mut args = vec![Value::String(page_name.to_string())];
        if let Some(content) = content {
            args.push(serde_json::json!({
                "content": content,
                "format": "markdown"
            }));
        }
        self.call_api("logseq.Editor.createPage", args).await
    }

    pub async fn insert_block(&self, parent_uuid: &str, content: &str, sibling: bool) -> Result<Value> {
        self.call_api(
            "logseq.Editor.insertBlock",
            vec![
                Value::String(parent_uuid.to_string()),
                Value::String(content.to_string()),
                serde_json::json!({ "sibling": sibling })
            ]
        ).await
    }

    pub async fn update_block(&self, uuid: &str, content: &str) -> Result<Value> {
        self.call_api(
            "logseq.Editor.updateBlock",
            vec![
                Value::String(uuid.to_string()),
                Value::String(content.to_string())
            ]
        ).await
    }

    pub async fn delete_block(&self, uuid: &str) -> Result<Value> {
        self.call_api("logseq.Editor.removeBlock", vec![Value::String(uuid.to_string())]).await
    }

    pub async fn append_block_in_page(&self, page_name: &str, content: &str) -> Result<Value> {
        self.call_api(
            "logseq.Editor.appendBlockInPage",
            vec![
                Value::String(page_name.to_string()),
                Value::String(content.to_string())
            ]
        ).await
    }
}