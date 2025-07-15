use anyhow::Result;
use serde_json::Value;
use crate::logseq_client::LogseqClient;

pub async fn list_graphs(client: &LogseqClient, _params: Value) -> Result<Value> {
    let graph = client.get_current_graph().await?;
    Ok(serde_json::json!({
        "graphs": [graph]
    }))
}

pub async fn list_pages(client: &LogseqClient, _params: Value) -> Result<Value> {
    let pages = client.get_all_pages().await?;
    Ok(serde_json::json!({
        "pages": pages
    }))
}

pub async fn get_page(client: &LogseqClient, params: Value) -> Result<Value> {
    let page_name = params["page_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("page_name parameter is required"))?;
    
    let page_info = client.get_page(page_name).await?;
    let blocks = client.get_page_blocks_tree(page_name).await?;
    
    Ok(serde_json::json!({
        "page": page_info,
        "blocks": blocks
    }))
}

pub async fn get_block(client: &LogseqClient, params: Value) -> Result<Value> {
    let uuid = params["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("uuid parameter is required"))?;
    
    let block = client.get_block(uuid).await?;
    Ok(serde_json::json!({
        "block": block
    }))
}

pub async fn search(client: &LogseqClient, params: Value) -> Result<Value> {
    let query = params["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("query parameter is required"))?;
    
    let results = client.search(query).await?;
    Ok(serde_json::json!({
        "results": results
    }))
}