use anyhow::Result;
use serde_json::Value;
use crate::logseq_client::LogseqClient;

pub async fn create_page(client: &LogseqClient, params: Value) -> Result<Value> {
    let page_name = params["page_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("page_name parameter is required"))?;
    
    let content = params["content"].as_str();
    
    let result = client.create_page(page_name, content).await?;
    Ok(serde_json::json!({
        "success": true,
        "page": result
    }))
}

pub async fn update_block(client: &LogseqClient, params: Value) -> Result<Value> {
    let uuid = params["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("uuid parameter is required"))?;
    
    let content = params["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("content parameter is required"))?;
    
    let result = client.update_block(uuid, content).await?;
    Ok(serde_json::json!({
        "success": true,
        "block": result
    }))
}

pub async fn insert_block(client: &LogseqClient, params: Value) -> Result<Value> {
    let parent_uuid = params["parent_uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("parent_uuid parameter is required"))?;
    
    let content = params["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("content parameter is required"))?;
    
    let sibling = params["sibling"].as_bool().unwrap_or(false);
    
    let result = client.insert_block(parent_uuid, content, sibling).await?;
    Ok(serde_json::json!({
        "success": true,
        "block": result
    }))
}

pub async fn delete_block(client: &LogseqClient, params: Value) -> Result<Value> {
    let uuid = params["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("uuid parameter is required"))?;
    
    let result = client.delete_block(uuid).await?;
    Ok(serde_json::json!({
        "success": true,
        "result": result
    }))
}

pub async fn append_to_page(client: &LogseqClient, params: Value) -> Result<Value> {
    let page_name = params["page_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("page_name parameter is required"))?;
    
    let content = params["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("content parameter is required"))?;
    
    let result = client.append_block_in_page(page_name, content).await?;
    Ok(serde_json::json!({
        "success": true,
        "block": result
    }))
}