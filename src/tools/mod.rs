pub mod query;
pub mod mutate;

use mcpr::schema::common::{Tool, ToolInputSchema};
use serde_json::json;
use std::collections::HashMap;

pub fn get_all_tools() -> Vec<Tool> {
    let mut tools = vec![];
    
    // Query tools
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
    
    // Write tools
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
    
    tools.push(Tool {
        name: "insert_block".to_string(),
        description: Some("Insert a new block".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("parent_uuid".to_string(), json!({
                    "type": "string",
                    "description": "UUID of the parent block"
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