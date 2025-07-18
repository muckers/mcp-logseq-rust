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

pub mod query;
pub mod mutate;

use serde_json::json;
use std::collections::HashMap;

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
    let mut tools = vec![];
    
    // ==========================================================================
    // Query Tools - Read-only operations
    // ==========================================================================
    
    tools.push(Tool {
        name: "list_graphs".to_string(),
        description: Some("List available Logseq graphs".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: None,
        },
    });
    
    tools.push(Tool {
        name: "list_pages".to_string(),
        description: Some("List all pages in the current graph".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: None,
        },
    });
    
    tools.push(Tool {
        name: "get_page".to_string(),
        description: Some("Get content of a specific page by name".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("page_name".to_string(), json!({
                    "type": "string",
                    "description": "Name of the page to retrieve"
                }));
                props
            }),
            required: Some(vec!["page_name".to_string()]),
        },
    });
    
    tools.push(Tool {
        name: "get_block".to_string(),
        description: Some("Get a specific block by its UUID".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("uuid".to_string(), json!({
                    "type": "string",
                    "description": "UUID of the block to retrieve"
                }));
                props
            }),
            required: Some(vec!["uuid".to_string()]),
        },
    });
    
    tools.push(Tool {
        name: "search".to_string(),
        description: Some("Search across all pages in the graph".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("query".to_string(), json!({
                    "type": "string",
                    "description": "Search query string"
                }));
                props
            }),
            required: Some(vec!["query".to_string()]),
        },
    });
    
    // ==========================================================================
    // Mutation Tools - Write operations that modify Logseq content
    // ==========================================================================
    
    tools.push(Tool {
        name: "create_page".to_string(),
        description: Some("Create a new page with optional content".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("page_name".to_string(), json!({
                    "type": "string",
                    "description": "Name of the page to create"
                }));
                props.insert("content".to_string(), json!({
                    "type": "string",
                    "description": "Initial content for the page (optional)"
                }));
                props
            }),
            required: Some(vec!["page_name".to_string()]),
        },
    });
    
    tools.push(Tool {
        name: "update_block".to_string(),
        description: Some("Update the content of an existing block".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("uuid".to_string(), json!({
                    "type": "string",
                    "description": "UUID of the block to update"
                }));
                props.insert("content".to_string(), json!({
                    "type": "string",
                    "description": "New content for the block"
                }));
                props
            }),
            required: Some(vec!["uuid".to_string(), "content".to_string()]),
        },
    });
    
    // Insert block tool has complex positioning logic worth documenting
    tools.push(Tool {
        name: "insert_block".to_string(),
        description: Some("Insert a new block with precise positioning control".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("parent_uuid".to_string(), json!({
                    "type": "string",
                    "description": "UUID of the parent block or page"
                }));
                props.insert("content".to_string(), json!({
                    "type": "string",
                    "description": "Content for the new block"
                }));
                props.insert("sibling".to_string(), json!({
                    "type": "boolean",
                    "description": "Whether to insert as sibling (true) or child (false)",
                    "default": false
                }));
                props
            }),
            required: Some(vec!["parent_uuid".to_string(), "content".to_string()]),
        },
    });
    
    tools.push(Tool {
        name: "delete_block".to_string(),
        description: Some("Delete a block by its UUID".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("uuid".to_string(), json!({
                    "type": "string",
                    "description": "UUID of the block to delete"
                }));
                props
            }),
            required: Some(vec!["uuid".to_string()]),
        },
    });
    
    tools.push(Tool {
        name: "append_to_page".to_string(),
        description: Some("Append a block to the end of a page".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("page_name".to_string(), json!({
                    "type": "string",
                    "description": "Name of the page to append to"
                }));
                props.insert("content".to_string(), json!({
                    "type": "string",
                    "description": "Content to append"
                }));
                props
            }),
            required: Some(vec!["page_name".to_string(), "content".to_string()]),
        },
    });
    
    tools
}