//! # Logseq HTTP API Client
//!
//! This module provides a client for interacting with Logseq's HTTP API.
//! The client handles authentication, request formatting, and response parsing
//! for all Logseq operations including querying and modifying graph data.
//!
//! ## Authentication
//!
//! All requests use Bearer token authentication via the `Authorization` header.
//! The token is configured through environment variables and must have appropriate
//! permissions for the requested operations.
//!
//! ## API Structure
//!
//! The Logseq API uses method-based calls where each request specifies:
//! - `method`: The API method name (e.g., "logseq.Editor.getPage")
//! - `args`: Array of arguments for the method
//!
//! ## Error Handling
//!
//! The client checks for API-level errors in responses and converts them
//! to Result errors for consistent error handling throughout the application.

use anyhow::Result;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde_json::Value;
use crate::{config::Config, models::LogseqApiRequest};

/// HTTP client for interacting with the Logseq API.
///
/// Encapsulates the HTTP client and configuration needed to make authenticated
/// requests to a Logseq instance. All API calls go through the `call_api` method
/// which handles request formatting, authentication, and error checking.
pub struct LogseqClient {
    /// The underlying HTTP client for making requests
    client: Client,
    /// Configuration including API URL and authentication token
    config: Config,
}

impl LogseqClient {
    /// Creates a new Logseq API client with the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration containing API URL and authentication token
    ///
    /// # Returns
    ///
    /// A configured client ready to make API requests, or an error if
    /// the HTTP client cannot be created.
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::new();
        Ok(Self { client, config })
    }

    /// Makes an authenticated API call to the Logseq HTTP API.
    ///
    /// This is the core method that all other API methods use. It handles:
    /// - Setting up authentication headers
    /// - Formatting the request in Logseq's expected format
    /// - Making the HTTP request
    /// - Parsing and validating the response
    /// - Converting API errors to Rust Result errors
    ///
    /// # Arguments
    ///
    /// * `method` - The Logseq API method name (e.g., "logseq.Editor.getPage")
    /// * `args` - Vector of arguments to pass to the API method
    ///
    /// # Returns
    ///
    /// The parsed JSON response from the API, or an error if the request failed
    /// or the API returned an error response.
    ///
    /// # Error Handling
    ///
    /// - Network errors are propagated as-is
    /// - JSON parsing errors are propagated as-is  
    /// - API-level errors (in response.error) are converted to anyhow errors
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
            .post(format!("{}/api", self.config.logseq_api_url))
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        // The Logseq API returns the result directly, not wrapped in an object
        let result: Value = response.json().await?;
        
        // Check if it's an error response from the Logseq API
        if let Some(error) = result.get("error") {
            anyhow::bail!("Logseq API error: {}", error);
        }
        
        Ok(result)
    }

    // =============================================================================
    // Query Operations
    // =============================================================================
    // These methods retrieve data from Logseq without modifying anything

    /// Gets information about the currently active graph.
    ///
    /// Returns metadata about the current graph including its name, path,
    /// and other configuration details. Useful for understanding the context
    /// of other operations.
    pub async fn get_current_graph(&self) -> Result<Value> {
        self.call_api("logseq.App.getCurrentGraph", vec![]).await
    }

    /// Retrieves a list of all pages in the current graph.
    ///
    /// Returns an array of page objects, each containing page metadata
    /// like name, creation date, and other properties. This is useful
    /// for getting an overview of all content in the graph.
    pub async fn get_all_pages(&self) -> Result<Value> {
        self.call_api("logseq.Editor.getAllPages", vec![]).await
    }

    /// Gets detailed information about a specific page by name.
    ///
    /// # Arguments
    ///
    /// * `page_name` - The name of the page to retrieve
    ///
    /// # Returns
    ///
    /// Page object containing metadata and properties, or an error if
    /// the page doesn't exist or cannot be accessed.
    pub async fn get_page(&self, page_name: &str) -> Result<Value> {
        self.call_api("logseq.Editor.getPage", vec![Value::String(page_name.to_string())]).await
    }

    /// Gets the complete block tree structure for a page.
    ///
    /// Returns a hierarchical representation of all blocks on the page,
    /// including their content, properties, and parent-child relationships.
    /// This is more comprehensive than just getting page metadata.
    ///
    /// # Arguments
    ///
    /// * `page_name` - The name of the page whose blocks to retrieve
    pub async fn get_page_blocks_tree(&self, page_name: &str) -> Result<Value> {
        self.call_api("logseq.Editor.getPageBlocksTree", vec![Value::String(page_name.to_string())]).await
    }

    /// Retrieves a specific block by its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the block to retrieve
    ///
    /// # Returns
    ///
    /// Block object containing content, properties, parent/child relationships,
    /// and other block metadata.
    pub async fn get_block(&self, uuid: &str) -> Result<Value> {
        self.call_api("logseq.Editor.getBlock", vec![Value::String(uuid.to_string())]).await
    }

    /// Searches across all content in the current graph.
    ///
    /// Uses Logseq's built-in search functionality which provides better
    /// results than simple text matching. Searches through block content,
    /// page names, and properties.
    ///
    /// # Arguments
    ///
    /// * `query` - The search terms to look for
    ///
    /// # Returns
    ///
    /// Array of search results with matching blocks and pages, ranked
    /// by relevance according to Logseq's search algorithm.
    pub async fn search(&self, query: &str) -> Result<Value> {
        self.call_api("logseq.App.search", vec![Value::String(query.to_string())]).await
    }

    // =============================================================================
    // Mutation Operations 
    // =============================================================================
    // These methods modify content in Logseq (create, update, delete)

    /// Creates a new page with optional initial content.
    ///
    /// # Arguments
    ///
    /// * `page_name` - The name for the new page
    /// * `content` - Optional initial content for the page
    ///
    /// # Returns
    ///
    /// The created page object, or an error if the page couldn't be created
    /// (e.g., if a page with that name already exists).
    ///
    /// # Format
    ///
    /// Content is created in markdown format when provided. If no content
    /// is specified, an empty page is created.
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

    /// Inserts a new block into the graph.
    ///
    /// # Arguments
    ///
    /// * `parent_uuid` - UUID of the parent block or page
    /// * `content` - The text content for the new block
    /// * `sibling` - If true, insert as sibling; if false, insert as child
    ///
    /// # Returns
    ///
    /// The created block object with its new UUID and metadata.
    ///
    /// # Block Positioning
    ///
    /// - `sibling: true` - Insert at the same level as the parent block
    /// - `sibling: false` - Insert as a child of the parent block
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

    /// Updates the content of an existing block.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the block to update
    /// * `content` - The new content for the block
    ///
    /// # Returns
    ///
    /// The updated block object, or an error if the block doesn't exist
    /// or cannot be modified.
    ///
    /// # Notes
    ///
    /// This completely replaces the block's content. To append or modify
    /// part of the content, retrieve the current content first, modify it,
    /// then call this method with the full new content.
    pub async fn update_block(&self, uuid: &str, content: &str) -> Result<Value> {
        self.call_api(
            "logseq.Editor.updateBlock",
            vec![
                Value::String(uuid.to_string()),
                Value::String(content.to_string())
            ]
        ).await
    }

    /// Deletes a block from the graph.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID of the block to delete
    ///
    /// # Returns
    ///
    /// Success confirmation, or an error if the block doesn't exist
    /// or cannot be deleted.
    ///
    /// # Warning
    ///
    /// This operation is irreversible. The block and all its child blocks
    /// will be permanently removed from the graph.
    pub async fn delete_block(&self, uuid: &str) -> Result<Value> {
        self.call_api("logseq.Editor.removeBlock", vec![Value::String(uuid.to_string())]).await
    }

    /// Appends a new block to the end of a page.
    ///
    /// # Arguments
    ///
    /// * `page_name` - The name of the page to append to
    /// * `content` - The content for the new block
    ///
    /// # Returns
    ///
    /// The created block object, or an error if the page doesn't exist
    /// or the block cannot be created.
    ///
    /// # Notes
    ///
    /// This is a convenience method that adds a block at the bottom of
    /// a page without needing to know the UUIDs of existing blocks.
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