//! # Mutation Tool Implementations
//!
//! This module implements all MCP tools that modify content in Logseq. These tools
//! provide write access to the graph, enabling creation, modification, and deletion
//! of pages and blocks. All operations are performed through the Logseq HTTP API.
//!
//! ## Safety Considerations
//!
//! These tools modify live Logseq data and should be used carefully:
//! - Deletions are permanent and cannot be undone
//! - Block modifications replace content entirely
//! - Page creation can overwrite existing pages
//!
//! ## Tool Functions
//!
//! Each function validates parameters, performs the requested operation via the
//! Logseq API, and returns success confirmation with relevant data. All functions
//! follow consistent error handling and response formatting patterns.

use anyhow::Result;
use serde_json::Value;
use crate::logseq_client::LogseqClient;

/// Creates a new page in the graph with optional initial content.
///
/// Creates a new page with the specified name and optionally populates it
/// with initial content in markdown format. If a page with the same name
/// already exists, the operation may fail or overwrite the existing page.
///
/// # Parameters
///
/// - `page_name` (required): The name for the new page
/// - `content` (optional): Initial markdown content for the page
///
/// # Returns
///
/// JSON object containing:
/// - `success`: Boolean indicating the operation succeeded
/// - `page`: The created page object with metadata and UUID
///
/// # Errors
///
/// Returns an error if:
/// - The page_name parameter is missing
/// - A page with that name already exists (behavior depends on Logseq settings)
/// - The API request fails due to network or permission issues
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

/// Updates the content of an existing block.
///
/// Completely replaces the content of the specified block with new text.
/// This operation preserves the block's position, properties, and relationships
/// while updating only the text content.
///
/// # Parameters
///
/// - `uuid` (required): The unique identifier of the block to update
/// - `content` (required): The new text content for the block
///
/// # Returns
///
/// JSON object containing:
/// - `success`: Boolean indicating the operation succeeded
/// - `block`: The updated block object with new content
///
/// # Important Notes
///
/// - This completely replaces the block's content (not a partial update)
/// - Block properties and metadata are preserved
/// - Parent-child relationships remain unchanged
/// - Markdown formatting in the content is preserved
///
/// # Errors
///
/// Returns an error if:
/// - Either uuid or content parameters are missing
/// - The specified block UUID doesn't exist
/// - The API request fails due to network or permission issues
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

/// Inserts a new block with precise positioning control.
///
/// Creates a new block and positions it either as a child of the specified
/// parent block or as a sibling at the same hierarchical level. This provides
/// fine-grained control over block organization within the graph structure.
///
/// # Parameters
///
/// - `parent_uuid` (required): UUID of the parent block or page
/// - `content` (required): Text content for the new block
/// - `sibling` (optional): Positioning mode, defaults to false
///   - `true`: Insert as a sibling (same level as parent)
///   - `false`: Insert as a child (nested under parent)
///
/// # Returns
///
/// JSON object containing:
/// - `success`: Boolean indicating the operation succeeded
/// - `block`: The newly created block with UUID and metadata
///
/// # Positioning Logic
///
/// The `sibling` parameter controls hierarchical placement:
/// - **Child mode** (`sibling: false`): New block becomes a child of parent_uuid
/// - **Sibling mode** (`sibling: true`): New block is inserted at same level as parent_uuid
///
/// # Errors
///
/// Returns an error if:
/// - parent_uuid or content parameters are missing
/// - The specified parent UUID doesn't exist
/// - The API request fails due to network or permission issues
pub async fn insert_block(client: &LogseqClient, params: Value) -> Result<Value> {
    let parent_uuid = params["parent_uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("parent_uuid parameter is required"))?;
    
    let content = params["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("content parameter is required"))?;
    
    // Default to child insertion if sibling parameter not specified
    let sibling = params["sibling"].as_bool().unwrap_or(false);
    
    let result = client.insert_block(parent_uuid, content, sibling).await?;
    Ok(serde_json::json!({
        "success": true,
        "block": result
    }))
}

/// Permanently deletes a block from the graph.
///
/// Removes the specified block and all its child blocks from the graph.
/// This operation is irreversible and will permanently destroy the content.
/// Use with caution as there is no undo functionality.
///
/// # Parameters
///
/// - `uuid` (required): The unique identifier of the block to delete
///
/// # Returns
///
/// JSON object containing:
/// - `success`: Boolean indicating the operation succeeded
/// - `result`: Confirmation data from the Logseq API
///
/// # ⚠️ WARNING - Destructive Operation
///
/// This operation:
/// - **Permanently** removes the block and cannot be undone
/// - **Recursively** deletes all child blocks
/// - **Immediately** updates the graph structure
/// - **Cannot** be reversed through the API
///
/// Always verify the UUID before calling this function.
///
/// # Errors
///
/// Returns an error if:
/// - The uuid parameter is missing
/// - The specified block doesn't exist
/// - The block cannot be deleted (e.g., due to permissions)
/// - The API request fails due to network issues
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

/// Appends a new block to the end of a page.
///
/// Creates a new block and adds it to the bottom of the specified page.
/// This is a convenience function that doesn't require knowing existing
/// block UUIDs - it simply adds content to the end of a page.
///
/// # Parameters
///
/// - `page_name` (required): The name of the page to append to
/// - `content` (required): The text content for the new block
///
/// # Returns
///
/// JSON object containing:
/// - `success`: Boolean indicating the operation succeeded
/// - `block`: The newly created block with UUID and metadata
///
/// # Usage Notes
///
/// This is ideal for:
/// - Adding new content to existing pages
/// - Appending notes or updates without complex positioning
/// - Simple content addition workflows
///
/// For more precise block positioning, use `insert_block` instead.
///
/// # Errors
///
/// Returns an error if:
/// - page_name or content parameters are missing
/// - The specified page doesn't exist
/// - The API request fails due to network or permission issues
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