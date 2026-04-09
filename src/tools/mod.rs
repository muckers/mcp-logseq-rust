//! # MCP Tools Module
//!
//! This module defines and manages all tools available through the MCP server.
//! Tools are the core interface that clients use to interact with Logseq data.
//! Each tool has a name, description, and input schema that describes its parameters.
//!
//! ## Tool Categories
//!
//! - **Query Tools**: Read-only operations that retrieve data from Logseq
//! - **Mutation Tools**: Write operations that modify Logseq content
//!
//! ## Tool Definition
//!
//! Each tool must specify:
//! - Unique name for identification
//! - Human-readable description
//! - JSON Schema defining input parameters and their types
//!
//! ## Usage
//!
//! Tools are registered in `get_all_tools()` and their implementations
//! are in the respective `query` and `mutate` modules.

pub mod builder;
pub mod mutate;
pub mod query;

use builder::{ToolBuilder, simple_tool, single_string_param_tool};
use serde_json::{Value, json};
use std::collections::HashMap;

impl Tool {
    /// Serializes this tool to the JSON format expected by MCP clients.
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description.as_deref().unwrap_or(""),
            "inputSchema": {
                "type": self.input_schema.r#type,
                "properties": self.input_schema.properties.as_ref().unwrap_or(&HashMap::new()),
                "required": self.input_schema.required.as_ref().unwrap_or(&Vec::new())
            }
        })
    }
}

/// Represents a single MCP tool with its metadata and input schema.
///
/// Each tool exposed through the MCP interface is represented by this struct,
/// which contains all the information clients need to understand and invoke the tool.
#[derive(Debug, Clone)]
pub struct Tool {
    /// Unique identifier for the tool (used in tool calls)
    pub name: String,
    /// Human-readable description of what the tool does
    pub description: Option<String>,
    /// JSON Schema definition of the tool's input parameters
    pub input_schema: ToolInputSchema,
}

/// JSON Schema definition for a tool's input parameters.
///
/// Follows the JSON Schema specification to describe what parameters
/// a tool accepts, their types, and which ones are required.
#[derive(Debug, Clone)]
pub struct ToolInputSchema {
    /// The schema type (typically "object" for tools)
    pub r#type: String,
    /// Definition of each parameter with its type and constraints
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// List of parameter names that are required
    pub required: Option<Vec<String>>,
}

/// Returns a complete list of all tools available through this MCP server.
///
/// This function registers and configures all tools that clients can invoke.
/// Each tool is defined with its name, description, and input schema.
/// The tools are organized into two categories: query tools (read-only)
/// and mutation tools (write operations).
///
/// ## Tool Registration
///
/// Tools must be added here to be discoverable by MCP clients. Each tool
/// definition includes:
/// - A unique name used in tool calls
/// - A description explaining what the tool does
/// - A JSON Schema defining expected parameters
///
/// ## Schema Guidelines
///
/// - Use "object" type for tools with parameters
/// - Define all parameters in the properties map
/// - List required parameters in the required array
/// - Include descriptions for each parameter
pub fn get_all_tools() -> Vec<Tool> {
    vec![
        // ==========================================================================
        // Query Tools - Read-only operations
        // ==========================================================================
        simple_tool("list_graphs", "List available Logseq graphs"),
        ToolBuilder::new("list_pages")
            .description("List pages in the current graph, with optional name filter and limit")
            .string_param(
                "name_contains",
                "Filter pages whose name contains this substring (case-insensitive)",
                false,
            )
            .string_param(
                "limit",
                "Maximum number of pages to return (default: 100)",
                false,
            )
            .build(),
        single_string_param_tool(
            "get_page",
            "Get content of a specific page by name",
            "page_name",
            "Name of the page to retrieve",
        ),
        single_string_param_tool(
            "get_block",
            "Get a specific block by its UUID",
            "uuid",
            "UUID of the block to retrieve",
        ),
        single_string_param_tool(
            "search",
            "Full-text search across all pages and blocks in the graph",
            "query",
            "Search query string",
        ),
        single_string_param_tool(
            "query",
            "Run a Datascript/Datalog query against the graph database for precise structured queries",
            "query",
            "Datalog query string, e.g. [:find ?n :where [?b :block/name ?n]]",
        ),
        simple_tool(
            "get_today_journal",
            "Get the content of today's journal page",
        ),
        single_string_param_tool(
            "get_page_references",
            "Get all blocks that link to a given page (backlinks)",
            "page_name",
            "Name of the page to find references for",
        ),
        single_string_param_tool(
            "get_block_properties",
            "Get all properties (tags, type, priority, etc.) on a specific block",
            "uuid",
            "UUID of the block",
        ),
        // ==========================================================================
        // Mutation Tools - Write operations that modify Logseq content
        // ==========================================================================
        ToolBuilder::new("create_page")
            .description("Create a new page with optional initial content")
            .string_param("page_name", "Name of the page to create", true)
            .string_param("content", "Initial content for the page (optional)", false)
            .build(),
        ToolBuilder::new("update_block")
            .description("Replace the content of an existing block")
            .string_param("uuid", "UUID of the block to update", true)
            .string_param("content", "New content for the block", true)
            .build(),
        ToolBuilder::new("insert_block")
            .description("Insert a new block as a child or sibling of an existing block")
            .string_param("parent_uuid", "UUID of the parent block or page", true)
            .string_param("content", "Content for the new block", true)
            .bool_param(
                "sibling",
                "Insert as sibling (true) or child (false, default)",
                Some(false),
                false,
            )
            .build(),
        single_string_param_tool(
            "delete_block",
            "Permanently delete a block and all its children by UUID",
            "uuid",
            "UUID of the block to delete",
        ),
        single_string_param_tool(
            "delete_page",
            "Permanently delete a page and all its blocks by name",
            "page_name",
            "Name of the page to delete",
        ),
        ToolBuilder::new("append_to_page")
            .description("Append a new block to the end of a page")
            .string_param("page_name", "Name of the page to append to", true)
            .string_param("content", "Content to append", true)
            .build(),
        single_string_param_tool(
            "append_to_journal",
            "Append a block to today's journal page",
            "content",
            "Content to append to today's journal",
        ),
        ToolBuilder::new("set_block_property")
            .description("Set a property (key-value pair) on a block, e.g. type, priority, tags")
            .string_param("uuid", "UUID of the block", true)
            .string_param(
                "key",
                "Property name (e.g. 'type', 'priority', 'tags')",
                true,
            )
            .string_param("value", "Property value", true)
            .build(),
        ToolBuilder::new("remove_block_property")
            .description("Remove a property from a block")
            .string_param("uuid", "UUID of the block", true)
            .string_param("key", "Property name to remove", true)
            .build(),
    ]
}
