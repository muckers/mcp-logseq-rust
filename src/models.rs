//! # Data Models
//!
//! This module defines the data structures used for communication with the Logseq API
//! and for representing Logseq entities within the MCP server. These models provide
//! type safety and serialization support for all API interactions.
//!
//! ## Model Categories
//!
//! - **API Models**: Request/response structures for Logseq HTTP API
//! - **Entity Models**: Representations of Logseq graph entities (pages, blocks, etc.)
//!
//! ## Serialization
//!
//! All models support bidirectional JSON serialization using serde, enabling
//! seamless integration with both the Logseq API and MCP protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// =============================================================================
// API Communication Models
// =============================================================================

/// Request structure for calling Logseq HTTP API methods.
///
/// The Logseq API uses a method-based approach where each request specifies
/// a method name and an array of arguments. This structure represents the
/// format expected by the Logseq API endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct LogseqApiRequest {
    /// The API method to invoke (e.g., "logseq.Editor.getPage")
    pub method: String,
    /// Arguments to pass to the method as a JSON array
    pub args: Vec<Value>,
}

/// Response structure from the Logseq HTTP API.
///
/// Logseq API responses can contain either a successful result or an error.
/// This structure helps parse and handle both cases consistently.
#[derive(Debug, Serialize, Deserialize)]
pub struct LogseqApiResponse {
    /// The successful result data from the API call
    pub result: Option<Value>,
    /// Error message if the API call failed
    pub error: Option<String>,
}

// =============================================================================
// Logseq Entity Models
// =============================================================================

/// Represents a page in a Logseq graph.
///
/// Pages are the top-level organizational units in Logseq. Each page has
/// a unique name and can contain blocks of content. This structure represents
/// the core metadata for a page.
#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    /// The page name (also serves as the page identifier)
    pub name: String,
    /// Unique identifier for the page
    pub uuid: String,
    /// Optional text content of the page
    pub content: Option<String>,
}

/// Represents a block within a Logseq page.
///
/// Blocks are the fundamental content units in Logseq. They can contain text,
/// media, or other content types. Blocks are organized in a hierarchical
/// structure with parent-child relationships.
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    /// Unique identifier for the block
    pub uuid: String,
    /// The text content of the block
    pub content: String,
    /// Name of the page this block belongs to
    pub page: Option<String>,
    /// Child blocks nested under this block
    pub children: Option<Vec<Block>>,
}

/// Represents a Logseq graph.
///
/// A graph is a complete Logseq workspace containing pages, blocks, and
/// their relationships. This structure represents the metadata for a graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    /// Human-readable name of the graph
    pub name: String,
    /// File system path where the graph is stored
    pub path: String,
}