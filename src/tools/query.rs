//! # Query Tool Implementations
//!
//! This module implements all read-only MCP tools that retrieve data from Logseq
//! without making any modifications. These tools provide safe access to graph
//! data for analysis, search, and content retrieval operations.
//!
//! ## Tool Functions
//!
//! Each function in this module corresponds to a specific MCP tool that clients
//! can invoke. All functions follow the same pattern:
//! - Accept a Logseq client and parameters
//! - Validate required parameters
//! - Call appropriate Logseq API methods
//! - Return formatted results as JSON
//!
//! ## Error Handling
//!
//! Parameter validation errors are returned immediately with descriptive messages.
//! API errors from Logseq are propagated up to the MCP layer for consistent handling.

use anyhow::Result;
use serde_json::Value;
use crate::logseq_client::LogseqClient;

/// Lists available Logseq graphs.
///
/// Currently returns information about the active graph, as the Logseq API
/// primarily works with the currently open graph. In the future, this could
/// be extended to list multiple graphs if the API supports it.
///
/// # Parameters
///
/// No parameters required.
///
/// # Returns
///
/// JSON object containing an array of graph information with name, path, and metadata.
pub async fn list_graphs(client: &LogseqClient, _params: Value) -> Result<Value> {
    let graph = client.get_current_graph().await?;
    Ok(serde_json::json!({
        "graphs": [graph]
    }))
}

/// Retrieves a list of all pages in the current graph.
///
/// Returns comprehensive information about every page in the graph, including
/// page names, UUIDs, creation dates, and other metadata. Useful for getting
/// an overview of the graph's structure and content.
///
/// # Parameters
///
/// No parameters required.
///
/// # Returns
///
/// JSON object containing an array of page objects with metadata for each page
/// in the graph.
pub async fn list_pages(client: &LogseqClient, _params: Value) -> Result<Value> {
    let pages = client.get_all_pages().await?;
    Ok(serde_json::json!({
        "pages": pages
    }))
}

/// Retrieves comprehensive information about a specific page.
///
/// Fetches both the page metadata and the complete block tree structure
/// for the specified page. This provides a complete view of the page's
/// content and organization.
///
/// # Parameters
///
/// - `page_name` (required): The name of the page to retrieve
///
/// # Returns
///
/// JSON object containing:
/// - `page`: Page metadata (name, UUID, properties)
/// - `blocks`: Complete hierarchical block tree for the page
///
/// # Errors
///
/// Returns an error if the page_name parameter is missing or if the page
/// doesn't exist in the graph.
pub async fn get_page(client: &LogseqClient, params: Value) -> Result<Value> {
    let page_name = params["page_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("page_name parameter is required"))?;
    
    // Fetch both page metadata and block structure for complete information
    let page_info = client.get_page(page_name).await?;
    let blocks = client.get_page_blocks_tree(page_name).await?;
    
    Ok(serde_json::json!({
        "page": page_info,
        "blocks": blocks
    }))
}

/// Retrieves a specific block by its UUID.
///
/// Fetches detailed information about a single block, including its content,
/// properties, parent/child relationships, and metadata. Useful for examining
/// specific pieces of content within the graph.
///
/// # Parameters
///
/// - `uuid` (required): The unique identifier of the block to retrieve
///
/// # Returns
///
/// JSON object containing the complete block information including content,
/// properties, relationships, and metadata.
///
/// # Errors
///
/// Returns an error if the uuid parameter is missing or if no block exists
/// with the specified UUID.
pub async fn get_block(client: &LogseqClient, params: Value) -> Result<Value> {
    let uuid = params["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("uuid parameter is required"))?;
    
    let block = client.get_block(uuid).await?;
    Ok(serde_json::json!({
        "block": block
    }))
}

/// Searches across all content in the graph.
///
/// Performs a full-text search using Logseq's built-in search engine, which
/// provides intelligent ranking and supports various search features like
/// exact phrases, property searches, and content type filtering.
///
/// # Parameters
///
/// - `query` (required): The search terms to look for
///
/// # Returns
///
/// JSON object containing an array of search results, each with:
/// - Matching content snippets
/// - Page/block context information
/// - Relevance ranking from Logseq's search algorithm
///
/// # Search Features
///
/// The search supports:
/// - Full-text search across all blocks and pages
/// - Property searches (e.g., searching for specific tags or metadata)
/// - Phrase searches with quotes
/// - Advanced filtering based on content type
///
/// # Errors
///
/// Returns an error if the query parameter is missing.
pub async fn search(client: &LogseqClient, params: Value) -> Result<Value> {
    let query = params["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("query parameter is required"))?;
    
    let results = client.search(query).await?;
    Ok(serde_json::json!({
        "results": results
    }))
}